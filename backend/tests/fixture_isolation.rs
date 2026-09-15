mod common;

use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

async fn setup_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&common::test_database_url())
        .await
        .expect("connect to TEST as migrator")
}

fn unique_client_id() -> String {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let suffix: String = Uuid::new_v4()
        .as_bytes()
        .iter()
        .map(|byte| ALPHABET[(*byte & 31) as usize] as char)
        .collect();
    format!("XFC{suffix}")
}

fn unique_hash() -> [u8; 32] {
    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    let mut hash = [0_u8; 32];
    hash[..16].copy_from_slice(first.as_bytes());
    hash[16..].copy_from_slice(second.as_bytes());
    hash
}

async fn protected_external_setup(
    setup: PgPool,
    actor_ready: tokio::sync::oneshot::Sender<Uuid>,
    continue_setup: tokio::sync::oneshot::Receiver<()>,
    client_ready: tokio::sync::oneshot::Sender<(Uuid, Uuid)>,
    continue_cleanup: tokio::sync::oneshot::Receiver<()>,
    cleanup_done: tokio::sync::oneshot::Sender<Result<(), String>>,
    application_name: String,
) -> Result<(Uuid, Uuid), sqlx::Error> {
    let _lock = common::acquire_test_fixture_lock_named(&application_name).await;
    let actor_id: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash)
         VALUES ($1,'fixture-isolation-green') RETURNING id",
    )
    .bind(format!(
        "fixture-isolation-green-{}@test.invalid",
        Uuid::new_v4()
    ))
    .fetch_one(&setup)
    .await?;
    actor_ready.send(actor_id).expect("signal actor creation");
    continue_setup.await.expect("continue fixture setup");
    let client_id: Uuid = sqlx::query_scalar(
        "INSERT INTO api_clients (
             client_id,display_name,description,status,
             created_by_staff_user_id,updated_by_staff_user_id
         ) VALUES ($1,'Fixture isolation GREEN',NULL,'ACTIVE',$2,$2)
         RETURNING id",
    )
    .bind(unique_client_id())
    .bind(actor_id)
    .fetch_one(&setup)
    .await?;
    client_ready
        .send((actor_id, client_id))
        .expect("signal external fixture creation");
    continue_cleanup
        .await
        .expect("continue external fixture cleanup");
    let cleanup_result = cleanup_external_fixture(&setup, client_id, actor_id).await;
    let cleanup_report = cleanup_result
        .as_ref()
        .map(|_| ())
        .map_err(ToString::to_string);
    cleanup_done
        .send(cleanup_report)
        .expect("signal external fixture cleanup");
    cleanup_result?;
    Ok((actor_id, client_id))
}

async fn wait_for_fixture_lock_wait(setup: &PgPool, application_name: &str) {
    let observation = async {
        loop {
            let waiting: i64 = sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM pg_stat_activity AS waiter
                 WHERE waiter.datname=current_database()
                   AND waiter.application_name=$1
                   AND EXISTS (
                       SELECT 1 FROM pg_locks AS blocked
                       WHERE blocked.pid=waiter.pid
                         AND blocked.locktype='advisory'
                         AND NOT blocked.granted
                   )",
            )
            .bind(application_name)
            .fetch_one(setup)
            .await
            .expect("inspect TEST fixture lock wait");
            if waiting == 1 {
                return;
            }
            tokio::task::yield_now().await;
        }
    };
    tokio::time::timeout(Duration::from_secs(10), observation)
        .await
        .expect("fixture operation did not reach advisory-lock wait");
}

async fn cleanup_external_fixture(
    setup: &PgPool,
    client_id: Uuid,
    actor_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM api_clients WHERE id=$1")
        .bind(client_id)
        .execute(setup)
        .await?;
    sqlx::query("DELETE FROM staff_users WHERE id=$1")
        .bind(actor_id)
        .execute(setup)
        .await?;
    Ok(())
}

#[tokio::test]
async fn external_fixture_setup_is_serialized_with_staff_cleanup() {
    let setup = setup_pool().await;
    let (actor_ready_tx, actor_ready_rx) = tokio::sync::oneshot::channel();
    let (continue_tx, continue_rx) = tokio::sync::oneshot::channel();
    let (client_ready_tx, client_ready_rx) = tokio::sync::oneshot::channel();
    let (continue_cleanup_tx, continue_cleanup_rx) = tokio::sync::oneshot::channel();
    let (cleanup_done_tx, cleanup_done_rx) = tokio::sync::oneshot::channel();
    let external_application_name = format!("fixture-isolation-external-{}", Uuid::new_v4());
    let external = tokio::spawn(protected_external_setup(
        setup.clone(),
        actor_ready_tx,
        continue_rx,
        client_ready_tx,
        continue_cleanup_rx,
        cleanup_done_tx,
        external_application_name,
    ));
    let actor_id = actor_ready_rx.await.expect("actor became visible");

    let staff_setup = setup.clone();
    let staff_name = format!("fixture-isolation-staff-{}", Uuid::new_v4());
    let staff_name_for_task = staff_name.clone();
    let staff = tokio::spawn(async move {
        let _lock = common::acquire_test_fixture_lock_named(&staff_name_for_task).await;
        let result = sqlx::query("DELETE FROM staff_users WHERE id=$1")
            .bind(actor_id)
            .execute(&staff_setup)
            .await;
        result
    });
    wait_for_fixture_lock_wait(&setup, &staff_name).await;
    continue_tx.send(()).expect("continue external setup");

    let (created_actor_id, client_id) =
        client_ready_rx.await.expect("external fixture was created");
    assert_eq!(created_actor_id, actor_id);
    let body_setup = setup.clone();
    common::run_fixture_body_with_cleanup(
        move || async move {
            let client_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM api_clients WHERE id=$1")
                    .bind(client_id)
                    .fetch_one(&body_setup)
                    .await
                    .expect("inspect protected external fixture");
            assert_eq!(client_count, 1);
        },
        move || async move {
            continue_cleanup_tx
                .send(())
                .map_err(|_| "continue external fixture cleanup channel closed".to_owned())?;
            cleanup_done_rx
                .await
                .map_err(|_| "external fixture cleanup did not complete".to_owned())??;
            Ok::<(), String>(())
        },
    )
    .await;
    external
        .await
        .expect("external setup task")
        .expect("protected fixture setup");
    staff
        .await
        .expect("staff cleanup task")
        .expect("staff cleanup");
}

#[derive(Clone, Copy)]
struct CleanupFixture {
    actor_id: Uuid,
    client_id: Uuid,
}

async fn create_cleanup_fixture(setup: &PgPool) -> CleanupFixture {
    let mut transaction = setup.begin().await.expect("begin cleanup fixture");
    let actor_id: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash)
         VALUES ($1,'fixture-cleanup-green') RETURNING id",
    )
    .bind(format!(
        "fixture-cleanup-green-{}@test.invalid",
        Uuid::new_v4()
    ))
    .fetch_one(&mut *transaction)
    .await
    .expect("fixture actor");
    let client_id: Uuid = sqlx::query_scalar(
        "INSERT INTO api_clients (
             client_id,display_name,description,status,
             created_by_staff_user_id,updated_by_staff_user_id
         ) VALUES ($1,'Fixture cleanup GREEN',NULL,'ACTIVE',$2,$2)
         RETURNING id",
    )
    .bind(unique_client_id())
    .bind(actor_id)
    .fetch_one(&mut *transaction)
    .await
    .expect("fixture client");
    sqlx::query(
        "INSERT INTO api_client_allowed_scopes
         (api_client_id,scope_code,assigned_by_staff_user_id)
         VALUES ($1,'flights:read',$2)",
    )
    .bind(client_id)
    .bind(actor_id)
    .execute(&mut *transaction)
    .await
    .expect("fixture scope");
    let credential_id: Uuid = sqlx::query_scalar(
        "INSERT INTO api_client_credentials
         (api_client_id,secret_digest,digest_version,issued_at,issued_by_staff_user_id)
         VALUES ($1,$2,1,clock_timestamp(),$3) RETURNING id",
    )
    .bind(client_id)
    .bind([0x33_u8; 32].as_slice())
    .bind(actor_id)
    .fetch_one(&mut *transaction)
    .await
    .expect("fixture credential");
    sqlx::query(
        "INSERT INTO external_access_tokens
         (api_client_credential_id,token_hash,issued_at,expires_at)
         VALUES ($1,$2,clock_timestamp(),clock_timestamp()+interval '15 minutes')",
    )
    .bind(credential_id)
    .bind(unique_hash().as_slice())
    .execute(&mut *transaction)
    .await
    .expect("fixture token");
    sqlx::query(
        "INSERT INTO api_client_management_audit
         (api_client_id,actor_staff_user_id,action,before_state,after_state,credential_id)
         VALUES ($1,$2,'CREDENTIAL_ISSUED',NULL,'{}'::jsonb,$3)",
    )
    .bind(client_id)
    .bind(actor_id)
    .bind(credential_id)
    .execute(&mut *transaction)
    .await
    .expect("fixture audit");
    transaction.commit().await.expect("commit cleanup fixture");
    CleanupFixture {
        actor_id,
        client_id,
    }
}

async fn cleanup_fixture(setup: &PgPool, fixture: CleanupFixture) -> Result<(), sqlx::Error> {
    let mut transaction = setup.begin().await?;
    sqlx::query(
        "DELETE FROM api_client_management_audit
         WHERE api_client_id=$1",
    )
    .bind(fixture.client_id)
    .execute(&mut *transaction)
    .await?;
    sqlx::query(
        "DELETE FROM external_access_tokens
         WHERE api_client_credential_id IN
             (SELECT id FROM api_client_credentials WHERE api_client_id=$1)",
    )
    .bind(fixture.client_id)
    .execute(&mut *transaction)
    .await?;
    sqlx::query("DELETE FROM api_client_credentials WHERE api_client_id=$1")
        .bind(fixture.client_id)
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1")
        .bind(fixture.client_id)
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM api_clients WHERE id=$1")
        .bind(fixture.client_id)
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM staff_users WHERE id=$1")
        .bind(fixture.actor_id)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await
}

#[tokio::test]
async fn fixture_body_failure_still_cleans_owned_rows_and_preserves_failure() {
    let _lock = common::acquire_test_fixture_lock().await;
    let setup = setup_pool().await;
    let fixture = create_cleanup_fixture(&setup).await;
    let cleanup_setup = setup.clone();
    let cleanup_fixture_value = fixture;
    let body = tokio::spawn(async move {
        common::run_fixture_body_with_cleanup(
            || async { panic!("intentional fixture body failure") },
            move || {
                let setup = cleanup_setup.clone();
                async move { cleanup_fixture(&setup, cleanup_fixture_value).await }
            },
        )
        .await;
    })
    .await;
    assert!(body.is_err(), "primary inner failure must be preserved");

    let remaining: (i64, i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT
            (SELECT COUNT(*) FROM staff_users WHERE id=$1),
            (SELECT COUNT(*) FROM api_clients WHERE id=$2),
            (SELECT COUNT(*) FROM api_client_allowed_scopes WHERE api_client_id=$2),
            (SELECT COUNT(*) FROM api_client_credentials WHERE api_client_id=$2),
            (SELECT COUNT(*) FROM external_access_tokens token
             JOIN api_client_credentials credential ON credential.id=token.api_client_credential_id
             WHERE credential.api_client_id=$2),
            (SELECT COUNT(*) FROM api_client_management_audit WHERE api_client_id=$2)",
    )
    .bind(fixture.actor_id)
    .bind(fixture.client_id)
    .fetch_one(&setup)
    .await
    .expect("inspect cleaned fixture");
    assert_eq!(remaining, (0, 0, 0, 0, 0, 0));
}

#[tokio::test]
async fn owned_fixture_cleanup_preserves_unrelated_fixture_rows() {
    let _lock = common::acquire_test_fixture_lock().await;
    let setup = setup_pool().await;
    let owned = create_cleanup_fixture(&setup).await;
    let unrelated = create_cleanup_fixture(&setup).await;
    let body_setup = setup.clone();
    let cleanup_setup = setup.clone();
    common::run_fixture_body_with_cleanup(
        move || async move {
            cleanup_fixture(&body_setup, owned)
                .await
                .expect("owned fixture cleanup");
            let counts: (i64, i64, i64, i64, i64, i64) = sqlx::query_as(
                "SELECT
                    (SELECT COUNT(*) FROM staff_users WHERE id=$1),
                    (SELECT COUNT(*) FROM api_clients WHERE id=$2),
                    (SELECT COUNT(*) FROM api_client_allowed_scopes WHERE api_client_id=$2),
                    (SELECT COUNT(*) FROM api_client_credentials WHERE api_client_id=$2),
                    (SELECT COUNT(*) FROM external_access_tokens token
                     JOIN api_client_credentials credential ON credential.id=token.api_client_credential_id
                     WHERE credential.api_client_id=$2),
                    (SELECT COUNT(*) FROM api_client_management_audit WHERE api_client_id=$2)",
            )
            .bind(unrelated.actor_id)
            .bind(unrelated.client_id)
            .fetch_one(&body_setup)
            .await
            .expect("inspect unrelated fixture");
            assert_eq!(counts, (1, 1, 1, 1, 1, 1));
        },
        move || async move {
            cleanup_fixture(&cleanup_setup, owned).await?;
            cleanup_fixture(&cleanup_setup, unrelated).await
        },
    )
    .await;
}

#[tokio::test]
async fn cleanup_failure_does_not_replace_primary_fixture_panic() {
    let result = tokio::spawn(async {
        common::run_fixture_body_with_cleanup(
            || async { panic!("primary fixture panic") },
            || async { Err::<(), _>("secondary cleanup failure") },
        )
        .await;
    })
    .await
    .expect_err("primary panic must be returned to the test");
    let payload = result.into_panic();
    let message = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str));
    assert_eq!(message, Some("primary fixture panic"));
}

async fn panicking_cleanup() -> Result<(), &'static str> {
    panic!("secondary cleanup panic");
}

#[tokio::test]
async fn cleanup_panic_does_not_replace_primary_fixture_panic() {
    let result = tokio::spawn(async {
        common::run_fixture_body_with_cleanup(
            || async { panic!("primary fixture panic") },
            panicking_cleanup,
        )
        .await;
    })
    .await
    .expect_err("primary panic must remain the propagated failure");
    let payload = result.into_panic();
    let message = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str));
    assert_eq!(message, Some("primary fixture panic"));
}

#[tokio::test]
async fn cleanup_panic_fails_a_successful_fixture_body() {
    let result = tokio::spawn(async {
        common::run_fixture_body_with_cleanup(|| async {}, panicking_cleanup).await;
    })
    .await
    .expect_err("cleanup panic must fail an otherwise successful body");
    assert!(result.is_panic());
}
