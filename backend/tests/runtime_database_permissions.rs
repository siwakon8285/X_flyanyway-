mod common;

use std::{future::Future, sync::Arc, time::Duration};

use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Algorithm, Argon2, Params, Version,
};
use chrono::{NaiveDate, Utc};
use rand::RngCore;
use sqlx::{PgPool, Row};
use x_fly_api::{
    application::{api_client::ApiClientRepository, staff_auth::StaffAuthService},
    domain::{
        entities::{CreateSeatHold, FlightSelection},
        repositories::SeatHoldRepository,
        value_objects::{CabinClass, PassengerCounts, SeatNumber},
    },
    infrastructure::database::{
        migrate_database, prepare_test_database, verify_database_ready, SqlxApiClientRepository,
        SqlxSeatHoldRepository, SqlxStaffAuthRepository,
    },
    infrastructure::password::Argon2PasswordService,
};

#[derive(Clone)]
struct AuthFixture {
    client_id: uuid::Uuid,
    public_client_id: String,
    staff_id: uuid::Uuid,
}

fn assert_constraint_error(error: sqlx::Error, code: &str, constraint: &str) {
    let database_error = error
        .as_database_error()
        .expect("expected a PostgreSQL constraint error");
    assert!(database_error.code().is_some_and(|value| value == code));
    assert_eq!(database_error.constraint(), Some(constraint));
}

async fn permission_pools() -> (PgPool, PgPool) {
    let setup_pool = PgPool::connect(&common::test_database_url()).await.unwrap();
    migrate_database(&setup_pool).await.unwrap();
    let runtime_pool = PgPool::connect(&common::test_runtime_database_url())
        .await
        .unwrap();
    verify_database_ready(&runtime_pool).await.unwrap();
    (setup_pool, runtime_pool)
}

fn fixture_public_client_id() -> String {
    let suffix: String = uuid::Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(16)
        .map(|value| match value {
            '0' => '2',
            '1' => '3',
            value => value.to_ascii_uppercase(),
        })
        .collect();
    format!("XFC{suffix}")
}

async fn create_auth_fixture(setup_pool: &PgPool) -> AuthFixture {
    let mut transaction = setup_pool.begin().await.unwrap();
    let staff_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email, password_hash)
         VALUES ($1, 'task2-permission-fixture') RETURNING id",
    )
    .bind(format!(
        "task2-permission-{}@x-fly.test",
        uuid::Uuid::new_v4().simple()
    ))
    .fetch_one(&mut *transaction)
    .await
    .unwrap();
    let public_client_id = fixture_public_client_id();
    let client_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO api_clients (
             client_id, display_name, description, status,
             created_by_staff_user_id, updated_by_staff_user_id
         ) VALUES ($1, 'Task 2 permission fixture', NULL, 'ACTIVE', $2, $2)
         RETURNING id",
    )
    .bind(&public_client_id)
    .bind(staff_id)
    .fetch_one(&mut *transaction)
    .await
    .unwrap();
    transaction.commit().await.unwrap();
    AuthFixture {
        client_id,
        public_client_id,
        staff_id,
    }
}

async fn cleanup_auth_fixture(
    setup_pool: &PgPool,
    fixture: &AuthFixture,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM api_client_management_audit WHERE api_client_id = $1")
        .bind(fixture.client_id)
        .execute(setup_pool)
        .await?;
    sqlx::query(
        "DELETE FROM external_access_tokens
         WHERE api_client_credential_id IN (
             SELECT id FROM api_client_credentials WHERE api_client_id = $1
         )",
    )
    .bind(fixture.client_id)
    .execute(setup_pool)
    .await?;
    sqlx::query("DELETE FROM api_client_credentials WHERE api_client_id = $1")
        .bind(fixture.client_id)
        .execute(setup_pool)
        .await?;
    sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id = $1")
        .bind(fixture.client_id)
        .execute(setup_pool)
        .await?;
    sqlx::query("DELETE FROM api_clients WHERE id = $1")
        .bind(fixture.client_id)
        .execute(setup_pool)
        .await?;
    sqlx::query("DELETE FROM staff_users WHERE id = $1")
        .bind(fixture.staff_id)
        .execute(setup_pool)
        .await?;
    Ok(())
}

async fn run_auth_fixture_body<F, Fut>(setup_pool: &PgPool, fixtures: Vec<AuthFixture>, body: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let cleanup_pool = setup_pool.clone();
    common::run_fixture_body_with_cleanup(body, move || async move {
        let mut errors = Vec::new();
        for fixture in fixtures {
            if let Err(error) = cleanup_auth_fixture(&cleanup_pool, &fixture).await {
                errors.push(error.to_string());
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    })
    .await;
}

async fn cleanup_staff_actor(setup_pool: &PgPool, staff_id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM staff_security_audit WHERE actor_staff_user_id=$1")
        .bind(staff_id)
        .execute(setup_pool)
        .await?;
    sqlx::query("DELETE FROM staff_sessions WHERE staff_user_id=$1")
        .bind(staff_id)
        .execute(setup_pool)
        .await?;
    sqlx::query("DELETE FROM staff_user_roles WHERE staff_user_id=$1")
        .bind(staff_id)
        .execute(setup_pool)
        .await?;
    sqlx::query("DELETE FROM staff_users WHERE id=$1")
        .bind(staff_id)
        .execute(setup_pool)
        .await?;
    Ok(())
}

async fn run_staff_actor_body<F, Fut>(setup_pool: &PgPool, staff_id: uuid::Uuid, body: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let cleanup_pool = setup_pool.clone();
    common::run_fixture_body_with_cleanup(body, move || async move {
        cleanup_staff_actor(&cleanup_pool, staff_id).await
    })
    .await;
}

async fn cleanup_runtime_seat_hold(
    setup_pool: &PgPool,
    state: &Arc<tokio::sync::Mutex<Option<(uuid::Uuid, NaiveDate)>>>,
) -> Result<(), sqlx::Error> {
    let Some((hold_id, departure_date)) = *state.lock().await else {
        return Ok(());
    };

    sqlx::query("DELETE FROM seat_holds WHERE id = $1")
        .bind(hold_id)
        .execute(setup_pool)
        .await?;
    sqlx::query(
        "DELETE FROM flight_instances
         WHERE departure_date = $1
           AND flight_service_id = (SELECT id FROM flight_services WHERE public_id = 'xf-201')",
    )
    .bind(departure_date)
    .execute(setup_pool)
    .await?;
    state.lock().await.take();
    Ok(())
}

async fn insert_setup_credential(setup_pool: &PgPool, fixture: &AuthFixture) -> uuid::Uuid {
    let mut transaction = setup_pool.begin().await.unwrap();
    let credential_id = sqlx::query_scalar(
        "INSERT INTO api_client_credentials (
             api_client_id, secret_digest, digest_version, issued_at,
             issued_by_staff_user_id
         ) VALUES ($1, $2, 1, $3, $4)
         RETURNING id",
    )
    .bind(fixture.client_id)
    .bind(vec![0x11_u8; 32])
    .bind(Utc::now())
    .bind(fixture.staff_id)
    .fetch_one(&mut *transaction)
    .await
    .unwrap();
    transaction.commit().await.unwrap();
    credential_id
}

fn fresh_test_token_hash() -> [u8; 32] {
    let mut token_hash = [0_u8; 32];
    rand::rng().fill_bytes(&mut token_hash);
    token_hash
}

async fn insert_setup_token(setup_pool: &PgPool, credential_id: uuid::Uuid) -> uuid::Uuid {
    let mut transaction = setup_pool.begin().await.unwrap();
    let token_id = sqlx::query_scalar(
        "INSERT INTO external_access_tokens (
             api_client_credential_id, token_hash, issued_at, expires_at
         ) VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(credential_id)
    .bind(fresh_test_token_hash().to_vec())
    .bind(Utc::now())
    .bind(Utc::now() + chrono::Duration::minutes(15))
    .fetch_one(&mut *transaction)
    .await
    .unwrap();
    transaction.commit().await.unwrap();
    token_id
}

async fn permission_pools_only() -> (PgPool, PgPool) {
    permission_pools().await
}

#[tokio::test]
async fn runtime_can_load_api_client_detail_with_credential_metadata() {
    let (setup_pool, runtime_pool) = permission_pools().await;
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let fixture = create_auth_fixture(&setup_pool).await;
    let fixture_for_body = fixture.clone();
    run_auth_fixture_body(&setup_pool, vec![fixture], move || async move {
        let repository = SqlxApiClientRepository::new(runtime_pool);
        let detail = repository
            .detail(&fixture_for_body.public_client_id)
            .await
            .expect("runtime can read the detail credential metadata query");
        assert_eq!(detail.client.client_id, fixture_for_body.public_client_id);
        assert!(!detail.credential_metadata.has_live_credential);
    })
    .await;
}

async fn table_privilege(pool: &PgPool, role: &str, table: &str, privilege: &str) -> bool {
    sqlx::query_scalar("SELECT has_table_privilege($1, $2::regclass, $3)")
        .bind(role)
        .bind(table)
        .bind(privilege)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn column_privilege(
    pool: &PgPool,
    role: &str,
    table: &str,
    column: &str,
    privilege: &str,
) -> bool {
    sqlx::query_scalar("SELECT has_column_privilege($1, $2::regclass, $3, $4)")
        .bind(role)
        .bind(table)
        .bind(column)
        .bind(privilege)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn runtime_role_has_required_positive_and_negative_permissions() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let setup_url = common::test_database_url();
    let runtime_url = common::test_runtime_database_url();
    let setup_pool = PgPool::connect(&setup_url).await.unwrap();
    prepare_test_database(&setup_pool).await.unwrap();

    let runtime_pool = PgPool::connect(&runtime_url).await.unwrap();
    verify_database_ready(&runtime_pool).await.unwrap();

    let role = sqlx::query(
        "SELECT rolsuper, rolcreatedb, rolcreaterole, rolreplication, rolbypassrls, rolinherit
         FROM pg_roles WHERE rolname = current_user",
    )
    .fetch_one(&runtime_pool)
    .await
    .unwrap();
    assert!(!role.get::<bool, _>("rolsuper"));
    assert!(!role.get::<bool, _>("rolcreatedb"));
    assert!(!role.get::<bool, _>("rolcreaterole"));
    assert!(!role.get::<bool, _>("rolreplication"));
    assert!(!role.get::<bool, _>("rolbypassrls"));
    assert!(!role.get::<bool, _>("rolinherit"));

    let owned_application_objects: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM pg_class object
         JOIN pg_namespace namespace ON namespace.oid = object.relnamespace
         WHERE namespace.nspname = 'public'
           AND object.relkind IN ('r', 'p', 'S', 'v', 'm', 'f')
           AND object.relowner = (SELECT oid FROM pg_roles WHERE rolname = current_user)",
    )
    .fetch_one(&runtime_pool)
    .await
    .unwrap();
    assert_eq!(owned_application_objects, 0);

    let schema_owned: bool = sqlx::query_scalar(
        "SELECT namespace.nspowner = (SELECT oid FROM pg_roles WHERE rolname = current_user)
         FROM pg_namespace namespace WHERE namespace.nspname = 'public'",
    )
    .fetch_one(&runtime_pool)
    .await
    .unwrap();
    assert!(!schema_owned);
    let owned_functions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pg_proc function
         JOIN pg_namespace namespace ON namespace.oid = function.pronamespace
         WHERE namespace.nspname = 'public'
           AND function.proowner = (SELECT oid FROM pg_roles WHERE rolname = current_user)",
    )
    .fetch_one(&runtime_pool)
    .await
    .unwrap();
    assert_eq!(owned_functions, 0);

    let mut transaction = runtime_pool.begin().await.unwrap();
    let service_id: uuid::Uuid = sqlx::query_scalar(
        "SELECT id FROM flight_services WHERE status = 'SCHEDULED'
         ORDER BY flight_number LIMIT 1 FOR UPDATE",
    )
    .fetch_one(&mut *transaction)
    .await
    .unwrap();
    let updated: uuid::Uuid = sqlx::query_scalar(
        "UPDATE flight_services SET updated_at = updated_at WHERE id = $1 RETURNING id",
    )
    .bind(service_id)
    .fetch_one(&mut *transaction)
    .await
    .unwrap();
    assert_eq!(updated, service_id);

    let throttle_hash = vec![0x5a; 32];
    let upserted_count: i32 = sqlx::query_scalar(
        "INSERT INTO staff_login_throttles
             (identifier_hash, failure_count, window_started_at, updated_at)
         VALUES ($1, 1, NOW(), NOW())
         ON CONFLICT (identifier_hash) DO UPDATE
         SET failure_count = staff_login_throttles.failure_count + 1, updated_at = NOW()
         RETURNING failure_count",
    )
    .bind(&throttle_hash)
    .fetch_one(&mut *transaction)
    .await
    .unwrap();
    assert!(upserted_count >= 1);
    let deleted_hash: Vec<u8> = sqlx::query_scalar(
        "DELETE FROM staff_login_throttles WHERE identifier_hash = $1 RETURNING identifier_hash",
    )
    .bind(&throttle_hash)
    .fetch_one(&mut *transaction)
    .await
    .unwrap();
    assert_eq!(deleted_hash, throttle_hash);

    let audit_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO flight_management_audit
             (actor_staff_user_id, actor_email, flight_service_id, action, after_state)
         VALUES ($1, 'runtime-permission-test@invalid', $2, 'FLIGHT_EDITED', '{}'::jsonb)
         RETURNING id",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(service_id)
    .fetch_one(&mut *transaction)
    .await
    .unwrap();
    assert_ne!(audit_id, uuid::Uuid::nil());

    let function_result: bool =
        sqlx::query_scalar("SELECT has_protected_stripe_card_finalization(gen_random_uuid())")
            .fetch_one(&mut *transaction)
            .await
            .unwrap();
    assert!(!function_result);
    transaction.rollback().await.unwrap();

    let hold_state = Arc::new(tokio::sync::Mutex::new(None::<(uuid::Uuid, NaiveDate)>));
    let body_hold_state = hold_state.clone();
    let cleanup_hold_state = hold_state.clone();
    let body_setup_pool = setup_pool.clone();
    let body_runtime_pool = runtime_pool.clone();
    let cleanup_setup_pool = setup_pool.clone();
    common::run_fixture_body_with_cleanup(
        move || async move {
            let earliest = chrono::Utc::now().date_naive() + chrono::Duration::days(30);
            let departure_date = common::allocate_test_departure_date(
                "xf-201",
                earliest,
                earliest + chrono::Duration::days(300),
            )
            .await
            .unwrap();
            let repository = SqlxSeatHoldRepository::new(body_runtime_pool.clone());
            let hold = repository
                .create_hold(
                    CreateSeatHold {
                        selection: FlightSelection {
                            flight_id: "xf-201".to_owned(),
                            departure_date,
                            cabin: CabinClass::Business,
                        },
                        passengers: PassengerCounts::new(1, 0, 0).unwrap(),
                        seats: vec![SeatNumber::parse("3A").unwrap()],
                        token_hash: [0x6b; 32],
                    },
                    Duration::from_secs(600),
                )
                .await
                .unwrap();
            body_hold_state
                .lock()
                .await
                .replace((hold.id, departure_date));
            repository.get_hold(hold.id, [0x6b; 32]).await.unwrap();
            cleanup_runtime_seat_hold(&body_setup_pool, &body_hold_state)
                .await
                .unwrap();

            assert!(
                sqlx::query("CREATE TABLE public.runtime_must_not_create(id integer)")
                    .execute(&body_runtime_pool)
                    .await
                    .is_err()
            );
            assert!(sqlx::query(
                "ALTER TABLE public.flight_services ADD COLUMN runtime_must_not_add integer"
            )
            .execute(&body_runtime_pool)
            .await
            .is_err());
            assert!(sqlx::query("DROP TABLE public.airports")
                .execute(&body_runtime_pool)
                .await
                .is_err());
            assert!(sqlx::query("CREATE ROLE runtime_must_not_create")
                .execute(&body_runtime_pool)
                .await
                .is_err());
            assert!(sqlx::query(
                "INSERT INTO staff_users (email, password_hash) VALUES ('runtime-must-not-provision@x-fly.test', 'nope')",
            )
            .execute(&body_runtime_pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO staff_user_roles (staff_user_id, role_code) VALUES ($1, 'SYSTEM_ADMIN')",
            )
            .bind(uuid::Uuid::new_v4())
            .execute(&body_runtime_pool)
            .await
            .is_err());
            assert!(sqlx::query("ALTER ROLE x_fly_runtime CREATEDB")
                .execute(&body_runtime_pool)
                .await
                .is_err());
            assert!(sqlx::query("CREATE DATABASE runtime_must_not_create")
                .execute(&body_runtime_pool)
                .await
                .is_err());
            assert!(sqlx::query("SET ROLE x_fly_migrator")
                .execute(&body_runtime_pool)
                .await
                .is_err());
            assert!(sqlx::query("UPDATE _sqlx_migrations SET success = success")
                .execute(&body_runtime_pool)
                .await
                .is_err());
        },
        move || async move {
            cleanup_runtime_seat_hold(&cleanup_setup_pool, &cleanup_hold_state).await
        },
    )
    .await;
}

#[tokio::test]
async fn runtime_staff_login_can_rehash_without_provisioning_privileges() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let setup_url = common::test_database_url();
    let setup_pool = PgPool::connect(&setup_url).await.unwrap();
    prepare_test_database(&setup_pool).await.unwrap();
    let runtime_pool = PgPool::connect(&common::test_runtime_database_url())
        .await
        .unwrap();

    let email = format!(
        "runtime-rehash-{}@x-fly.test",
        uuid::Uuid::new_v4().simple()
    );
    let password = "Runtime rehash passphrase 2026";
    let weak_params = Params::new(4_096, 1, 1, None).unwrap();
    let weak_engine = Argon2::new(Algorithm::Argon2id, Version::V0x13, weak_params);
    let salt = SaltString::encode_b64(b"0123456789abcdef").unwrap();
    let weak_hash = weak_engine
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    let mut setup_transaction = setup_pool.begin().await.unwrap();
    let staff_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email, password_hash) VALUES ($1, $2) RETURNING id",
    )
    .bind(&email)
    .bind(&weak_hash)
    .fetch_one(&mut *setup_transaction)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO staff_user_roles (staff_user_id, role_code) VALUES ($1, 'SYSTEM_ADMIN')",
    )
    .bind(staff_id)
    .execute(&mut *setup_transaction)
    .await
    .unwrap();
    setup_transaction.commit().await.unwrap();
    let setup_for_body = setup_pool.clone();
    run_staff_actor_body(&setup_pool, staff_id, move || async move {
        let auth = StaffAuthService::new(
            Arc::new(SqlxStaffAuthRepository::new(runtime_pool)),
            Argon2PasswordService::default(),
            Duration::from_secs(3_600),
        )
        .unwrap();
        let login = auth.login(&email, password).await.unwrap();
        assert!(!login.token.is_empty());
        let stored_hash: String =
            sqlx::query_scalar("SELECT password_hash FROM staff_users WHERE id=$1")
                .bind(staff_id)
                .fetch_one(&setup_for_body)
                .await
                .unwrap();
        assert_ne!(stored_hash, weak_hash);
        assert!(stored_hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
    })
    .await;
}

#[tokio::test]
async fn runtime_has_no_table_level_audit_insert() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    assert!(
        !table_privilege(
            &runtime_pool,
            "x_fly_runtime",
            "public.api_client_management_audit",
            "INSERT"
        )
        .await
    );
    assert!(
        !table_privilege(
            &runtime_pool,
            "x_fly_runtime",
            "public.staff_security_audit",
            "INSERT"
        )
        .await
    );
}

#[tokio::test]
async fn runtime_staff_security_audit_is_append_only_and_column_scoped() {
    let _fixture_lock = common::acquire_test_fixture_lock_named("x-fly-audit-permissions").await;
    let (setup_pool, runtime_pool) = permission_pools_only().await;
    let table = "public.staff_security_audit";
    for privilege in [
        "SELECT",
        "UPDATE",
        "DELETE",
        "TRUNCATE",
        "REFERENCES",
        "TRIGGER",
    ] {
        assert!(
            !table_privilege(&runtime_pool, "x_fly_runtime", table, privilege).await,
            "runtime must not have {privilege} on staff security audit"
        );
        assert!(
            !table_privilege(&runtime_pool, "public", table, privilege).await,
            "PUBLIC must not have {privilege} on staff security audit"
        );
    }
    for column in [
        "action",
        "actor_staff_user_id",
        "session_id",
        "permission_code",
        "request_id",
    ] {
        assert!(column_privilege(&runtime_pool, "x_fly_runtime", table, column, "INSERT").await);
        assert!(!column_privilege(&runtime_pool, "x_fly_runtime", table, column, "UPDATE").await);
        assert!(!column_privilege(&runtime_pool, "public", table, column, "INSERT").await);
    }
    for column in ["id", "created_at"] {
        assert!(!column_privilege(&runtime_pool, "x_fly_runtime", table, column, "INSERT").await);
        assert!(!column_privilege(&runtime_pool, "public", table, column, "INSERT").await);
    }

    let request_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO staff_security_audit
            (action, actor_staff_user_id, session_id, permission_code, request_id)
         VALUES ('STAFF_AUTHZ_DENIED', $1, $2, 'flights:write', $3)",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(uuid::Uuid::new_v4())
    .bind(request_id)
    .execute(&runtime_pool)
    .await
    .expect("runtime can insert approved security-audit columns");
    let persisted: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM staff_security_audit WHERE request_id = $1")
            .bind(request_id)
            .fetch_one(&setup_pool)
            .await
            .unwrap();
    assert_eq!(persisted, 1);
    sqlx::query("DELETE FROM staff_security_audit WHERE request_id = $1")
        .bind(request_id)
        .execute(&setup_pool)
        .await
        .unwrap();

    let protected_insert = sqlx::query(
        "INSERT INTO staff_security_audit
            (id, action, actor_staff_user_id, session_id, request_id)
         VALUES ($1, 'STAFF_LOGIN_SUCCEEDED', $2, $3, $4)",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(uuid::Uuid::new_v4())
    .bind(uuid::Uuid::new_v4())
    .bind(uuid::Uuid::new_v4())
    .execute(&runtime_pool)
    .await;
    assert!(
        protected_insert.is_err(),
        "runtime must not supply generated id"
    );

    let protected_created_at_insert = sqlx::query(
        "INSERT INTO staff_security_audit
            (action, actor_staff_user_id, session_id, request_id, created_at)
         VALUES ('STAFF_LOGIN_SUCCEEDED', $1, $2, $3, NOW())",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(uuid::Uuid::new_v4())
    .bind(uuid::Uuid::new_v4())
    .execute(&runtime_pool)
    .await;
    assert!(
        protected_created_at_insert.is_err(),
        "runtime must not supply generated created_at"
    );

    for permission in ["\t", "\n"] {
        let malformed_insert = sqlx::query(
            "INSERT INTO staff_security_audit
                (action, actor_staff_user_id, session_id, permission_code, request_id)
             VALUES ('STAFF_AUTHZ_DENIED', $1, $2, $3, $4)",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(uuid::Uuid::new_v4())
        .bind(permission)
        .bind(uuid::Uuid::new_v4())
        .execute(&runtime_pool)
        .await
        .unwrap_err();
        assert_constraint_error(
            malformed_insert,
            "23514",
            "staff_security_audit_action_context_check",
        );
    }
}

#[tokio::test]
async fn runtime_can_insert_each_staff_security_audit_action() {
    let _fixture_lock = common::acquire_test_fixture_lock_named("x-fly-audit-permissions").await;
    let (setup_pool, runtime_pool) = permission_pools_only().await;
    let mut request_ids = Vec::new();

    for (action, permission_code) in [
        ("STAFF_LOGIN_SUCCEEDED", None),
        ("STAFF_SESSION_REVOKED", None),
        ("STAFF_AUTHZ_DENIED", Some("flights:write")),
    ] {
        let request_id = uuid::Uuid::new_v4();
        request_ids.push(request_id);
        sqlx::query(
            "INSERT INTO staff_security_audit
                (action, actor_staff_user_id, session_id, permission_code, request_id)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(action)
        .bind(uuid::Uuid::new_v4())
        .bind(uuid::Uuid::new_v4())
        .bind(permission_code)
        .bind(request_id)
        .execute(&runtime_pool)
        .await
        .expect("runtime can insert each supported security-audit action");
    }

    for request_id in &request_ids {
        let persisted: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM staff_security_audit WHERE request_id = $1")
                .bind(request_id)
                .fetch_one(&setup_pool)
                .await
                .unwrap();
        assert_eq!(persisted, 1);
    }

    sqlx::query("DELETE FROM staff_security_audit WHERE request_id = ANY($1)")
        .bind(&request_ids)
        .execute(&setup_pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn runtime_has_audit_insert_on_approved_columns() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    for column in [
        "api_client_id",
        "actor_staff_user_id",
        "action",
        "before_state",
        "after_state",
        "credential_id",
        "created_at",
    ] {
        assert!(
            column_privilege(
                &runtime_pool,
                "x_fly_runtime",
                "public.api_client_management_audit",
                column,
                "INSERT"
            )
            .await,
            "runtime must be able to insert approved audit column"
        );
    }
}

#[tokio::test]
async fn runtime_can_select_audit_history() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    sqlx::query(
        "SELECT id, api_client_id, actor_staff_user_id, action,
                before_state, after_state, created_at
         FROM api_client_management_audit",
    )
    .fetch_all(&runtime_pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn runtime_has_no_audit_insert_on_protected_columns() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    for column in ["id"] {
        assert!(
            !column_privilege(
                &runtime_pool,
                "x_fly_runtime",
                "public.api_client_management_audit",
                column,
                "INSERT"
            )
            .await
        );
    }
}

#[tokio::test]
async fn public_has_no_audit_write_privilege() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    assert!(
        !table_privilege(
            &runtime_pool,
            "public",
            "public.api_client_management_audit",
            "INSERT"
        )
        .await
    );

    let public_table_write_acl: bool = sqlx::query_scalar(
        "SELECT NOT EXISTS (
             SELECT 1
             FROM aclexplode(COALESCE(relation.relacl,
                                     acldefault('r', relation.relowner))) privilege
             WHERE privilege.grantee = 0
               AND privilege.privilege_type IN ('INSERT', 'UPDATE', 'DELETE', 'TRUNCATE')
         )
         FROM pg_class relation
         WHERE relation.oid = 'public.api_client_management_audit'::regclass",
    )
    .fetch_one(&runtime_pool)
    .await
    .unwrap();
    assert!(public_table_write_acl);

    for column in [
        "id",
        "api_client_id",
        "actor_staff_user_id",
        "action",
        "before_state",
        "after_state",
        "credential_id",
        "created_at",
    ] {
        assert!(
            !column_privilege(
                &runtime_pool,
                "public",
                "public.api_client_management_audit",
                column,
                "INSERT"
            )
            .await
        );
    }
}

#[tokio::test]
async fn public_has_no_auth_table_write_privilege() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    let auth_tables = [
        (
            "public.api_client_credentials",
            [
                "id",
                "api_client_id",
                "secret_digest",
                "digest_version",
                "issued_at",
                "issued_by_staff_user_id",
                "revoked_at",
                "revoked_by_staff_user_id",
                "revocation_reason",
            ]
            .as_slice(),
        ),
        (
            "public.external_access_tokens",
            [
                "id",
                "api_client_credential_id",
                "token_hash",
                "issued_at",
                "expires_at",
                "revoked_at",
            ]
            .as_slice(),
        ),
    ];

    for (table, columns) in auth_tables {
        for privilege in ["INSERT", "UPDATE", "DELETE", "TRUNCATE"] {
            assert!(
                !table_privilege(&runtime_pool, "public", table, privilege).await,
                "PUBLIC must not have auth-table write privilege"
            );
        }
        for column in columns {
            assert!(!column_privilege(&runtime_pool, "public", table, column, "INSERT").await);
            assert!(!column_privilege(&runtime_pool, "public", table, column, "UPDATE").await);
        }
    }
}

#[tokio::test]
async fn runtime_has_exact_auth_table_column_privileges() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    let credentials_table = "public.api_client_credentials";
    let credential_insert = [
        "api_client_id",
        "secret_digest",
        "digest_version",
        "issued_at",
        "issued_by_staff_user_id",
    ];
    let credential_insert_protected = [
        "id",
        "revoked_at",
        "revoked_by_staff_user_id",
        "revocation_reason",
    ];
    let credential_update = [
        "revoked_at",
        "revoked_by_staff_user_id",
        "revocation_reason",
    ];
    let credential_update_protected = [
        "id",
        "api_client_id",
        "secret_digest",
        "digest_version",
        "issued_at",
        "issued_by_staff_user_id",
    ];
    assert!(!table_privilege(&runtime_pool, "x_fly_runtime", credentials_table, "INSERT").await);
    assert!(!table_privilege(&runtime_pool, "x_fly_runtime", credentials_table, "UPDATE").await);
    for column in credential_insert {
        assert!(
            column_privilege(
                &runtime_pool,
                "x_fly_runtime",
                credentials_table,
                column,
                "INSERT"
            )
            .await
        );
    }
    for column in credential_insert_protected {
        assert!(
            !column_privilege(
                &runtime_pool,
                "x_fly_runtime",
                credentials_table,
                column,
                "INSERT"
            )
            .await
        );
    }
    for column in credential_update {
        assert!(
            column_privilege(
                &runtime_pool,
                "x_fly_runtime",
                credentials_table,
                column,
                "UPDATE"
            )
            .await
        );
    }
    for column in credential_update_protected {
        assert!(
            !column_privilege(
                &runtime_pool,
                "x_fly_runtime",
                credentials_table,
                column,
                "UPDATE"
            )
            .await
        );
    }

    let tokens_table = "public.external_access_tokens";
    let token_insert = [
        "api_client_credential_id",
        "token_hash",
        "issued_at",
        "expires_at",
    ];
    let token_insert_protected = ["id", "revoked_at"];
    let token_update_protected = [
        "id",
        "api_client_credential_id",
        "token_hash",
        "issued_at",
        "expires_at",
    ];
    assert!(!table_privilege(&runtime_pool, "x_fly_runtime", tokens_table, "INSERT").await);
    assert!(!table_privilege(&runtime_pool, "x_fly_runtime", tokens_table, "UPDATE").await);
    for column in token_insert {
        assert!(
            column_privilege(
                &runtime_pool,
                "x_fly_runtime",
                tokens_table,
                column,
                "INSERT"
            )
            .await
        );
    }
    for column in token_insert_protected {
        assert!(
            !column_privilege(
                &runtime_pool,
                "x_fly_runtime",
                tokens_table,
                column,
                "INSERT"
            )
            .await
        );
    }
    assert!(
        column_privilege(
            &runtime_pool,
            "x_fly_runtime",
            tokens_table,
            "revoked_at",
            "UPDATE"
        )
        .await
    );
    for column in token_update_protected {
        assert!(
            !column_privilege(
                &runtime_pool,
                "x_fly_runtime",
                tokens_table,
                column,
                "UPDATE"
            )
            .await
        );
    }
}

#[tokio::test]
async fn runtime_can_select_approved_auth_columns() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    assert!(
        !table_privilege(
            &runtime_pool,
            "x_fly_runtime",
            "public.api_client_credentials",
            "INSERT"
        )
        .await
    );
    assert!(
        !table_privilege(
            &runtime_pool,
            "x_fly_runtime",
            "public.external_access_tokens",
            "INSERT"
        )
        .await
    );
    sqlx::query(
        "SELECT id, api_client_id, secret_digest, digest_version, issued_at,
                revoked_at, revocation_reason
         FROM api_client_credentials",
    )
    .fetch_all(&runtime_pool)
    .await
    .unwrap();
    sqlx::query(
        "SELECT id, api_client_credential_id, token_hash, issued_at,
                expires_at, revoked_at
         FROM external_access_tokens",
    )
    .fetch_all(&runtime_pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn runtime_cannot_select_credential_attribution_fields() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    for column in ["issued_by_staff_user_id", "revoked_by_staff_user_id"] {
        let statement = format!("SELECT {column} FROM api_client_credentials");
        assert!(sqlx::query(&statement)
            .fetch_all(&runtime_pool)
            .await
            .is_err());
    }
}

#[tokio::test]
async fn branch24_audit_insert_succeeds_as_runtime() {
    let (setup_pool, runtime_pool) = permission_pools_only().await;
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let fixture = create_auth_fixture(&setup_pool).await;
    let fixture_for_body = fixture.clone();
    run_auth_fixture_body(&setup_pool, vec![fixture], move || async move {
        let audit_id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO api_client_management_audit (
                 api_client_id, actor_staff_user_id, action,
                 before_state, after_state, created_at
             ) VALUES ($1, $2, 'CLIENT_CREATED', NULL, '{}'::jsonb, clock_timestamp())
             RETURNING id",
        )
        .bind(fixture_for_body.client_id)
        .bind(fixture_for_body.staff_id)
        .fetch_one(&runtime_pool)
        .await
        .unwrap();
        assert_ne!(audit_id, uuid::Uuid::nil());
    })
    .await;
}

#[tokio::test]
async fn credential_issued_audit_insert_succeeds_as_runtime() {
    let (setup_pool, runtime_pool) = permission_pools_only().await;
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let fixture = create_auth_fixture(&setup_pool).await;
    let credential_id = insert_setup_credential(&setup_pool, &fixture).await;
    let fixture_for_body = fixture.clone();
    run_auth_fixture_body(&setup_pool, vec![fixture], move || async move {
        let audit_id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO api_client_management_audit (
                 api_client_id, actor_staff_user_id, action,
                 before_state, after_state, credential_id, created_at
             ) VALUES ($1, $2, 'CREDENTIAL_ISSUED', NULL, '{}'::jsonb, $3, clock_timestamp())
             RETURNING id",
        )
        .bind(fixture_for_body.client_id)
        .bind(fixture_for_body.staff_id)
        .bind(credential_id)
        .fetch_one(&runtime_pool)
        .await
        .unwrap();
        assert_ne!(audit_id, uuid::Uuid::nil());
    })
    .await;
}

#[tokio::test]
async fn credential_revoked_audit_insert_succeeds_as_runtime() {
    let (setup_pool, runtime_pool) = permission_pools_only().await;
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let fixture = create_auth_fixture(&setup_pool).await;
    let credential_id = insert_setup_credential(&setup_pool, &fixture).await;
    let fixture_for_body = fixture.clone();
    run_auth_fixture_body(&setup_pool, vec![fixture], move || async move {
        let audit_id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO api_client_management_audit (
                 api_client_id, actor_staff_user_id, action,
                 before_state, after_state, credential_id, created_at
             ) VALUES ($1, $2, 'CREDENTIAL_REVOKED', '{}'::jsonb, '{}'::jsonb, $3, clock_timestamp())
             RETURNING id",
        )
        .bind(fixture_for_body.client_id)
        .bind(fixture_for_body.staff_id)
        .bind(credential_id)
        .fetch_one(&runtime_pool)
        .await
        .unwrap();
        assert_ne!(audit_id, uuid::Uuid::nil());
    })
    .await;
}

#[tokio::test]
async fn audit_credential_context_constraint_rejects_mismatches() {
    let (setup_pool, _runtime_pool) = permission_pools_only().await;
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let fixture = create_auth_fixture(&setup_pool).await;
    let credential_id = insert_setup_credential(&setup_pool, &fixture).await;
    let fixture_for_body = fixture.clone();
    let setup_for_body = setup_pool.clone();
    run_auth_fixture_body(&setup_pool, vec![fixture], move || async move {
        assert!(sqlx::query(
            "INSERT INTO api_client_management_audit (
                 api_client_id, actor_staff_user_id, action,
                 before_state, after_state, created_at
             ) VALUES ($1, $2, 'CREDENTIAL_ISSUED', NULL, '{}'::jsonb, clock_timestamp())",
        )
        .bind(fixture_for_body.client_id)
        .bind(fixture_for_body.staff_id)
        .execute(&setup_for_body)
        .await
        .is_err());

        assert!(sqlx::query(
            "INSERT INTO api_client_management_audit (
                 api_client_id, actor_staff_user_id, action,
                 before_state, after_state, credential_id, created_at
             ) VALUES ($1, $2, 'CLIENT_CREATED', NULL, '{}'::jsonb, $3, clock_timestamp())",
        )
        .bind(fixture_for_body.client_id)
        .bind(fixture_for_body.staff_id)
        .bind(credential_id)
        .execute(&setup_for_body)
        .await
        .is_err());
    })
    .await;
}

#[tokio::test]
async fn runtime_cannot_update_historical_audit_rows() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    assert!(sqlx::query(
        "UPDATE api_client_management_audit
         SET after_state = after_state",
    )
    .execute(&runtime_pool)
    .await
    .is_err());
}

#[tokio::test]
async fn runtime_cannot_delete_audit_history() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    assert!(sqlx::query("DELETE FROM api_client_management_audit")
        .execute(&runtime_pool)
        .await
        .is_err());
}

#[tokio::test]
async fn runtime_cannot_truncate_audit_history() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    assert!(sqlx::query("TRUNCATE api_client_management_audit")
        .execute(&runtime_pool)
        .await
        .is_err());
}

#[tokio::test]
async fn runtime_cannot_alter_or_transfer_audit_table() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    assert!(
        sqlx::query("ALTER TABLE api_client_management_audit OWNER TO x_fly_runtime",)
            .execute(&runtime_pool)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn runtime_can_insert_credential_with_issuance_columns() {
    let (setup_pool, runtime_pool) = permission_pools_only().await;
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let fixture = create_auth_fixture(&setup_pool).await;
    let fixture_for_body = fixture.clone();
    run_auth_fixture_body(&setup_pool, vec![fixture], move || async move {
        let credential_id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO api_client_credentials (
                 api_client_id, secret_digest, digest_version, issued_at,
                 issued_by_staff_user_id
             ) VALUES ($1, $2, 1, $3, $4)
             RETURNING id",
        )
        .bind(fixture_for_body.client_id)
        .bind(vec![0x33_u8; 32])
        .bind(Utc::now())
        .bind(fixture_for_body.staff_id)
        .fetch_one(&runtime_pool)
        .await
        .unwrap();
        assert_ne!(credential_id, uuid::Uuid::nil());
    })
    .await;
}

#[tokio::test]
async fn runtime_can_insert_token_with_issuance_columns() {
    let (setup_pool, runtime_pool) = permission_pools_only().await;
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let fixture = create_auth_fixture(&setup_pool).await;
    let credential_id = insert_setup_credential(&setup_pool, &fixture).await;
    run_auth_fixture_body(&setup_pool, vec![fixture], move || async move {
        let token_id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO external_access_tokens (
                 api_client_credential_id, token_hash, issued_at, expires_at
             ) VALUES ($1, $2, $3, $4)
             RETURNING id",
        )
        .bind(credential_id)
        .bind(fresh_test_token_hash().to_vec())
        .bind(Utc::now())
        .bind(Utc::now() + chrono::Duration::minutes(15))
        .fetch_one(&runtime_pool)
        .await
        .unwrap();
        assert_ne!(token_id, uuid::Uuid::nil());
    })
    .await;
}

#[tokio::test]
async fn independent_token_fixtures_do_not_reuse_unique_hashes() {
    let (setup_pool, _runtime_pool) = permission_pools_only().await;
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let first_fixture = create_auth_fixture(&setup_pool).await;
    let second_fixture = create_auth_fixture(&setup_pool).await;
    let first_for_body = first_fixture.clone();
    let second_for_body = second_fixture.clone();
    let setup_for_body = setup_pool.clone();
    run_auth_fixture_body(
        &setup_pool,
        vec![first_fixture, second_fixture],
        move || async move {
            let first_credential = insert_setup_credential(&setup_for_body, &first_for_body).await;
            let second_credential =
                insert_setup_credential(&setup_for_body, &second_for_body).await;
            let first_hash = fresh_test_token_hash();
            let second_hash = fresh_test_token_hash();

            let first_result = sqlx::query_scalar::<_, uuid::Uuid>(
                "INSERT INTO external_access_tokens (
                     api_client_credential_id, token_hash, issued_at, expires_at
                 ) VALUES ($1, $2, $3, $4)
                 RETURNING id",
            )
            .bind(first_credential)
            .bind(first_hash.to_vec())
            .bind(Utc::now())
            .bind(Utc::now() + chrono::Duration::minutes(15))
            .fetch_one(&setup_for_body)
            .await;
            let second_result = sqlx::query_scalar::<_, uuid::Uuid>(
                "INSERT INTO external_access_tokens (
                     api_client_credential_id, token_hash, issued_at, expires_at
                 ) VALUES ($1, $2, $3, $4)
                 RETURNING id",
            )
            .bind(second_credential)
            .bind(second_hash.to_vec())
            .bind(Utc::now())
            .bind(Utc::now() + chrono::Duration::minutes(15))
            .fetch_one(&setup_for_body)
            .await;

            assert_ne!(first_hash, second_hash);
            assert!(first_result.is_ok());
            assert!(second_result.is_ok());
        },
    )
    .await;
}

#[tokio::test]
async fn runtime_can_revoke_credential() {
    let (setup_pool, runtime_pool) = permission_pools_only().await;
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let fixture = create_auth_fixture(&setup_pool).await;
    let credential_id = insert_setup_credential(&setup_pool, &fixture).await;
    let fixture_for_body = fixture.clone();
    run_auth_fixture_body(&setup_pool, vec![fixture], move || async move {
        let mut runtime_connection = runtime_pool.acquire().await.unwrap();
        let db_before: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&mut *runtime_connection)
            .await
            .unwrap();
        let revoked_at: chrono::DateTime<Utc> = sqlx::query_scalar(
            "UPDATE api_client_credentials
             SET revoked_at = clock_timestamp(), revoked_by_staff_user_id = $2,
                 revocation_reason = 'ADMIN_REQUEST'
             WHERE id = $1
             RETURNING revoked_at",
        )
        .bind(credential_id)
        .bind(fixture_for_body.staff_id)
        .fetch_one(&mut *runtime_connection)
        .await
        .unwrap();
        let db_after: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&mut *runtime_connection)
            .await
            .unwrap();
        assert!(db_before <= revoked_at);
        assert!(revoked_at <= db_after);
    })
    .await;
}

#[tokio::test]
async fn runtime_can_revoke_token() {
    let (setup_pool, runtime_pool) = permission_pools_only().await;
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let fixture = create_auth_fixture(&setup_pool).await;
    let credential_id = insert_setup_credential(&setup_pool, &fixture).await;
    let token_id = insert_setup_token(&setup_pool, credential_id).await;
    run_auth_fixture_body(&setup_pool, vec![fixture], move || async move {
        let mut runtime_connection = runtime_pool.acquire().await.unwrap();
        let db_before: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&mut *runtime_connection)
            .await
            .unwrap();
        let revoked_at: chrono::DateTime<Utc> = sqlx::query_scalar(
            "UPDATE external_access_tokens
             SET revoked_at = clock_timestamp()
             WHERE id = $1
             RETURNING revoked_at",
        )
        .bind(token_id)
        .fetch_one(&mut *runtime_connection)
        .await
        .unwrap();
        let db_after: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&mut *runtime_connection)
            .await
            .unwrap();
        assert!(db_before <= revoked_at);
        assert!(revoked_at <= db_after);
    })
    .await;
}

#[tokio::test]
async fn runtime_cannot_update_credential_identity_fields() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    for statement in [
        "UPDATE api_client_credentials SET id = id",
        "UPDATE api_client_credentials SET api_client_id = api_client_id",
    ] {
        assert!(sqlx::query(statement).execute(&runtime_pool).await.is_err());
    }
}

#[tokio::test]
async fn runtime_cannot_update_immutable_verifier_fields() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    for statement in [
        "UPDATE api_client_credentials SET secret_digest = secret_digest",
        "UPDATE api_client_credentials SET digest_version = digest_version",
    ] {
        assert!(sqlx::query(statement).execute(&runtime_pool).await.is_err());
    }
}

#[tokio::test]
async fn runtime_cannot_update_credential_issuance_fields() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    for statement in [
        "UPDATE api_client_credentials SET issued_at = issued_at",
        "UPDATE api_client_credentials SET issued_by_staff_user_id = issued_by_staff_user_id",
    ] {
        assert!(sqlx::query(statement).execute(&runtime_pool).await.is_err());
    }
}

#[tokio::test]
async fn runtime_cannot_update_token_identity_fields() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    for statement in [
        "UPDATE external_access_tokens SET id = id",
        "UPDATE external_access_tokens SET api_client_credential_id = api_client_credential_id",
    ] {
        assert!(sqlx::query(statement).execute(&runtime_pool).await.is_err());
    }
}

#[tokio::test]
async fn runtime_cannot_update_token_hash() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    assert!(
        sqlx::query("UPDATE external_access_tokens SET token_hash = token_hash")
            .execute(&runtime_pool)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn runtime_cannot_update_token_issuance_fields() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    for statement in [
        "UPDATE external_access_tokens SET issued_at = issued_at",
        "UPDATE external_access_tokens SET expires_at = expires_at",
    ] {
        assert!(sqlx::query(statement).execute(&runtime_pool).await.is_err());
    }
}

#[tokio::test]
async fn runtime_cannot_delete_auth_history() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    for statement in [
        "DELETE FROM api_client_credentials",
        "DELETE FROM external_access_tokens",
        "TRUNCATE api_client_credentials",
        "TRUNCATE external_access_tokens",
    ] {
        assert!(sqlx::query(statement).execute(&runtime_pool).await.is_err());
    }
}

#[tokio::test]
async fn runtime_cannot_alter_or_transfer_auth_tables() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    for table in ["api_client_credentials", "external_access_tokens"] {
        for statement in [
            format!("ALTER TABLE {table} OWNER TO x_fly_runtime"),
            format!("DROP TABLE {table}"),
        ] {
            assert!(sqlx::query(&statement)
                .execute(&runtime_pool)
                .await
                .is_err());
        }
    }
}

#[tokio::test]
async fn runtime_schema_and_function_privileges_are_narrow() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    let schema_usage: bool =
        sqlx::query_scalar("SELECT has_schema_privilege('x_fly_runtime', 'public', 'USAGE')")
            .fetch_one(&runtime_pool)
            .await
            .unwrap();
    let schema_create: bool =
        sqlx::query_scalar("SELECT has_schema_privilege('x_fly_runtime', 'public', 'CREATE')")
            .fetch_one(&runtime_pool)
            .await
            .unwrap();
    assert!(schema_usage);
    assert!(!schema_create);

    let public_execute: bool = sqlx::query_scalar(
        "SELECT has_function_privilege(
             'public', 'public.has_protected_stripe_card_finalization(uuid)', 'EXECUTE'
         )",
    )
    .fetch_one(&runtime_pool)
    .await
    .unwrap();
    let runtime_execute: bool = sqlx::query_scalar(
        "SELECT has_function_privilege(
             'x_fly_runtime', 'public.has_protected_stripe_card_finalization(uuid)', 'EXECUTE'
         )",
    )
    .fetch_one(&runtime_pool)
    .await
    .unwrap();
    assert!(!public_execute);
    assert!(runtime_execute);
}

#[tokio::test]
async fn runtime_cannot_mutate_migration_ledger() {
    let (_setup_pool, runtime_pool) = permission_pools_only().await;
    assert!(sqlx::query("UPDATE _sqlx_migrations SET success = success")
        .execute(&runtime_pool)
        .await
        .is_err());
}
