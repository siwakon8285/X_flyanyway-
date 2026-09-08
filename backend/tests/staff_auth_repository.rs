mod common;

use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use sqlx::{postgres::PgPoolOptions, PgPool};
use x_fly_api::{
    application::staff_auth::{ProvisionMode, StaffAuthError, StaffAuthService},
    domain::staff::{PermissionCode, RoleCode},
    infrastructure::{
        database::{prepare_database, SqlxStaffAuthRepository},
        password::Argon2PasswordService,
    },
};

async fn fixture_guard() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn clean_staff(pool: &PgPool) {
    sqlx::raw_sql(
        "DELETE FROM staff_sessions;
         DELETE FROM staff_login_throttles;
         DELETE FROM staff_user_roles;
         DELETE FROM staff_users;",
    )
    .execute(pool)
    .await
    .unwrap();
}

fn service(pool: PgPool) -> StaffAuthService {
    StaffAuthService::new(
        Arc::new(SqlxStaffAuthRepository::new(pool)),
        Argon2PasswordService::default(),
        Duration::from_secs(3600),
    )
    .unwrap()
}

async fn test_pool() -> PgPool {
    let database_url = common::test_database_url();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();
    prepare_database(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn migration_seeds_the_exact_least_privilege_role_matrix() {
    let pool = test_pool().await;
    let rows: Vec<(String, Vec<String>)> = sqlx::query_as(
        "SELECT role.code, array_agg(role_grant.permission_code ORDER BY role_grant.permission_code)
         FROM roles AS role
         JOIN role_permissions AS role_grant ON role_grant.role_code = role.code
         GROUP BY role.code
         ORDER BY role.code",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(
        rows,
        vec![
            (
                "API_ADMIN".into(),
                vec!["api_clients:manage".into(), "api_clients:read".into()]
            ),
            (
                "BAGGAGE_STAFF".into(),
                vec![
                    "baggage_context:read".into(),
                    "bookings:read_limited".into(),
                    "flights:read".into(),
                    "passengers:read_limited".into()
                ]
            ),
            (
                "BOOKING_OPERATIONS".into(),
                vec!["bookings:manage".into(), "bookings:read".into()]
            ),
            (
                "EXECUTIVE".into(),
                vec![
                    "analytics:read".into(),
                    "dashboard:read".into(),
                    "reports:read".into()
                ]
            ),
            (
                "FLIGHT_MANAGER".into(),
                vec!["flights:read".into(), "flights:write".into()]
            ),
            (
                "SYSTEM_ADMIN".into(),
                vec![
                    "roles:manage".into(),
                    "roles:read".into(),
                    "staff:manage".into(),
                    "staff:read".into()
                ]
            ),
            (
                "TICKET_PASSENGER_OPERATIONS".into(),
                vec![
                    "passengers:read".into(),
                    "tickets:print".into(),
                    "tickets:read".into()
                ]
            ),
        ]
    );

    let system_admin_can_write_flights: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM role_permissions
            WHERE role_code = 'SYSTEM_ADMIN' AND permission_code = 'flights:write'
        )",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!system_admin_can_write_flights);
}

#[tokio::test]
async fn bootstrap_then_create_provisions_distinct_least_privilege_staff() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    clean_staff(&pool).await;
    let auth = service(pool.clone());

    assert_eq!(
        auth.provision(
            ProvisionMode::Create,
            "too-early@x-fly.internal",
            "Early account passphrase 2026",
            &[RoleCode::Executive],
        )
        .await,
        Err(StaffAuthError::BootstrapRequired)
    );

    auth.provision(
        ProvisionMode::Bootstrap,
        "system@x-fly.internal",
        "System admin passphrase 2026",
        &[RoleCode::SystemAdmin],
    )
    .await
    .unwrap();
    auth.provision(
        ProvisionMode::Create,
        "flight@x-fly.internal",
        "Flight manager passphrase 2026",
        &[RoleCode::FlightManager],
    )
    .await
    .unwrap();

    assert_eq!(
        auth.provision(
            ProvisionMode::Bootstrap,
            "second@x-fly.internal",
            "Second bootstrap passphrase 2026",
            &[RoleCode::Executive],
        )
        .await,
        Err(StaffAuthError::BootstrapAlreadyCompleted)
    );

    let system = auth
        .login("system@x-fly.internal", "System admin passphrase 2026")
        .await
        .unwrap();
    let flight = auth
        .login("flight@x-fly.internal", "Flight manager passphrase 2026")
        .await
        .unwrap();
    assert!(!system.principal.can(PermissionCode::FlightsWrite));
    assert!(flight.principal.can(PermissionCode::FlightsWrite));

    let stored_hash: String = sqlx::query_scalar(
        "SELECT password_hash FROM staff_users WHERE email = 'flight@x-fly.internal'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_ne!(stored_hash, "Flight manager passphrase 2026");
    assert!(stored_hash.starts_with("$argon2id$"));
}

#[tokio::test]
async fn sessions_use_current_roles_and_immediately_honor_revocation_and_disable() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    clean_staff(&pool).await;
    let auth = service(pool.clone());
    auth.provision(
        ProvisionMode::Bootstrap,
        "staff@x-fly.internal",
        "Staff account passphrase 2026",
        &[RoleCode::SystemAdmin],
    )
    .await
    .unwrap();
    let login = auth
        .login("staff@x-fly.internal", "Staff account passphrase 2026")
        .await
        .unwrap();

    let first = auth.authenticate(&login.token).await.unwrap();
    assert!(!first.can(PermissionCode::ApiClientsManage));
    sqlx::query(
        "INSERT INTO staff_user_roles (staff_user_id, role_code)
         VALUES ($1, 'API_ADMIN')",
    )
    .bind(first.staff_user_id())
    .execute(&pool)
    .await
    .unwrap();
    assert!(auth
        .authenticate(&login.token)
        .await
        .unwrap()
        .can(PermissionCode::ApiClientsManage));

    let second_instance = service(pool.clone());
    assert!(second_instance.authenticate(&login.token).await.is_ok());
    let mut tampered = login.token.clone();
    tampered.replace_range(..1, if tampered.starts_with('0') { "1" } else { "0" });
    assert_eq!(
        second_instance.authenticate(&tampered).await,
        Err(StaffAuthError::Unauthenticated)
    );

    sqlx::query(
        "DELETE FROM staff_user_roles WHERE staff_user_id = $1 AND role_code = 'API_ADMIN'",
    )
    .bind(first.staff_user_id())
    .execute(&pool)
    .await
    .unwrap();
    assert!(!auth
        .authenticate(&login.token)
        .await
        .unwrap()
        .can(PermissionCode::ApiClientsManage));

    sqlx::query(
        "UPDATE staff_sessions
         SET created_at = NOW() - INTERVAL '2 hours', expires_at = NOW() - INTERVAL '1 hour'
         WHERE id = $1",
    )
    .bind(first.session_id())
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        auth.authenticate(&login.token).await,
        Err(StaffAuthError::Unauthenticated)
    );

    auth.logout(&login.token).await.unwrap();
    assert_eq!(
        auth.authenticate(&login.token).await,
        Err(StaffAuthError::Unauthenticated)
    );

    let replacement = auth
        .login("staff@x-fly.internal", "Staff account passphrase 2026")
        .await
        .unwrap();
    sqlx::query(
        "UPDATE staff_users SET status = 'DISABLED', disabled_at = NOW(), updated_at = NOW()
         WHERE email = 'staff@x-fly.internal'",
    )
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        auth.authenticate(&replacement.token).await,
        Err(StaffAuthError::Unauthenticated)
    );
}

#[tokio::test]
async fn invalid_unknown_and_disabled_logins_are_generic_and_durably_throttled() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    clean_staff(&pool).await;
    let auth = service(pool.clone());
    auth.provision(
        ProvisionMode::Bootstrap,
        "staff@x-fly.internal",
        "Staff account passphrase 2026",
        &[RoleCode::SystemAdmin],
    )
    .await
    .unwrap();

    assert_eq!(
        auth.login("unknown@x-fly.internal", "wrong password").await,
        Err(StaffAuthError::InvalidCredentials)
    );
    assert_eq!(
        auth.login("staff@x-fly.internal", "wrong password").await,
        Err(StaffAuthError::InvalidCredentials)
    );
    for _ in 0..3 {
        assert_eq!(
            auth.login("staff@x-fly.internal", "wrong password").await,
            Err(StaffAuthError::InvalidCredentials)
        );
    }
    assert_eq!(
        auth.login("staff@x-fly.internal", "wrong password").await,
        Err(StaffAuthError::Throttled)
    );

    let persisted: i32 = sqlx::query_scalar(
        "SELECT failure_count FROM staff_login_throttles WHERE blocked_until > NOW()",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(persisted, 5);
}
