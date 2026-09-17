mod common;

use std::{future::Future, sync::Arc, time::Duration};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::ServiceExt;
use uuid::Uuid;
use x_fly_api::{
    application::staff_auth::{ProvisionMode, StaffAuthError, StaffAuthService},
    domain::staff::RoleCode,
    infrastructure::{
        database::{prepare_test_database, verify_database_ready, SqlxSeatHoldRepository},
        http::build_router,
        password::Argon2PasswordService,
    },
    state::AppState,
};

const SYSTEM_EMAIL: &str = "security-audit-system@x-fly.internal";
const SYSTEM_PASSWORD: &str = "Security audit system passphrase 2026";

async fn fixture_guard() -> common::TestFixtureLock {
    common::acquire_test_fixture_lock_named("x-fly-staff-security-audit").await
}

async fn test_pool() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&common::test_database_url())
        .await
        .expect("connect to TEST database");
    prepare_test_database(&pool)
        .await
        .expect("TEST database migrations are ready");
    clean_fixture(&pool)
        .await
        .expect("clean TEST staff fixture");
    pool
}

async fn runtime_pool() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&common::test_runtime_database_url())
        .await
        .expect("connect to TEST runtime database");
    verify_database_ready(&pool)
        .await
        .expect("TEST runtime database migrations are ready");
    pool
}

async fn clean_fixture(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(
        "DELETE FROM staff_security_audit;
         DELETE FROM staff_sessions;
         DELETE FROM staff_login_throttles;
         DELETE FROM staff_user_roles;
         DELETE FROM staff_users;",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn run_fixture_body<F, Fut>(pool: PgPool, body: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let cleanup_pool = pool.clone();
    common::run_fixture_body_with_cleanup(body, move || async move {
        clean_fixture(&cleanup_pool).await
    })
    .await;
}

fn staff_service(pool: PgPool) -> StaffAuthService {
    StaffAuthService::new(
        Arc::new(x_fly_api::infrastructure::database::SqlxStaffAuthRepository::new(pool)),
        Argon2PasswordService::default(),
        Duration::from_secs(3600),
    )
    .expect("staff auth service")
}

fn app(pool: PgPool, auth: StaffAuthService) -> axum::Router {
    let booking = Arc::new(SqlxSeatHoldRepository::new(pool));
    build_router(
        AppState::new(
            booking.clone(),
            booking.clone(),
            booking.clone(),
            booking.clone(),
            Duration::from_secs(600),
            true,
            "http://localhost:3000".to_owned(),
        )
        .with_staff_auth(auth),
    )
}

async fn provision(auth: &StaffAuthService, email: &str, roles: &[RoleCode]) {
    auth.provision(ProvisionMode::Bootstrap, email, SYSTEM_PASSWORD, roles)
        .await
        .expect("provision TEST staff account");
}

fn login_request(email: &str, password: &str, csrf: bool) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/v1/admin/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ORIGIN, "http://localhost:3000");
    if csrf {
        builder = builder.header("x-x-fly-csrf", "1");
    }
    builder
        .body(Body::from(
            json!({"email": email, "password": password}).to_string(),
        ))
        .expect("login request")
}

fn cookie_from_login(response: &axum::response::Response) -> String {
    response.headers()[header::SET_COOKIE]
        .to_str()
        .expect("set-cookie header")
        .split(';')
        .next()
        .expect("session cookie")
        .to_owned()
}

async fn audit_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM staff_security_audit")
        .fetch_one(pool)
        .await
        .expect("count staff security audit rows")
}

async fn grant_runtime_audit_insert(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "GRANT INSERT (action, actor_staff_user_id, session_id, permission_code, request_id)
         ON TABLE public.staff_security_audit TO x_fly_runtime",
    )
    .execute(pool)
    .await
    .map(|_| ())
}

async fn session_count_for_user(pool: &PgPool, email: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM staff_sessions
         WHERE staff_user_id = (SELECT id FROM staff_users WHERE email = $1)",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .expect("count staff sessions")
}

#[tokio::test]
async fn successful_login_has_a_durable_attributable_audit_event() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    run_fixture_body(pool.clone(), move || async move {
        let auth = staff_service(pool.clone());
        provision(&auth, SYSTEM_EMAIL, &[RoleCode::SystemAdmin]).await;
        let router = app(pool.clone(), auth);
        let response = router
            .oneshot(login_request(SYSTEM_EMAIL, SYSTEM_PASSWORD, true))
            .await
            .expect("login response");
        assert_eq!(response.status(), StatusCode::OK);
        let request_id = Uuid::parse_str(response.headers()["x-request-id"].to_str().unwrap())
            .expect("server request id");
        assert_eq!(audit_count(&pool).await, 1);
        let row: (String, Uuid, Uuid, Uuid) = sqlx::query_as(
            "SELECT action, actor_staff_user_id, session_id, request_id
             FROM staff_security_audit",
        )
        .fetch_one(&pool)
        .await
        .expect("successful login audit row");
        assert_eq!(row.0, "STAFF_LOGIN_SUCCEEDED");
        let staff_id: Uuid = sqlx::query_scalar("SELECT id FROM staff_users WHERE email = $1")
            .bind(SYSTEM_EMAIL)
            .fetch_one(&pool)
            .await
            .expect("staff identity");
        assert_eq!(row.1, staff_id);
        assert_eq!(row.3, request_id);
        assert_ne!(row.1, Uuid::nil());
        assert_ne!(row.2, Uuid::nil());
    })
    .await;
}

#[tokio::test]
async fn successful_login_audit_failure_rolls_back_session_creation() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let runtime = runtime_pool().await;
    run_fixture_body_with_audit_grant_cleanup(pool.clone(), runtime.clone(), move || async move {
        let setup_auth = staff_service(pool.clone());
        provision(&setup_auth, SYSTEM_EMAIL, &[RoleCode::SystemAdmin]).await;
        sqlx::query(
            "REVOKE INSERT (request_id) ON TABLE public.staff_security_audit FROM x_fly_runtime",
        )
        .execute(&pool)
        .await
        .expect("revoke TEST runtime audit request_id insert permission");

        let runtime_auth = staff_service(runtime.clone());
        let result = runtime_auth
            .login_with_request_id(SYSTEM_EMAIL, SYSTEM_PASSWORD, Uuid::new_v4())
            .await;
        assert_eq!(result, Err(StaffAuthError::Infrastructure));
        assert_eq!(session_count_for_user(&pool, SYSTEM_EMAIL).await, 0);
        assert_eq!(audit_count(&pool).await, 0);
    })
    .await;
}

#[tokio::test]
async fn self_logout_audit_failure_rolls_back_revocation() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let runtime = runtime_pool().await;
    run_fixture_body_with_audit_grant_cleanup(pool.clone(), runtime.clone(), move || async move {
        let setup_auth = staff_service(pool.clone());
        provision(&setup_auth, SYSTEM_EMAIL, &[RoleCode::SystemAdmin]).await;
        let runtime_auth = staff_service(runtime.clone());
        let login = runtime_auth
            .login_with_request_id(SYSTEM_EMAIL, SYSTEM_PASSWORD, Uuid::new_v4())
            .await
            .expect("runtime login");
        sqlx::query(
            "REVOKE INSERT (request_id) ON TABLE public.staff_security_audit FROM x_fly_runtime",
        )
        .execute(&pool)
        .await
        .expect("revoke TEST runtime audit request_id insert permission");

        let result = runtime_auth
            .logout_with_request_id(&login.token, Uuid::new_v4())
            .await;
        assert_eq!(result, Err(StaffAuthError::Infrastructure));
        let active: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM staff_sessions
             WHERE staff_user_id = (SELECT id FROM staff_users WHERE email = $1)
               AND revoked_at IS NULL",
        )
        .bind(SYSTEM_EMAIL)
        .fetch_one(&pool)
        .await
        .expect("inspect rollback session state");
        assert_eq!(active, 1);
        assert_eq!(audit_count(&pool).await, 1);
    })
    .await;
}

async fn run_fixture_body_with_audit_grant_cleanup<F, Fut>(pool: PgPool, runtime: PgPool, body: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    common::run_fixture_body_with_cleanup(body, move || async move {
        grant_runtime_audit_insert(&pool).await?;
        clean_fixture(&pool).await?;
        drop(runtime);
        Ok::<(), sqlx::Error>(())
    })
    .await;
}

#[tokio::test]
async fn failed_unknown_and_blocked_login_attempts_create_no_f03_rows() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    run_fixture_body(pool.clone(), move || async move {
        let auth = staff_service(pool.clone());
        provision(&auth, SYSTEM_EMAIL, &[RoleCode::SystemAdmin]).await;
        let router = app(pool.clone(), auth);

        for (email, password) in [
            (SYSTEM_EMAIL, "wrong password"),
            ("unknown-security-audit@x-fly.internal", "wrong password"),
        ] {
            let response = router
                .clone()
                .oneshot(login_request(email, password, true))
                .await
                .expect("failed login response");
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }

        for _ in 0..4 {
            let response = router
                .clone()
                .oneshot(login_request(SYSTEM_EMAIL, "wrong password", true))
                .await
                .expect("throttle login response");
            assert!(matches!(
                response.status(),
                StatusCode::UNAUTHORIZED | StatusCode::TOO_MANY_REQUESTS
            ));
        }
        let blocked = router
            .clone()
            .oneshot(login_request(SYSTEM_EMAIL, "wrong password", true))
            .await
            .expect("blocked login response");
        assert_eq!(blocked.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(audit_count(&pool).await, 0);
    })
    .await;
}

#[tokio::test]
async fn self_logout_creates_one_event_and_unknown_or_repeated_logout_creates_none() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    run_fixture_body(pool.clone(), move || async move {
        let auth = staff_service(pool.clone());
        provision(&auth, SYSTEM_EMAIL, &[RoleCode::SystemAdmin]).await;
        let router = app(pool.clone(), auth);
        let login = router
            .clone()
            .oneshot(login_request(SYSTEM_EMAIL, SYSTEM_PASSWORD, true))
            .await
            .expect("login response");
        let cookie = cookie_from_login(&login);

        for expected in [StatusCode::NO_CONTENT, StatusCode::NO_CONTENT] {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/admin/auth/logout")
                        .header(header::COOKIE, &cookie)
                        .header(header::ORIGIN, "http://localhost:3000")
                        .header("x-x-fly-csrf", "1")
                        .body(Body::empty())
                        .expect("logout request"),
                )
                .await
                .expect("logout response");
            assert_eq!(response.status(), expected);
        }
        assert_eq!(audit_count(&pool).await, 2);
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM staff_security_audit
                 WHERE action = 'STAFF_SESSION_REVOKED'",
            )
            .fetch_one(&pool)
            .await
            .expect("count session revocation audit rows"),
            1
        );

        let unknown = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/auth/logout")
                    .header(header::ORIGIN, "http://localhost:3000")
                    .header("x-x-fly-csrf", "1")
                    .header(
                        header::COOKIE,
                        format!("x_fly_staff_session={}", hex::encode([0_u8; 32])),
                    )
                    .body(Body::empty())
                    .expect("unknown logout request"),
            )
            .await
            .expect("unknown logout response");
        assert_eq!(unknown.status(), StatusCode::NO_CONTENT);
        assert_eq!(audit_count(&pool).await, 2);
    })
    .await;
}

#[tokio::test]
async fn typed_rbac_denials_are_audited_but_generic_forbidden_is_not() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    run_fixture_body(pool.clone(), move || async move {
        let auth = staff_service(pool.clone());
        provision(&auth, SYSTEM_EMAIL, &[RoleCode::FlightManager]).await;
        let router = app(pool.clone(), auth);
        let login = router
            .clone()
            .oneshot(login_request(SYSTEM_EMAIL, SYSTEM_PASSWORD, true))
            .await
            .expect("login response");
        let cookie = cookie_from_login(&login);

        for _ in 0..2 {
            let denied = router
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/admin/api-clients")
                        .header(header::COOKIE, &cookie)
                        .body(Body::empty())
                        .expect("RBAC denial request"),
                )
                .await
                .expect("RBAC denial response");
            assert_eq!(denied.status(), StatusCode::FORBIDDEN);
        }
        let (action, actor, session, permission, request_id): (String, Uuid, Uuid, String, Uuid) =
            sqlx::query_as(
                "SELECT action, actor_staff_user_id, session_id, permission_code, request_id
                 FROM staff_security_audit
                 WHERE action = 'STAFF_AUTHZ_DENIED'",
            )
            .fetch_one(&pool)
            .await
            .expect("RBAC audit row");
        assert_eq!(action, "STAFF_AUTHZ_DENIED");
        assert_eq!(permission, "api_clients:read");
        assert_ne!(actor, Uuid::nil());
        assert_ne!(session, Uuid::nil());
        assert_ne!(request_id, Uuid::nil());

        let generic = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/flights")
                    .header(header::COOKIE, &cookie)
                    .body(Body::empty())
                    .expect("generic forbidden request"),
            )
            .await
            .expect("generic forbidden response");
        assert_eq!(generic.status(), StatusCode::FORBIDDEN);
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM staff_security_audit WHERE action = 'STAFF_AUTHZ_DENIED'",
            )
            .fetch_one(&pool)
            .await
            .expect("count RBAC audit rows"),
            1
        );
    })
    .await;
}

#[tokio::test]
async fn authorization_audit_failure_cannot_convert_denial_to_allow() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let runtime = runtime_pool().await;
    run_fixture_body_with_audit_grant_cleanup(pool.clone(), runtime.clone(), move || async move {
        let setup_auth = staff_service(pool.clone());
        provision(&setup_auth, SYSTEM_EMAIL, &[RoleCode::FlightManager]).await;
        let runtime_auth = staff_service(runtime.clone());
        let login = runtime_auth
            .login_with_request_id(SYSTEM_EMAIL, SYSTEM_PASSWORD, Uuid::new_v4())
            .await
            .expect("runtime login");
        sqlx::query(
            "REVOKE INSERT (request_id) ON TABLE public.staff_security_audit FROM x_fly_runtime",
        )
        .execute(&pool)
        .await
        .expect("revoke TEST runtime audit request_id insert permission");
        let router = app(runtime.clone(), runtime_auth);
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/api-clients")
                    .header(
                        header::COOKIE,
                        format!("x_fly_staff_session={}", login.token),
                    )
                    .body(Body::empty())
                    .expect("RBAC denial request"),
            )
            .await
            .expect("RBAC denial response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let authz_rows: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM staff_security_audit WHERE action = 'STAFF_AUTHZ_DENIED'",
        )
        .fetch_one(&pool)
        .await
        .expect("count authz audit rows");
        assert_eq!(authz_rows, 0);
    })
    .await;
}

#[tokio::test]
async fn audit_rows_do_not_contain_secret_sentinels() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    run_fixture_body(pool.clone(), move || async move {
        let auth = staff_service(pool.clone());
        let sentinel_email = "f03-identifier-secret-sentinel@x-fly.internal";
        provision(&auth, sentinel_email, &[RoleCode::SystemAdmin]).await;
        let router = app(pool.clone(), auth);
        let sentinel = "f03-password-bearer-cookie-authorization-body-sentinel";
        let response = router
            .clone()
            .oneshot(login_request(sentinel_email, SYSTEM_PASSWORD, true))
            .await
            .expect("sentinel success response");
        assert_eq!(response.status(), StatusCode::OK);
        let failed = router
            .oneshot(login_request(sentinel_email, sentinel, true))
            .await
            .expect("sentinel failure response");
        assert_eq!(failed.status(), StatusCode::UNAUTHORIZED);

        let rows: Vec<(String, Option<String>)> =
            sqlx::query_as("SELECT action, permission_code::text FROM staff_security_audit")
                .fetch_all(&pool)
                .await
                .expect("audit rows");
        let rendered = format!("{rows:?}");
        assert!(!rendered.contains(sentinel_email));
        assert!(!rendered.contains(sentinel));
    })
    .await;
}

#[tokio::test]
async fn staff_security_audit_table_has_append_only_runtime_privileges() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;

    let table_exists: bool =
        sqlx::query_scalar("SELECT to_regclass('public.staff_security_audit') IS NOT NULL")
            .fetch_one(&pool)
            .await
            .expect("inspect audit table");
    assert!(table_exists, "F03 requires the audit table migration");

    let checks: Vec<(String, bool)> = sqlx::query_as(
        "SELECT privilege, has_table_privilege('x_fly_runtime', 'public.staff_security_audit', privilege)
         FROM unnest(ARRAY['INSERT','SELECT','UPDATE','DELETE','TRUNCATE','REFERENCES','TRIGGER']) AS privilege",
    )
    .fetch_all(&pool)
    .await
    .expect("inspect runtime table privileges");
    assert!(checks.iter().all(|(_, granted)| !granted));

    for column in [
        "action",
        "actor_staff_user_id",
        "session_id",
        "permission_code",
        "request_id",
    ] {
        let granted: bool = sqlx::query_scalar(
            "SELECT has_column_privilege('x_fly_runtime', 'public.staff_security_audit', $1, 'INSERT')",
        )
        .bind(column)
        .fetch_one(&pool)
        .await
        .expect("inspect runtime column INSERT privilege");
        assert!(granted, "runtime must insert approved column {column}");
    }
    for column in ["id", "created_at"] {
        let granted: bool = sqlx::query_scalar(
            "SELECT has_column_privilege('x_fly_runtime', 'public.staff_security_audit', $1, 'INSERT')",
        )
        .bind(column)
        .fetch_one(&pool)
        .await
        .expect("inspect protected runtime column privilege");
        assert!(
            !granted,
            "runtime must not insert generated column {column}"
        );
    }
}
