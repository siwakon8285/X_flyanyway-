mod common;

use std::{sync::Arc, time::Duration};

use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Algorithm, Argon2, Params, Version,
};
use chrono::{Duration as ChronoDuration, NaiveDate};
use sqlx::{PgPool, Row};
use x_fly_api::{
    application::staff_auth::StaffAuthService,
    domain::{
        entities::{CreateSeatHold, FlightSelection},
        repositories::SeatHoldRepository,
        value_objects::{CabinClass, PassengerCounts, SeatNumber},
    },
    infrastructure::database::{
        prepare_test_database, verify_database_ready, SqlxSeatHoldRepository,
        SqlxStaffAuthRepository,
    },
    infrastructure::password::Argon2PasswordService,
};

#[tokio::test]
async fn runtime_role_has_required_positive_and_negative_permissions() {
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

    let departure_date = NaiveDate::from_ymd_opt(2100, 1, 1).unwrap()
        + ChronoDuration::days((uuid::Uuid::new_v4().as_u128() % 50_000) as i64);
    let repository = SqlxSeatHoldRepository::new(runtime_pool.clone());
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
    repository.get_hold(hold.id, [0x6b; 32]).await.unwrap();
    sqlx::query("DELETE FROM seat_holds WHERE id = $1")
        .bind(hold.id)
        .execute(&setup_pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM flight_instances WHERE departure_date = $1")
        .bind(departure_date)
        .execute(&setup_pool)
        .await
        .unwrap();

    assert!(
        sqlx::query("CREATE TABLE public.runtime_must_not_create(id integer)")
            .execute(&runtime_pool)
            .await
            .is_err()
    );
    assert!(sqlx::query(
        "ALTER TABLE public.flight_services ADD COLUMN runtime_must_not_add integer"
    )
    .execute(&runtime_pool)
    .await
    .is_err());
    assert!(sqlx::query("DROP TABLE public.airports")
        .execute(&runtime_pool)
        .await
        .is_err());
    assert!(sqlx::query("CREATE ROLE runtime_must_not_create")
        .execute(&runtime_pool)
        .await
        .is_err());
    assert!(sqlx::query(
        "INSERT INTO staff_users (email, password_hash) VALUES ('runtime-must-not-provision@x-fly.test', 'nope')",
    )
    .execute(&runtime_pool)
    .await
    .is_err());
    assert!(sqlx::query(
        "INSERT INTO staff_user_roles (staff_user_id, role_code) VALUES ($1, 'SYSTEM_ADMIN')",
    )
    .bind(uuid::Uuid::new_v4())
    .execute(&runtime_pool)
    .await
    .is_err());
    assert!(sqlx::query("ALTER ROLE x_fly_runtime CREATEDB")
        .execute(&runtime_pool)
        .await
        .is_err());
    assert!(sqlx::query("CREATE DATABASE runtime_must_not_create")
        .execute(&runtime_pool)
        .await
        .is_err());
    assert!(sqlx::query("SET ROLE x_fly_migrator")
        .execute(&runtime_pool)
        .await
        .is_err());
    assert!(sqlx::query("UPDATE _sqlx_migrations SET success = success")
        .execute(&runtime_pool)
        .await
        .is_err());
}

#[tokio::test]
async fn runtime_staff_login_can_rehash_without_provisioning_privileges() {
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
    let staff_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email, password_hash) VALUES ($1, $2) RETURNING id",
    )
    .bind(&email)
    .bind(&weak_hash)
    .fetch_one(&setup_pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO staff_user_roles (staff_user_id, role_code) VALUES ($1, 'SYSTEM_ADMIN')",
    )
    .bind(staff_id)
    .execute(&setup_pool)
    .await
    .unwrap();

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
            .fetch_one(&setup_pool)
            .await
            .unwrap();
    assert_ne!(stored_hash, weak_hash);
    assert!(stored_hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));

    sqlx::query("DELETE FROM staff_sessions WHERE staff_user_id=$1")
        .bind(staff_id)
        .execute(&setup_pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM staff_user_roles WHERE staff_user_id=$1")
        .bind(staff_id)
        .execute(&setup_pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM staff_users WHERE id=$1")
        .bind(staff_id)
        .execute(&setup_pool)
        .await
        .unwrap();
}
