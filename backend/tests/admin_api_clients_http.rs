mod common;

use std::{ops::Deref, sync::Arc, time::Duration};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{
    postgres::{PgConnection, PgPoolOptions},
    Connection, PgPool,
};
use tower::ServiceExt;
use uuid::Uuid;
use x_fly_api::{
    application::{api_client::ApiClientManagement, staff_auth::StaffAuthService},
    infrastructure::{
        database::{
            prepare_test_database, SqlxApiClientRepository, SqlxSeatHoldRepository,
            SqlxStaffAuthRepository,
        },
        http::build_router,
        password::Argon2PasswordService,
    },
    state::AppState,
};

async fn test_pool() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&common::test_database_url())
        .await
        .unwrap();
    prepare_test_database(&pool).await.unwrap();
    pool
}

struct StaffFixture {
    id: Uuid,
    cookie: String,
}

impl Deref for StaffFixture {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.cookie
    }
}

// Only the temporary FLIGHT_MANAGER permission grant is shared state; this
// lock coordinates that grant with the companion credential-admin test.
const ROLE_PERMISSION_FIXTURE_LOCK: i64 = 0x5846_4c59_4150_4943;

async fn role_permission_fixture_lock() -> PgConnection {
    let mut connection = PgConnection::connect(&common::test_database_url())
        .await
        .unwrap();
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(ROLE_PERMISSION_FIXTURE_LOCK)
        .execute(&mut connection)
        .await
        .unwrap();
    connection
}

async fn remove_stale_role_permission_fixtures(pool: &PgPool) {
    sqlx::query(
        "DELETE FROM role_permissions
         WHERE role_code='FLIGHT_MANAGER'
           AND permission_code IN ('api_clients:read','api_clients:manage')
           AND granted_by_staff_user_id IN (
               SELECT id FROM staff_users WHERE email LIKE '%@api-client-http.test'
           )",
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn cleanup_actor(pool: &PgPool, actor: Uuid) {
    sqlx::query(
        "DELETE FROM role_permissions
         WHERE granted_by_staff_user_id=$1
           AND role_code='FLIGHT_MANAGER'
           AND permission_code IN ('api_clients:read','api_clients:manage')",
    )
    .bind(actor)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "DELETE FROM api_client_management_audit
         WHERE actor_staff_user_id=$1
            OR api_client_id IN (
                SELECT id FROM api_clients WHERE created_by_staff_user_id=$1
            )",
    )
    .bind(actor)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "DELETE FROM external_access_tokens
         WHERE api_client_credential_id IN (
             SELECT credential.id
             FROM api_client_credentials credential
             LEFT JOIN api_clients client ON client.id=credential.api_client_id
             WHERE client.created_by_staff_user_id=$1
                OR credential.issued_by_staff_user_id=$1
                OR credential.revoked_by_staff_user_id=$1
         )",
    )
    .bind(actor)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "DELETE FROM api_client_credentials
         WHERE api_client_id IN (
             SELECT id FROM api_clients WHERE created_by_staff_user_id=$1
         )
            OR issued_by_staff_user_id=$1
            OR revoked_by_staff_user_id=$1",
    )
    .bind(actor)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "DELETE FROM api_client_allowed_scopes
         WHERE api_client_id IN (
             SELECT id FROM api_clients WHERE created_by_staff_user_id=$1
         )
            OR assigned_by_staff_user_id=$1",
    )
    .bind(actor)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "DELETE FROM api_clients
         WHERE created_by_staff_user_id=$1 OR updated_by_staff_user_id=$1",
    )
    .bind(actor)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("DELETE FROM staff_sessions WHERE staff_user_id=$1")
        .bind(actor)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM staff_user_roles WHERE staff_user_id=$1")
        .bind(actor)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM staff_users WHERE id=$1")
        .bind(actor)
        .execute(pool)
        .await
        .unwrap();
}

fn app(pool: PgPool) -> axum::Router {
    let repository = Arc::new(SqlxSeatHoldRepository::new(pool.clone()));
    let auth = StaffAuthService::new(
        Arc::new(SqlxStaffAuthRepository::new(pool.clone())),
        Argon2PasswordService::default(),
        Duration::from_secs(3600),
    )
    .unwrap();
    build_router(
        AppState::new(
            repository.clone(),
            repository.clone(),
            repository.clone(),
            repository,
            Duration::from_secs(600),
            true,
            "http://localhost:3000".to_owned(),
        )
        .with_staff_auth(auth)
        .with_api_clients(ApiClientManagement::new(Arc::new(
            SqlxApiClientRepository::new(pool),
        ))),
    )
}

async fn cookie(pool: &PgPool, role: &str, label: &str) -> StaffFixture {
    let user: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash) VALUES ($1,'hash') RETURNING id",
    )
    .bind(format!("{label}-{}@api-client-http.test", Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO staff_user_roles (staff_user_id,role_code) VALUES ($1,$2)")
        .bind(user)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
    let raw: [u8; 32] = Sha256::digest(Uuid::new_v4().as_bytes()).into();
    let hash: [u8; 32] = Sha256::digest(raw).into();
    sqlx::query(
        "INSERT INTO staff_sessions (staff_user_id,token_hash,expires_at)
         VALUES ($1,$2,NOW()+INTERVAL '1 hour')",
    )
    .bind(user)
    .bind(hash.as_slice())
    .execute(pool)
    .await
    .unwrap();
    StaffFixture {
        id: user,
        cookie: format!("x_fly_staff_session={}", hex::encode(raw)),
    }
}

async fn send(
    router: &axum::Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    payload: Option<Value>,
    trusted: bool,
) -> axum::response::Response {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    if payload.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    if trusted {
        builder = builder
            .header(header::ORIGIN, "http://localhost:3000")
            .header("x-x-fly-csrf", "1");
    }
    router
        .clone()
        .oneshot(
            builder
                .body(payload.map_or_else(Body::empty, |value| Body::from(value.to_string())))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn body(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    }
}

fn create_payload(name: &str) -> Value {
    json!({
        "name": name,
        "description": "Approved aggregate reporting system",
        "status": "SUSPENDED",
        "allowedScopes": ["analytics:read"]
    })
}

fn unique_name(label: &str) -> String {
    format!("{label} {}", Uuid::new_v4().simple())
}

#[tokio::test]
async fn create_and_detail_preserve_literal_scope_codes_independent_of_input_order() {
    let pool = test_pool().await;
    let admin = cookie(&pool, "API_ADMIN", "scope-codes").await;
    let router = app(pool.clone());
    let namespace = Uuid::new_v4().simple().to_string();

    for (name, input, expected) in [
        (
            format!("Flights only {namespace}"),
            json!(["flights:read"]),
            json!(["flights:read"]),
        ),
        (
            format!("Analytics only {namespace}"),
            json!(["analytics:read"]),
            json!(["analytics:read"]),
        ),
        (
            format!("Both reversed {namespace}"),
            json!(["flights:read", "analytics:read"]),
            json!(["analytics:read", "flights:read"]),
        ),
    ] {
        let created = body(
            send(
                &router,
                "POST",
                "/api/v1/admin/api-clients",
                Some(&admin),
                Some(json!({
                    "name":name,
                    "description":null,
                    "status":"ACTIVE",
                    "allowedScopes":input,
                })),
                true,
            )
            .await,
        )
        .await;
        assert_eq!(created["allowedScopes"], expected);
        let client_id = created["clientId"].as_str().unwrap();
        let detail = body(
            send(
                &router,
                "GET",
                &format!("/api/v1/admin/api-clients/{client_id}"),
                Some(&admin),
                None,
                false,
            )
            .await,
        )
        .await;
        assert_eq!(detail["allowedScopes"], expected);
        assert_eq!(detail["audit"][0]["after"]["allowedScopes"], expected);
    }
    cleanup_actor(&pool, admin.id).await;
}

#[tokio::test]
async fn effective_permissions_authorize_without_role_name_bypasses() {
    let _role_lock = role_permission_fixture_lock().await;
    let pool = test_pool().await;
    remove_stale_role_permission_fixtures(&pool).await;
    let router = app(pool.clone());

    assert_eq!(
        send(
            &router,
            "GET",
            "/api/v1/admin/api-clients",
            None,
            None,
            false
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    let denied_system = cookie(&pool, "SYSTEM_ADMIN", "system").await;
    let denied_flight = cookie(&pool, "FLIGHT_MANAGER", "flight").await;
    let denied_executive = cookie(&pool, "EXECUTIVE", "executive").await;
    for (role, denied) in [
        ("SYSTEM_ADMIN", &denied_system),
        ("FLIGHT_MANAGER", &denied_flight),
        ("EXECUTIVE", &denied_executive),
    ] {
        assert_eq!(
            send(
                &router,
                "GET",
                "/api/v1/admin/api-clients",
                Some(denied),
                None,
                false,
            )
            .await
            .status(),
            StatusCode::FORBIDDEN,
            "{role} must not receive implicit access"
        );
    }

    let api_admin = cookie(&pool, "API_ADMIN", "api-admin").await;
    assert_eq!(
        send(
            &router,
            "GET",
            "/api/v1/admin/api-clients/scopes",
            Some(&api_admin),
            None,
            false,
        )
        .await
        .status(),
        StatusCode::OK
    );
    sqlx::query(
        "INSERT INTO role_permissions(role_code,permission_code,granted_by_staff_user_id) VALUES
            ('FLIGHT_MANAGER','api_clients:read',$1),
            ('FLIGHT_MANAGER','api_clients:manage',$1)",
    )
    .bind(api_admin.id)
    .execute(&pool)
    .await
    .unwrap();
    let granted = cookie(&pool, "FLIGHT_MANAGER", "effective").await;
    assert_eq!(
        send(
            &router,
            "POST",
            "/api/v1/admin/api-clients",
            Some(&granted),
            Some(create_payload(&unique_name("Effective grant"))),
            true,
        )
        .await
        .status(),
        StatusCode::CREATED
    );
    cleanup_actor(&pool, granted.id).await;
    cleanup_actor(&pool, api_admin.id).await;
    for actor in [denied_system.id, denied_flight.id, denied_executive.id] {
        cleanup_actor(&pool, actor).await;
    }
}

#[tokio::test]
async fn mutations_require_origin_csrf_and_return_safe_public_dtos() {
    let pool = test_pool().await;
    let admin = cookie(&pool, "API_ADMIN", "security").await;
    let router = app(pool.clone());
    let rejected = send(
        &router,
        "POST",
        "/api/v1/admin/api-clients",
        Some(&admin),
        Some(create_payload(&unique_name("Rejected origin"))),
        false,
    )
    .await;
    assert_eq!(rejected.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        rejected.headers()[header::CACHE_CONTROL],
        "no-store, private"
    );

    let response = send(
        &router,
        "POST",
        "/api/v1/admin/api-clients",
        Some(&admin),
        Some(create_payload(&unique_name("Safe DTO"))),
        true,
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store, private"
    );
    let created = body(response).await;
    assert!(created["clientId"].as_str().unwrap().starts_with("XFC"));
    for forbidden in [
        "id",
        "secret",
        "token",
        "credential",
        "createdByStaffUserId",
    ] {
        assert!(created.get(forbidden).is_none(), "must omit {forbidden}");
    }

    let client_id = created["clientId"].as_str().unwrap();
    let detail = send(
        &router,
        "GET",
        &format!("/api/v1/admin/api-clients/{client_id}"),
        Some(&admin),
        None,
        false,
    )
    .await;
    assert_eq!(detail.status(), StatusCode::OK);
    let detail = body(detail).await;
    assert_eq!(detail["audit"][0]["action"], "CLIENT_CREATED");
    assert!(detail["audit"][0].get("actorStaffUserId").is_none());
    cleanup_actor(&pool, admin.id).await;
}

#[tokio::test]
async fn rejects_pagination_offsets_that_cannot_advance() {
    let pool = test_pool().await;
    let admin = cookie(&pool, "API_ADMIN", "pagination-overflow").await;
    let router = app(pool.clone());

    let normal = send(
        &router,
        "GET",
        "/api/v1/admin/api-clients?limit=50&offset=0",
        Some(&admin),
        None,
        false,
    )
    .await;
    assert_eq!(normal.status(), StatusCode::OK);

    let extreme = send(
        &router,
        "GET",
        "/api/v1/admin/api-clients?limit=50&offset=9223372036854775807",
        Some(&admin),
        None,
        false,
    )
    .await;
    assert_eq!(extreme.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        body(extreme).await["error"]["code"],
        "API_CLIENT_VALIDATION_FAILED"
    );

    cleanup_actor(&pool, admin.id).await;
}

#[tokio::test]
async fn validates_catalog_filters_pagination_edits_and_terminal_lifecycle() {
    let pool = test_pool().await;
    let admin = cookie(&pool, "API_ADMIN", "workflow").await;
    let router = app(pool.clone());
    let namespace = Uuid::new_v4().simple().to_string();
    let alpha_name = format!("Alpha Analytics {namespace}");
    let beta_name = format!("Beta Analytics {namespace}");
    let updated_alpha_name = format!("Alpha Flight Analytics {namespace}");

    for payload in [
        json!({"name":"Bad scope","description":null,"status":"SUSPENDED","allowedScopes":["passengers:read"]}),
        json!({"name":"No scope active","description":null,"status":"ACTIVE","allowedScopes":[]}),
        json!({"name":"Extra field","description":null,"status":"SUSPENDED","allowedScopes":[],"clientId":"XFCFORGED2222222222"}),
    ] {
        let response = send(
            &router,
            "POST",
            "/api/v1/admin/api-clients",
            Some(&admin),
            Some(payload),
            true,
        )
        .await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body(response).await["error"]["code"],
            "API_CLIENT_VALIDATION_FAILED"
        );
    }

    let first = body(
        send(
            &router,
            "POST",
            "/api/v1/admin/api-clients",
            Some(&admin),
            Some(create_payload(&alpha_name)),
            true,
        )
        .await,
    )
    .await;
    let _second = send(
        &router,
        "POST",
        "/api/v1/admin/api-clients",
        Some(&admin),
        Some(create_payload(&beta_name)),
        true,
    )
    .await;
    let page = body(
        send(
            &router,
            "GET",
            &format!(
                "/api/v1/admin/api-clients?search={namespace}&scope=analytics%3Aread&status=SUSPENDED&limit=1&offset=0"
            ),
            Some(&admin),
            None,
            false,
        )
        .await,
    )
    .await;
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert_eq!(page["nextOffset"], 1);
    assert_eq!(
        send(
            &router,
            "GET",
            "/api/v1/admin/api-clients?limit=51",
            Some(&admin),
            None,
            false,
        )
        .await
        .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let client_id = first["clientId"].as_str().unwrap();
    for payload in [
        json!({
            "name":"   ",
            "description":null,
            "allowedScopes":["analytics:read"],
            "version":first["version"]
        }),
        json!({
            "name":alpha_name,
            "description":null,
            "allowedScopes":["analytics:read"],
            "version":0
        }),
    ] {
        let response = send(
            &router,
            "PUT",
            &format!("/api/v1/admin/api-clients/{client_id}"),
            Some(&admin),
            Some(payload),
            true,
        )
        .await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body(response).await["error"]["code"],
            "API_CLIENT_VALIDATION_FAILED"
        );
    }
    let updated = body(
        send(
            &router,
            "PUT",
            &format!("/api/v1/admin/api-clients/{client_id}"),
            Some(&admin),
            Some(json!({
                "name":updated_alpha_name,
                "description":null,
                "allowedScopes":["flights:read","analytics:read"],
                "version":first["version"]
            })),
            true,
        )
        .await,
    )
    .await;
    assert_eq!(updated["clientId"], client_id);
    assert_eq!(updated["version"], 2);

    let active = body(
        send(
            &router,
            "POST",
            &format!("/api/v1/admin/api-clients/{client_id}/activate"),
            Some(&admin),
            Some(json!({"version":2})),
            true,
        )
        .await,
    )
    .await;
    assert_eq!(active["status"], "ACTIVE");
    let revoked = body(
        send(
            &router,
            "POST",
            &format!("/api/v1/admin/api-clients/{client_id}/revoke"),
            Some(&admin),
            Some(json!({"version":3})),
            true,
        )
        .await,
    )
    .await;
    assert_eq!(revoked["status"], "REVOKED");
    let forbidden_reactivation = send(
        &router,
        "POST",
        &format!("/api/v1/admin/api-clients/{client_id}/activate"),
        Some(&admin),
        Some(json!({"version":4})),
        true,
    )
    .await;
    assert_eq!(forbidden_reactivation.status(), StatusCode::CONFLICT);
    assert_eq!(
        body(forbidden_reactivation).await["error"]["code"],
        "API_CLIENT_STATUS_CONFLICT"
    );
    cleanup_actor(&pool, admin.id).await;
}
