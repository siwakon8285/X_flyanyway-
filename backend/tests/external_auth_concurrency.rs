mod common;

use std::{str::FromStr, sync::Arc, time::Duration};

use chrono::{Duration as ChronoDuration, Utc};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool, Postgres, Transaction,
};
use tokio::sync::oneshot;
use uuid::Uuid;
use x_fly_api::{
    application::external_auth::{ExternalAuthRepository, ExternalCredentialCrypto},
    domain::{
        api_client::ApiClientScope,
        external_api::{
            AccessTokenHash, CredentialAdministrationError, CredentialDigest,
            CredentialRevocationReason, ExternalApiCredentialPepper, ExternalTokenExchangeError,
            PlaintextClientSecret,
        },
    },
    infrastructure::{
        database::{
            migrate_database, revoke_client_credentials_and_tokens, verify_database_ready,
            SqlxExternalAuthRepository,
        },
        external_auth_crypto::HmacExternalCredentialCrypto,
    },
};

fn pepper() -> ExternalApiCredentialPepper {
    ExternalApiCredentialPepper::parse_hex(
        "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
    )
    .unwrap()
}

struct Fixture {
    client_pk: Uuid,
    client_id: String,
    actor_id: Uuid,
}

struct TestPools {
    setup: PgPool,
    runtime: PgPool,
    runtime_application_name: String,
}

async fn pools() -> TestPools {
    let setup = PgPoolOptions::new()
        .max_connections(10)
        .connect(&common::test_database_url())
        .await
        .expect("connect to TEST as migrator");
    migrate_database(&setup).await.unwrap();
    let runtime_application_name = format!("branch25-task3-{}", Uuid::new_v4());
    let runtime_options = PgConnectOptions::from_str(&common::test_runtime_database_url())
        .expect("valid TEST runtime connection options")
        .application_name(&runtime_application_name);
    let runtime = PgPoolOptions::new()
        .max_connections(10)
        .connect_with(runtime_options)
        .await
        .expect("connect to TEST as runtime");
    verify_database_ready(&runtime).await.unwrap();
    TestPools {
        setup,
        runtime,
        runtime_application_name,
    }
}

async fn runtime_pool_with_application_name(application_name: &str) -> PgPool {
    let options = PgConnectOptions::from_str(&common::test_runtime_database_url())
        .expect("valid TEST runtime connection options")
        .application_name(application_name);
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("connect to TEST as named runtime actor");
    verify_database_ready(&pool).await.unwrap();
    pool
}

async fn wait_for_runtime_lock_wait(
    setup: &PgPool,
    application_name: &str,
    blocker_pid: i32,
    expected_waiters: i64,
) {
    let observation = async {
        loop {
            let waiting: i64 = sqlx::query_scalar(
                "WITH RECURSIVE blocker_chain(waiter_pid, blocker_pid, depth) AS (
                     SELECT waiter.pid, direct.blocker_pid, 1
                     FROM pg_stat_activity AS waiter
                     CROSS JOIN LATERAL unnest(pg_blocking_pids(waiter.pid)) AS direct(blocker_pid)
                     WHERE waiter.datname=current_database()
                       AND waiter.application_name=$1
                       AND EXISTS (
                           SELECT 1
                           FROM pg_locks AS blocked
                           WHERE blocked.pid=waiter.pid
                             AND NOT blocked.granted
                       )
                     UNION ALL
                     SELECT chain.waiter_pid, next.blocker_pid, chain.depth+1
                     FROM blocker_chain AS chain
                     CROSS JOIN LATERAL unnest(pg_blocking_pids(chain.blocker_pid)) AS next(blocker_pid)
                     WHERE chain.depth < 8
                 )
                 SELECT COUNT(DISTINCT waiter_pid)
                 FROM blocker_chain
                 WHERE blocker_pid=$2",
            )
            .bind(application_name)
            .bind(blocker_pid)
            .fetch_one(setup)
            .await
            .expect("inspect TEST lock wait");
            if waiting >= expected_waiters {
                break;
            }
            tokio::task::yield_now().await;
        }
    };
    tokio::time::timeout(Duration::from_secs(10), observation)
        .await
        .expect("runtime operation did not reach the expected PostgreSQL lock wait");
}

async fn hold_client_lock(setup: &PgPool, client_pk: Uuid) -> (Transaction<'_, Postgres>, i32) {
    let mut transaction = setup
        .begin()
        .await
        .expect("begin TEST controller transaction");
    let backend_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *transaction)
        .await
        .expect("controller identifies its TEST backend");
    sqlx::query("SELECT id FROM api_clients WHERE id=$1 FOR UPDATE")
        .bind(client_pk)
        .execute(&mut *transaction)
        .await
        .expect("controller acquires api_clients lock");
    (transaction, backend_pid)
}

async fn hold_scope_table_lock(
    setup: &PgPool,
    client_pk: Uuid,
) -> (Transaction<'_, Postgres>, i32) {
    let mut transaction = setup
        .begin()
        .await
        .expect("begin TEST scope controller transaction");
    let backend_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *transaction)
        .await
        .expect("scope controller identifies its TEST backend");
    sqlx::query("LOCK TABLE api_client_allowed_scopes IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *transaction)
        .await
        .expect("controller acquires scope table lock");
    sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1")
        .bind(client_pk)
        .execute(&mut *transaction)
        .await
        .expect("controller stages scope removal");
    (transaction, backend_pid)
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

async fn fixture(setup: &PgPool, scopes: &[ApiClientScope], status: &str) -> Fixture {
    let actor_id: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash) VALUES ($1,'external-auth-concurrency') RETURNING id",
    )
    .bind(format!("external-auth-concurrency-{}@test.invalid", Uuid::new_v4()))
    .fetch_one(setup)
    .await
    .unwrap();
    let client_id = unique_client_id();
    let client_pk: Uuid = sqlx::query_scalar(
        "INSERT INTO api_clients (
             client_id,display_name,description,status,
             created_by_staff_user_id,updated_by_staff_user_id
         ) VALUES ($1,'External auth concurrency',NULL,$2,$3,$3) RETURNING id",
    )
    .bind(&client_id)
    .bind(status)
    .bind(actor_id)
    .fetch_one(setup)
    .await
    .unwrap();
    for scope in scopes {
        sqlx::query(
            "INSERT INTO api_client_allowed_scopes (api_client_id,scope_code,assigned_by_staff_user_id)
             VALUES ($1,$2,$3)",
        )
        .bind(client_pk)
        .bind(scope.as_str())
        .bind(actor_id)
        .execute(setup)
        .await
        .unwrap();
    }
    Fixture {
        client_pk,
        client_id,
        actor_id,
    }
}

async fn cleanup(setup: &PgPool, fixture: &Fixture) {
    sqlx::query("DELETE FROM api_client_management_audit WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(setup)
        .await
        .unwrap();
    sqlx::query(
        "DELETE FROM external_access_tokens WHERE api_client_credential_id IN
             (SELECT id FROM api_client_credentials WHERE api_client_id=$1)",
    )
    .bind(fixture.client_pk)
    .execute(setup)
    .await
    .unwrap();
    sqlx::query("DELETE FROM api_client_credentials WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(setup)
        .await
        .unwrap();
    sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(setup)
        .await
        .unwrap();
    sqlx::query("DELETE FROM api_clients WHERE id=$1")
        .bind(fixture.client_pk)
        .execute(setup)
        .await
        .unwrap();
    sqlx::query("DELETE FROM staff_users WHERE id=$1")
        .bind(fixture.actor_id)
        .execute(setup)
        .await
        .unwrap();
}

async fn issue_initial(
    repository: &SqlxExternalAuthRepository,
    fixture: &Fixture,
    crypto: &HmacExternalCredentialCrypto,
    byte: u8,
) -> (CredentialDigest, Uuid) {
    let secret = PlaintextClientSecret::parse_hex(&hex::encode([byte; 32])).unwrap();
    let digest = crypto.credential_digest(&fixture.client_id, &secret);
    let metadata = repository
        .issue_credential(&fixture.client_id, fixture.actor_id, 1, &digest, Utc::now())
        .await
        .unwrap();
    (digest, metadata.credential_id)
}

async fn issue_token(
    repository: &SqlxExternalAuthRepository,
    fixture: &Fixture,
    digest: &CredentialDigest,
    crypto: &HmacExternalCredentialCrypto,
) -> (AccessTokenHash, chrono::DateTime<Utc>) {
    let token = crypto.generate_access_token();
    let hash = crypto.access_token_hash(&token);
    let now = Utc::now();
    repository
        .issue_access_token(
            &fixture.client_id,
            digest,
            &hash,
            now,
            now + ChronoDuration::minutes(15),
        )
        .await
        .unwrap();
    (hash, now)
}

async fn lifecycle_update(
    runtime: PgPool,
    client_pk: Uuid,
    actor_id: Uuid,
    status: &str,
    reason: CredentialRevocationReason,
    hold: Option<(oneshot::Sender<i32>, oneshot::Receiver<()>)>,
) {
    let mut tx = runtime.begin().await.unwrap();
    let backend_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    sqlx::query("SELECT id FROM api_clients WHERE id=$1 FOR UPDATE")
        .bind(client_pk)
        .execute(&mut *tx)
        .await
        .unwrap();
    if let Some((ready, release)) = hold {
        ready.send(backend_pid).unwrap();
        release.await.unwrap();
    }
    sqlx::query(
        "UPDATE api_clients SET status=$2,version=version+1,updated_by_staff_user_id=$3,updated_at=clock_timestamp() WHERE id=$1",
    )
    .bind(client_pk)
    .bind(status)
    .bind(actor_id)
    .execute(&mut *tx)
    .await
    .unwrap();
    revoke_client_credentials_and_tokens(&mut tx, client_pk, reason, Utc::now())
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

#[tokio::test]
async fn simultaneous_issue_has_one_success() {
    let TestPools {
        setup,
        runtime: _,
        runtime_application_name: _,
    } = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let crypto = Arc::new(HmacExternalCredentialCrypto::from_pepper(pepper()));
    let first_application_name = format!("branch25-task3-issue-a-{}", Uuid::new_v4());
    let second_application_name = format!("branch25-task3-issue-b-{}", Uuid::new_v4());
    let first_runtime = runtime_pool_with_application_name(&first_application_name).await;
    let second_runtime = runtime_pool_with_application_name(&second_application_name).await;
    let first_repository = Arc::new(SqlxExternalAuthRepository::new(first_runtime));
    let second_repository = Arc::new(SqlxExternalAuthRepository::new(second_runtime));
    let (controller, controller_pid) = hold_client_lock(&setup, fixture.client_pk).await;
    let issue = |repository: Arc<SqlxExternalAuthRepository>, byte: u8| {
        let crypto = crypto.clone();
        let fixture_id = fixture.client_id.clone();
        let actor = fixture.actor_id;
        tokio::spawn(async move {
            let secret = PlaintextClientSecret::parse_hex(&hex::encode([byte; 32])).unwrap();
            let digest = crypto.credential_digest(&fixture_id, &secret);
            repository
                .issue_credential(&fixture_id, actor, 1, &digest, Utc::now())
                .await
        })
    };
    let first_task = issue(first_repository, 0xb1);
    wait_for_runtime_lock_wait(&setup, &first_application_name, controller_pid, 1).await;
    let second_task = issue(second_repository, 0xb2);
    wait_for_runtime_lock_wait(&setup, &second_application_name, controller_pid, 1).await;
    controller.commit().await.unwrap();
    let first = first_task.await.unwrap();
    let second = second_task.await.unwrap();
    assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
    assert!(matches!(
        first,
        Ok(_)
            | Err(CredentialAdministrationError::LiveCredential
                | CredentialAdministrationError::VersionConflict)
    ));
    assert!(matches!(
        second,
        Ok(_)
            | Err(CredentialAdministrationError::LiveCredential
                | CredentialAdministrationError::VersionConflict)
    ));
    let live: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_client_credentials WHERE api_client_id=$1 AND revoked_at IS NULL",
    )
    .bind(fixture.client_pk)
    .fetch_one(&setup)
    .await
    .unwrap();
    assert_eq!(live, 1);
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn token_issue_serializes_after_suspend() {
    let TestPools {
        setup,
        runtime,
        runtime_application_name,
    } = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let crypto = HmacExternalCredentialCrypto::from_pepper(pepper());
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    let (digest, _) = issue_initial(&repository, &fixture, &crypto, 0xc1).await;
    let (_hash, _) = issue_token(&repository, &fixture, &digest, &crypto).await;
    let (ready_tx, ready_rx) = oneshot::channel::<i32>();
    let (release_tx, release_rx) = oneshot::channel();
    let lifecycle = tokio::spawn(lifecycle_update(
        runtime.clone(),
        fixture.client_pk,
        fixture.actor_id,
        "SUSPENDED",
        CredentialRevocationReason::ClientSuspended,
        Some((ready_tx, release_rx)),
    ));
    let lifecycle_pid = ready_rx.await.unwrap();
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    let crypto = HmacExternalCredentialCrypto::from_pepper(pepper());
    let secret = PlaintextClientSecret::parse_hex(&hex::encode([0xc1; 32])).unwrap();
    let digest = crypto.credential_digest(&fixture.client_id, &secret);
    let token = crypto.generate_access_token();
    let hash = crypto.access_token_hash(&token);
    let now = Utc::now();
    let client_id = fixture.client_id.clone();
    let exchange = tokio::spawn(async move {
        repository
            .issue_access_token(
                &client_id,
                &digest,
                &hash,
                now,
                now + ChronoDuration::minutes(15),
            )
            .await
    });
    wait_for_runtime_lock_wait(&setup, &runtime_application_name, lifecycle_pid, 1).await;
    release_tx.send(()).unwrap();
    lifecycle.await.unwrap();
    assert_eq!(
        exchange.await.unwrap(),
        Err(ExternalTokenExchangeError::InvalidCredential)
    );
    let usable: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM external_access_tokens token
         JOIN api_client_credentials credential ON credential.id=token.api_client_credential_id
         WHERE credential.api_client_id=$1 AND token.revoked_at IS NULL",
    )
    .bind(fixture.client_pk)
    .fetch_one(&setup)
    .await
    .unwrap();
    assert_eq!(usable, 0);
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn token_issue_serializes_after_client_revoke() {
    let TestPools {
        setup,
        runtime,
        runtime_application_name,
    } = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let crypto = HmacExternalCredentialCrypto::from_pepper(pepper());
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    let (digest, _) = issue_initial(&repository, &fixture, &crypto, 0xd1).await;
    let (ready_tx, ready_rx) = oneshot::channel::<i32>();
    let (release_tx, release_rx) = oneshot::channel();
    let lifecycle = tokio::spawn(lifecycle_update(
        runtime.clone(),
        fixture.client_pk,
        fixture.actor_id,
        "REVOKED",
        CredentialRevocationReason::ClientRevoked,
        Some((ready_tx, release_rx)),
    ));
    let lifecycle_pid = ready_rx.await.unwrap();
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    let token = crypto.generate_access_token();
    let hash = crypto.access_token_hash(&token);
    let now = Utc::now();
    let client_id = fixture.client_id.clone();
    let exchange = tokio::spawn(async move {
        repository
            .issue_access_token(
                &client_id,
                &digest,
                &hash,
                now,
                now + ChronoDuration::minutes(15),
            )
            .await
    });
    wait_for_runtime_lock_wait(&setup, &runtime_application_name, lifecycle_pid, 1).await;
    release_tx.send(()).unwrap();
    lifecycle.await.unwrap();
    assert_eq!(
        exchange.await.unwrap(),
        Err(ExternalTokenExchangeError::InvalidCredential)
    );
    let live: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_client_credentials WHERE api_client_id=$1 AND revoked_at IS NULL",
    )
    .bind(fixture.client_pk)
    .fetch_one(&setup)
    .await
    .unwrap();
    assert_eq!(live, 0);
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn token_issue_serializes_after_credential_revoke() {
    let TestPools {
        setup,
        runtime,
        runtime_application_name,
    } = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let crypto = HmacExternalCredentialCrypto::from_pepper(pepper());
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    let (digest, _) = issue_initial(&repository, &fixture, &crypto, 0xe1).await;
    let (ready_tx, ready_rx) = oneshot::channel::<i32>();
    let (release_tx, release_rx) = oneshot::channel();
    let revoke_runtime = runtime.clone();
    let revoke_client_pk = fixture.client_pk;
    let revoke = tokio::spawn(async move {
        let mut tx = revoke_runtime.begin().await.unwrap();
        sqlx::query("SELECT id FROM api_clients WHERE id=$1 FOR UPDATE")
            .bind(revoke_client_pk)
            .execute(&mut *tx)
            .await
            .unwrap();
        let backend_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        ready_tx.send(backend_pid).unwrap();
        release_rx.await.unwrap();
        revoke_client_credentials_and_tokens(
            &mut tx,
            revoke_client_pk,
            CredentialRevocationReason::AdminRequest,
            Utc::now(),
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
    });
    let lifecycle_pid = ready_rx.await.unwrap();
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    let token = crypto.generate_access_token();
    let hash = crypto.access_token_hash(&token);
    let now = Utc::now();
    let client_id = fixture.client_id.clone();
    let exchange = tokio::spawn(async move {
        repository
            .issue_access_token(
                &client_id,
                &digest,
                &hash,
                now,
                now + ChronoDuration::minutes(15),
            )
            .await
    });
    wait_for_runtime_lock_wait(&setup, &runtime_application_name, lifecycle_pid, 1).await;
    release_tx.send(()).unwrap();
    revoke.await.unwrap();
    assert_eq!(
        exchange.await.unwrap(),
        Err(ExternalTokenExchangeError::InvalidCredential)
    );
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn credential_issue_vs_suspend_has_one_final_credential_and_no_usable_token() {
    let TestPools {
        setup,
        runtime,
        runtime_application_name,
    } = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let crypto = HmacExternalCredentialCrypto::from_pepper(pepper());
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    let secret = PlaintextClientSecret::parse_hex(&hex::encode([0xf1; 32])).unwrap();
    let digest = crypto.credential_digest(&fixture.client_id, &secret);
    let (ready_tx, ready_rx) = oneshot::channel::<i32>();
    let (release_tx, release_rx) = oneshot::channel();
    let suspend_runtime = runtime.clone();
    let suspend_client_pk = fixture.client_pk;
    let suspend_actor = fixture.actor_id;
    let suspend = tokio::spawn(async move {
        lifecycle_update(
            suspend_runtime,
            suspend_client_pk,
            suspend_actor,
            "SUSPENDED",
            CredentialRevocationReason::ClientSuspended,
            Some((ready_tx, release_rx)),
        )
        .await;
    });
    let lifecycle_pid = ready_rx.await.unwrap();
    let issue_repository = repository.clone();
    let issue_client_id = fixture.client_id.clone();
    let issue_digest = digest.clone();
    let issue_actor = fixture.actor_id;
    let issue = tokio::spawn(async move {
        issue_repository
            .issue_credential(&issue_client_id, issue_actor, 1, &issue_digest, Utc::now())
            .await
    });
    wait_for_runtime_lock_wait(&setup, &runtime_application_name, lifecycle_pid, 1).await;
    release_tx.send(()).unwrap();
    suspend.await.unwrap();
    assert!(matches!(
        issue.await.unwrap(),
        Err(CredentialAdministrationError::VersionConflict)
    ));
    let status: String = sqlx::query_scalar("SELECT status FROM api_clients WHERE id=$1")
        .bind(fixture.client_pk)
        .fetch_one(&setup)
        .await
        .unwrap();
    assert_eq!(status, "SUSPENDED");
    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM api_client_credentials WHERE api_client_id=$1")
            .bind(fixture.client_pk)
            .fetch_one(&setup)
            .await
            .unwrap();
    let live: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_client_credentials WHERE api_client_id=$1 AND revoked_at IS NULL",
    )
    .bind(fixture.client_pk)
    .fetch_one(&setup)
    .await
    .unwrap();
    let tokens: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM external_access_tokens token
         JOIN api_client_credentials credential ON credential.id=token.api_client_credential_id
         WHERE credential.api_client_id=$1 AND token.revoked_at IS NULL",
    )
    .bind(fixture.client_pk)
    .fetch_one(&setup)
    .await
    .unwrap();
    assert!(total <= 1);
    assert_eq!(live, 0);
    assert_eq!(tokens, 0);

    let issued_while_suspended = repository
        .issue_credential(&fixture.client_id, fixture.actor_id, 2, &digest, Utc::now())
        .await
        .unwrap();
    assert_ne!(issued_while_suspended.credential_id, Uuid::nil());
    let token = crypto.generate_access_token();
    let token_hash = crypto.access_token_hash(&token);
    let now = Utc::now();
    assert_eq!(
        repository
            .issue_access_token(
                &fixture.client_id,
                &digest,
                &token_hash,
                now,
                now + ChronoDuration::minutes(15),
            )
            .await,
        Err(ExternalTokenExchangeError::InvalidCredential)
    );
    let live_after_reissue: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_client_credentials WHERE api_client_id=$1 AND revoked_at IS NULL",
    )
    .bind(fixture.client_pk)
    .fetch_one(&setup)
    .await
    .unwrap();
    assert_eq!(live_after_reissue, 1);
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn credential_issue_then_suspend_revokes_credential_and_tokens() {
    let TestPools {
        setup,
        runtime: _,
        runtime_application_name: _,
    } = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let crypto = HmacExternalCredentialCrypto::from_pepper(pepper());
    let issue_application_name = format!("branch25-task3-issue-{}", Uuid::new_v4());
    let suspend_application_name = format!("branch25-task3-suspend-{}", Uuid::new_v4());
    let issue_runtime = runtime_pool_with_application_name(&issue_application_name).await;
    let suspend_runtime = runtime_pool_with_application_name(&suspend_application_name).await;
    let repository = Arc::new(SqlxExternalAuthRepository::new(issue_runtime));
    let secret = PlaintextClientSecret::parse_hex(&hex::encode([0xf2; 32])).unwrap();
    let digest = crypto.credential_digest(&fixture.client_id, &secret);
    let (controller, controller_pid) = hold_client_lock(&setup, fixture.client_pk).await;
    let issue_repository = repository.clone();
    let issue_client_id = fixture.client_id.clone();
    let issue_digest = digest.clone();
    let issue = tokio::spawn(async move {
        issue_repository
            .issue_credential(
                &issue_client_id,
                fixture.actor_id,
                1,
                &issue_digest,
                Utc::now(),
            )
            .await
    });
    wait_for_runtime_lock_wait(&setup, &issue_application_name, controller_pid, 1).await;
    let suspend = tokio::spawn(lifecycle_update(
        suspend_runtime,
        fixture.client_pk,
        fixture.actor_id,
        "SUSPENDED",
        CredentialRevocationReason::ClientSuspended,
        None,
    ));
    wait_for_runtime_lock_wait(&setup, &suspend_application_name, controller_pid, 1).await;
    controller.commit().await.unwrap();
    assert!(issue.await.unwrap().is_ok());
    suspend.await.unwrap();

    let live: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_client_credentials WHERE api_client_id=$1 AND revoked_at IS NULL",
    )
    .bind(fixture.client_pk)
    .fetch_one(&setup)
    .await
    .unwrap();
    let tokens: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM external_access_tokens token
         JOIN api_client_credentials credential ON credential.id=token.api_client_credential_id
         WHERE credential.api_client_id=$1 AND token.revoked_at IS NULL",
    )
    .bind(fixture.client_pk)
    .fetch_one(&setup)
    .await
    .unwrap();
    assert_eq!(live, 0);
    assert_eq!(tokens, 0);
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn credential_issue_vs_client_revoke_has_no_live_credential_or_token() {
    let TestPools {
        setup,
        runtime: _,
        runtime_application_name: _,
    } = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let crypto = Arc::new(HmacExternalCredentialCrypto::from_pepper(pepper()));
    let issue_application_name = format!("branch25-task3-issue-{}", Uuid::new_v4());
    let revoke_application_name = format!("branch25-task3-revoke-{}", Uuid::new_v4());
    let issue_runtime = runtime_pool_with_application_name(&issue_application_name).await;
    let revoke_runtime = runtime_pool_with_application_name(&revoke_application_name).await;
    let repository = Arc::new(SqlxExternalAuthRepository::new(issue_runtime));
    let secret = PlaintextClientSecret::parse_hex(&hex::encode([0x12; 32])).unwrap();
    let digest = crypto.credential_digest(&fixture.client_id, &secret);
    let (controller, controller_pid) = hold_client_lock(&setup, fixture.client_pk).await;
    let issue_repository = repository.clone();
    let issue_client_id = fixture.client_id.clone();
    let issue_digest = digest.clone();
    let issue_actor = fixture.actor_id;
    let issue = tokio::spawn(async move {
        issue_repository
            .issue_credential(&issue_client_id, issue_actor, 1, &issue_digest, Utc::now())
            .await
    });
    wait_for_runtime_lock_wait(&setup, &issue_application_name, controller_pid, 1).await;
    let revoke_client_pk = fixture.client_pk;
    let revoke_actor = fixture.actor_id;
    let revoke = tokio::spawn(async move {
        lifecycle_update(
            revoke_runtime,
            revoke_client_pk,
            revoke_actor,
            "REVOKED",
            CredentialRevocationReason::ClientRevoked,
            None,
        )
        .await;
    });
    wait_for_runtime_lock_wait(&setup, &revoke_application_name, controller_pid, 1).await;
    controller.commit().await.unwrap();
    let result = issue.await.unwrap();
    revoke.await.unwrap();
    assert!(matches!(
        result,
        Ok(_)
            | Err(CredentialAdministrationError::VersionConflict
                | CredentialAdministrationError::RevokedClient)
    ));
    let status: String = sqlx::query_scalar("SELECT status FROM api_clients WHERE id=$1")
        .bind(fixture.client_pk)
        .fetch_one(&setup)
        .await
        .unwrap();
    assert_eq!(status, "REVOKED");
    let live: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_client_credentials WHERE api_client_id=$1 AND revoked_at IS NULL",
    )
    .bind(fixture.client_pk)
    .fetch_one(&setup)
    .await
    .unwrap();
    assert_eq!(live, 0);
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn credential_issue_vs_credential_revoke_allows_replacement_after_revoke() {
    let TestPools {
        setup,
        runtime: _,
        runtime_application_name: _,
    } = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let crypto = Arc::new(HmacExternalCredentialCrypto::from_pepper(pepper()));
    let issue_application_name = format!("branch25-task3-issue-{}", Uuid::new_v4());
    let revoke_application_name = format!("branch25-task3-revoke-{}", Uuid::new_v4());
    let issue_runtime = runtime_pool_with_application_name(&issue_application_name).await;
    let revoke_runtime = runtime_pool_with_application_name(&revoke_application_name).await;
    let repository = Arc::new(SqlxExternalAuthRepository::new(issue_runtime));
    let old = PlaintextClientSecret::parse_hex(&hex::encode([0x21; 32])).unwrap();
    let old_digest = crypto.credential_digest(&fixture.client_id, &old);
    let first = repository
        .issue_credential(
            &fixture.client_id,
            fixture.actor_id,
            1,
            &old_digest,
            Utc::now(),
        )
        .await
        .unwrap();
    let replacement = PlaintextClientSecret::parse_hex(&hex::encode([0x22; 32])).unwrap();
    let replacement_digest = crypto.credential_digest(&fixture.client_id, &replacement);
    let (controller, controller_pid) = hold_client_lock(&setup, fixture.client_pk).await;
    let issue_repository = repository.clone();
    let issue_client_id = fixture.client_id.clone();
    let issue_digest = replacement_digest.clone();
    let issue_actor = fixture.actor_id;
    let issue = tokio::spawn(async move {
        issue_repository
            .issue_credential(&issue_client_id, issue_actor, 2, &issue_digest, Utc::now())
            .await
    });
    wait_for_runtime_lock_wait(&setup, &issue_application_name, controller_pid, 1).await;
    let revoke_repository = Arc::new(SqlxExternalAuthRepository::new(revoke_runtime));
    let revoke_client_id = fixture.client_id.clone();
    let revoke_actor = fixture.actor_id;
    let revoke = tokio::spawn(async move {
        revoke_repository
            .revoke_credential(
                &revoke_client_id,
                revoke_actor,
                2,
                CredentialRevocationReason::Replaced,
                Utc::now(),
            )
            .await
    });
    wait_for_runtime_lock_wait(&setup, &revoke_application_name, controller_pid, 1).await;
    controller.commit().await.unwrap();
    let issue_result = issue.await.unwrap();
    let revoke_result = revoke.await.unwrap();
    assert!(matches!(
        issue_result,
        Err(CredentialAdministrationError::LiveCredential
            | CredentialAdministrationError::VersionConflict)
    ));
    revoke_result.unwrap();
    let second = repository
        .issue_credential(
            &fixture.client_id,
            fixture.actor_id,
            3,
            &replacement_digest,
            Utc::now(),
        )
        .await
        .unwrap();
    assert_ne!(first.credential_id, second.credential_id);
    let live: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_client_credentials WHERE api_client_id=$1 AND revoked_at IS NULL",
    )
    .bind(fixture.client_pk)
    .fetch_one(&setup)
    .await
    .unwrap();
    assert_eq!(live, 1);
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn scope_removal_before_auth_is_denied() {
    let TestPools {
        setup,
        runtime,
        runtime_application_name,
    } = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let crypto = HmacExternalCredentialCrypto::from_pepper(pepper());
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    let (digest, _) = issue_initial(&repository, &fixture, &crypto, 0x31).await;
    let (hash, now) = issue_token(&repository, &fixture, &digest, &crypto).await;
    let (controller, controller_pid) = hold_scope_table_lock(&setup, fixture.client_pk).await;
    let auth_repository = SqlxExternalAuthRepository::new(runtime.clone());
    let auth = tokio::spawn(async move {
        auth_repository
            .authenticate_access_token(&hash, now + ChronoDuration::seconds(1))
            .await
    });
    wait_for_runtime_lock_wait(&setup, &runtime_application_name, controller_pid, 1).await;
    controller.commit().await.unwrap();
    let principal = auth.await.unwrap().unwrap();
    assert!(principal.scopes().is_empty());
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn scope_removal_after_guard_allows_in_flight() {
    let TestPools {
        setup,
        runtime,
        runtime_application_name: _,
    } = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let crypto = HmacExternalCredentialCrypto::from_pepper(pepper());
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    let (digest, _) = issue_initial(&repository, &fixture, &crypto, 0x41).await;
    let (hash, now) = issue_token(&repository, &fixture, &digest, &crypto).await;
    let (authenticated_tx, authenticated_rx) = oneshot::channel();
    let (finish_tx, finish_rx) = oneshot::channel();
    let in_flight = tokio::spawn(async move {
        let principal = repository
            .authenticate_access_token(&hash, now + ChronoDuration::seconds(1))
            .await
            .unwrap();
        assert!(principal.scopes().contains(&ApiClientScope::FlightsRead));
        authenticated_tx.send(()).unwrap();
        finish_rx.await.unwrap();
        principal
    });
    authenticated_rx.await.unwrap();
    sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(&setup)
        .await
        .unwrap();
    finish_tx.send(()).unwrap();
    let principal = in_flight.await.unwrap();
    assert!(principal.scopes().contains(&ApiClientScope::FlightsRead));
    cleanup(&setup, &fixture).await;
}

#[allow(dead_code)]
const _CONCURRENCY_TEST_TIMEOUT: Duration = Duration::from_secs(10);
