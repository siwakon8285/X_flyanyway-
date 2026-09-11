mod common;

use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    response::IntoResponse,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::ServiceExt;
use uuid::Uuid;
use x_fly_api::{
    application::staff_auth::{ProvisionMode, StaffAuthService},
    domain::staff::{PermissionCode, RoleCode},
    infrastructure::{
        database::{prepare_test_database, SqlxSeatHoldRepository, SqlxStaffAuthRepository},
        http::{
            admin::{permission_response, AuthenticatedStaff},
            build_router,
        },
        password::Argon2PasswordService,
    },
    state::AppState,
};

async fn fixture_guard() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn test_pool() -> PgPool {
    let database_url = common::test_database_url();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();
    prepare_test_database(&pool).await.unwrap();
    sqlx::raw_sql(
        "DELETE FROM staff_sessions;
         DELETE FROM staff_login_throttles;
         DELETE FROM staff_user_roles;
         DELETE FROM staff_users;",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

fn staff_service(pool: PgPool) -> StaffAuthService {
    StaffAuthService::new(
        Arc::new(SqlxStaffAuthRepository::new(pool)),
        Argon2PasswordService::default(),
        Duration::from_secs(3600),
    )
    .unwrap()
}

fn app(pool: PgPool, auth: StaffAuthService) -> axum::Router {
    let booking = Arc::new(SqlxSeatHoldRepository::new(pool));
    build_router(
        AppState::new(
            booking.clone(),
            booking.clone(),
            booking.clone(),
            booking,
            Duration::from_secs(600),
            true,
            "http://localhost:3000".to_owned(),
        )
        .with_staff_auth(auth),
    )
}

async fn body(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    }
}

async fn provision(auth: &StaffAuthService) {
    auth.provision(
        ProvisionMode::Bootstrap,
        "system@x-fly.internal",
        "System admin passphrase 2026",
        &[RoleCode::SystemAdmin],
    )
    .await
    .unwrap();
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
        .unwrap()
}

#[tokio::test]
async fn server_generates_unique_request_ids_and_replaces_client_values() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let router = app(pool.clone(), staff_service(pool));

    let first = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health?passengerName=must-not-be-logged")
                .header("x-request-id", "client-controlled")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    assert!(first.headers().get(header::SET_COOKIE).is_none());
    let first_id = first.headers()["x-request-id"].to_str().unwrap();
    assert_ne!(first_id, "client-controlled");
    Uuid::parse_str(first_id).unwrap();

    let second = router
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::OK);
    let second_id = second.headers()["x-request-id"].to_str().unwrap();
    Uuid::parse_str(second_id).unwrap();
    assert_ne!(first_id, second_id);
}

#[tokio::test]
async fn login_session_and_repeated_logout_use_a_separate_private_cookie() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let auth = staff_service(pool.clone());
    provision(&auth).await;
    let router = app(pool, auth);

    let login = router
        .clone()
        .oneshot(login_request(
            "system@x-fly.internal",
            "System admin passphrase 2026",
            true,
        ))
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    assert_eq!(login.headers()[header::CACHE_CONTROL], "no-store, private");
    Uuid::parse_str(login.headers()["x-request-id"].to_str().unwrap()).unwrap();
    let set_cookie = login.headers()[header::SET_COOKIE].to_str().unwrap();
    assert!(set_cookie.starts_with("x_fly_staff_session="));
    assert!(set_cookie.contains("Path=/admin"));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Strict"));
    assert!(set_cookie.contains("Secure"));
    let cookie = set_cookie.split(';').next().unwrap().to_owned();
    let login_body = body(login).await;
    assert_eq!(login_body["email"], "system@x-fly.internal");
    assert_eq!(login_body["roles"], json!(["SYSTEM_ADMIN"]));
    let serialized = login_body.to_string();
    for secret in [
        "password",
        "passwordHash",
        "token",
        "staffUserId",
        "sessionId",
    ] {
        assert!(!serialized.contains(secret));
    }

    let session = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/auth/session")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(session.status(), StatusCode::OK);

    for expected in [StatusCode::NO_CONTENT, StatusCode::NO_CONTENT] {
        let logout = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/auth/logout")
                    .header(header::COOKIE, &cookie)
                    .header(header::ORIGIN, "http://localhost:3000")
                    .header("x-x-fly-csrf", "1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(logout.status(), expected);
        assert!(logout.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .contains("Max-Age=0"));
    }

    let rejected = router
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/auth/session")
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_rejects_missing_csrf_and_keeps_unknown_wrong_and_disabled_generic() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let auth = staff_service(pool.clone());
    provision(&auth).await;
    let router = app(pool.clone(), auth);

    let csrf = router
        .clone()
        .oneshot(login_request(
            "system@x-fly.internal",
            "System admin passphrase 2026",
            false,
        ))
        .await
        .unwrap();
    assert_eq!(csrf.status(), StatusCode::FORBIDDEN);

    let unknown = router
        .clone()
        .oneshot(login_request(
            "unknown@x-fly.internal",
            "wrong password",
            true,
        ))
        .await
        .unwrap();
    let unknown_status = unknown.status();
    let unknown_body = body(unknown).await;
    let wrong = router
        .clone()
        .oneshot(login_request(
            "system@x-fly.internal",
            "wrong password",
            true,
        ))
        .await
        .unwrap();
    let wrong_status = wrong.status();
    let wrong_body = body(wrong).await;
    sqlx::query("UPDATE staff_users SET status='DISABLED', disabled_at=NOW() WHERE email=$1")
        .bind("system@x-fly.internal")
        .execute(&pool)
        .await
        .unwrap();
    let disabled = router
        .oneshot(login_request(
            "system@x-fly.internal",
            "System admin passphrase 2026",
            true,
        ))
        .await
        .unwrap();
    let disabled_status = disabled.status();
    let disabled_body = body(disabled).await;

    assert_eq!(unknown_status, StatusCode::UNAUTHORIZED);
    assert_eq!(unknown_status, wrong_status);
    assert_eq!(wrong_status, disabled_status);
    assert_eq!(unknown_body, wrong_body);
    assert_eq!(wrong_body, disabled_body);
    assert_eq!(wrong_body["error"]["code"], "STAFF_LOGIN_FAILED");
}

#[tokio::test]
async fn authorization_distinguishes_unauthenticated_from_missing_permission() {
    let principal = x_fly_api::domain::staff::StaffPrincipal::new(
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        "system@x-fly.internal".into(),
        vec![RoleCode::SystemAdmin],
        vec![PermissionCode::StaffManage],
        chrono::Utc::now() + chrono::Duration::hours(1),
    );
    assert_eq!(
        permission_response(None, PermissionCode::FlightsWrite).status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        permission_response(Some(&principal), PermissionCode::FlightsWrite).status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        permission_response(Some(&principal), PermissionCode::StaffManage).status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        AuthenticatedStaff(principal.clone())
            .require(PermissionCode::FlightsWrite)
            .unwrap_err()
            .into_response()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert!(AuthenticatedStaff(principal)
        .require(PermissionCode::StaffManage)
        .is_ok());
}
