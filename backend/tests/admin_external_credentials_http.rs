mod common;

use std::{future::Future, sync::Arc, time::Duration};

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
    application::{
        api_client::ApiClientManagement,
        external_auth::{ApiClientCredentialService, ExternalAuthService},
        staff_auth::StaffAuthService,
    },
    domain::external_api::{ExternalApiCredentialPepper, PlaintextClientSecret},
    infrastructure::{
        database::{
            prepare_test_database, SqlxApiClientRepository, SqlxExternalAuthRepository,
            SqlxSeatHoldRepository, SqlxStaffAuthRepository,
        },
        external_auth_crypto::HmacExternalCredentialCrypto,
        http::build_router,
        password::Argon2PasswordService,
    },
    state::AppState,
};

async fn setup_pool() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&common::test_database_url())
        .await
        .unwrap();
    prepare_test_database(&pool).await.unwrap();
    pool
}

async fn fixture_guard() -> common::TestFixtureLock {
    common::acquire_test_fixture_lock().await
}

async fn runtime_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&common::test_runtime_database_url())
        .await
        .unwrap()
}

// Only the temporary FLIGHT_MANAGER permission grant is shared state; this
// lock coordinates that grant with the companion API-client test.
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

fn app(pool: PgPool) -> axum::Router {
    let repository = Arc::new(SqlxSeatHoldRepository::new(pool.clone()));
    let staff_auth = StaffAuthService::new(
        Arc::new(SqlxStaffAuthRepository::new(pool.clone())),
        Argon2PasswordService::default(),
        Duration::from_secs(3600),
    )
    .unwrap();
    let external_repository = Arc::new(SqlxExternalAuthRepository::new(pool.clone()));
    let pepper = ExternalApiCredentialPepper::parse_hex(&"11".repeat(32)).unwrap();
    let crypto = Arc::new(HmacExternalCredentialCrypto::from_pepper(pepper));
    let credentials = ApiClientCredentialService::new(external_repository.clone(), crypto.clone());
    let external_auth = ExternalAuthService::new(external_repository, crypto);
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
        .with_staff_auth(staff_auth)
        .with_api_clients(ApiClientManagement::new(Arc::new(
            SqlxApiClientRepository::new(pool),
        )))
        .with_external_auth(credentials, external_auth),
    )
}

async fn staff_cookie(pool: &PgPool, role: &str, label: &str) -> (Uuid, String) {
    let actor: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash) VALUES ($1,'hash') RETURNING id",
    )
    .bind(format!(
        "{label}-{}@admin-credential-http.test",
        Uuid::new_v4()
    ))
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO staff_user_roles (staff_user_id,role_code) VALUES ($1,$2)")
        .bind(actor)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
    let raw = Sha256::digest(Uuid::new_v4().as_bytes());
    let hash: [u8; 32] = Sha256::digest(raw).into();
    sqlx::query(
        "INSERT INTO staff_sessions (staff_user_id,token_hash,expires_at)
         VALUES ($1,$2,NOW()+INTERVAL '1 hour')",
    )
    .bind(actor)
    .bind(hash.as_slice())
    .execute(pool)
    .await
    .unwrap();
    (actor, format!("x_fly_staff_session={}", hex::encode(raw)))
}

async fn cleanup_actor(pool: &PgPool, actor: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM api_client_management_audit
         WHERE api_client_id IN (SELECT id FROM api_clients WHERE created_by_staff_user_id=$1)",
    )
    .bind(actor)
    .execute(pool)
    .await?;
    sqlx::query(
        "DELETE FROM external_access_tokens
         WHERE api_client_credential_id IN (
             SELECT credential.id FROM api_client_credentials credential
             JOIN api_clients client ON client.id=credential.api_client_id
             WHERE client.created_by_staff_user_id=$1
         )",
    )
    .bind(actor)
    .execute(pool)
    .await?;
    sqlx::query(
        "DELETE FROM api_client_credentials
         WHERE api_client_id IN (SELECT id FROM api_clients WHERE created_by_staff_user_id=$1)",
    )
    .bind(actor)
    .execute(pool)
    .await?;
    sqlx::query(
        "DELETE FROM api_client_allowed_scopes
         WHERE api_client_id IN (SELECT id FROM api_clients WHERE created_by_staff_user_id=$1)",
    )
    .bind(actor)
    .execute(pool)
    .await?;
    sqlx::query("DELETE FROM api_clients WHERE created_by_staff_user_id=$1")
        .bind(actor)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM staff_sessions WHERE staff_user_id=$1")
        .bind(actor)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM staff_user_roles WHERE staff_user_id=$1")
        .bind(actor)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM staff_users WHERE id=$1")
        .bind(actor)
        .execute(pool)
        .await?;
    Ok(())
}

async fn run_fixture_body<F, Fut>(setup: &PgPool, actors: Vec<Uuid>, body: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let cleanup_setup = setup.clone();
    common::run_fixture_body_with_cleanup(body, move || async move {
        let mut errors = Vec::new();
        for actor in actors {
            if let Err(error) = cleanup_actor(&cleanup_setup, actor).await {
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

async fn create_client(
    router: &axum::Router,
    cookie: &str,
    name: &str,
    status: &str,
    scopes: Value,
) -> Value {
    let unique_name = format!("{name} {}", Uuid::new_v4().simple());
    let response = send(
        router,
        "POST",
        "/api/v1/admin/api-clients",
        Some(cookie),
        Some(json!({
            "name": unique_name,
            "description": "Credential administration integration",
            "status": status,
            "allowedScopes": scopes,
        })),
        true,
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    body(response).await
}

async fn issue(
    router: &axum::Router,
    cookie: &str,
    client_id: &str,
    version: i64,
) -> axum::response::Response {
    send(
        router,
        "POST",
        &format!("/api/v1/admin/api-clients/{client_id}/credentials"),
        Some(cookie),
        Some(json!({"version": version})),
        true,
    )
    .await
}

fn assert_secret_is_canonical(value: &str) {
    assert!(PlaintextClientSecret::parse_hex(value).is_ok());
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

#[tokio::test]
async fn issue_requires_api_clients_manage_and_csrf_origin() {
    let _role_lock = role_permission_fixture_lock().await;
    let _fixture_lock = fixture_guard().await;
    let setup = setup_pool().await;
    remove_stale_role_permission_fixtures(&setup).await;
    let runtime = runtime_pool().await;
    let (actor, admin_cookie) = staff_cookie(&setup, "API_ADMIN", "issue-auth").await;
    let (system_actor, system_cookie) = staff_cookie(&setup, "SYSTEM_ADMIN", "issue-system").await;
    let (flight_actor, flight_cookie) =
        staff_cookie(&setup, "FLIGHT_MANAGER", "issue-flight").await;
    run_fixture_body(
        &setup,
        vec![actor, system_actor, flight_actor],
        move || async move {
            let router = app(runtime);

            let unauthenticated = send(
                &router,
                "POST",
                "/api/v1/admin/api-clients/XFCAAAAAAAAAAAAAAAA/credentials",
                None,
                Some(json!({"version": 1})),
                true,
            )
            .await;
            assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

            for cookie in [&system_cookie, &flight_cookie] {
                let denied = send(
                    &router,
                    "POST",
                    "/api/v1/admin/api-clients/XFCAAAAAAAAAAAAAAAA/credentials",
                    Some(cookie),
                    Some(json!({"version": 1})),
                    true,
                )
                .await;
                assert_eq!(denied.status(), StatusCode::FORBIDDEN);
            }

            let response = send(
                &router,
                "POST",
                "/api/v1/admin/api-clients/XFCAAAAAAAAAAAAAAAA/credentials",
                Some(&admin_cookie),
                Some(json!({"version": 1})),
                false,
            )
            .await;

            assert_eq!(response.status(), StatusCode::FORBIDDEN);
            assert_eq!(
                response.headers()[header::CACHE_CONTROL],
                "no-store, private"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn issue_returns_secret_once_and_safe_metadata() {
    let _fixture_lock = fixture_guard().await;
    let setup = setup_pool().await;
    let runtime = runtime_pool().await;
    let (actor, admin_cookie) = staff_cookie(&setup, "API_ADMIN", "issue-success").await;
    run_fixture_body(&setup, vec![actor], move || async move {
        let router = app(runtime);
        let created = create_client(
            &router,
            &admin_cookie,
            "Credential issue client",
            "SUSPENDED",
            json!([]),
        )
        .await;
        let client_id = created["clientId"].as_str().unwrap().to_owned();

        let response = issue(
            &router,
            &admin_cookie,
            &client_id,
            created["version"].as_i64().unwrap(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "no-store, private"
        );
        let issued = body(response).await;
        assert_eq!(issued["clientId"], client_id);
        assert_secret_is_canonical(issued["clientSecret"].as_str().unwrap());
        assert!(issued["issuedAt"].is_string());
        let serialized = issued.to_string();
        assert!(!serialized.contains("credentialId"));
        assert!(!serialized.contains("secretDigest"));
        assert!(!serialized.contains("tokenHash"));

        let detail_response = send(
            &router,
            "GET",
            &format!("/api/v1/admin/api-clients/{client_id}"),
            Some(&admin_cookie),
            None,
            false,
        )
        .await;
        assert_eq!(detail_response.status(), StatusCode::OK);
        let detail = body(detail_response).await;
        assert_eq!(detail["credentialMetadata"]["hasLiveCredential"], true);
        assert!(detail["credentialMetadata"]["issuedAt"].is_string());
        assert!(detail["credentialMetadata"]["revokedAt"].is_null());
        let detail_text = detail.to_string();
        assert!(!detail_text.contains(issued["clientSecret"].as_str().unwrap()));
        assert!(!detail_text.contains("secretDigest"));
        assert!(!detail_text.contains("credentialId"));
    })
    .await;
}

#[tokio::test]
async fn detail_never_returns_digest_or_secret() {
    let _fixture_lock = fixture_guard().await;
    let setup = setup_pool().await;
    let runtime = runtime_pool().await;
    let (actor, admin_cookie) = staff_cookie(&setup, "API_ADMIN", "detail-secret-free").await;
    run_fixture_body(&setup, vec![actor], move || async move {
        let router = app(runtime);
        let created = create_client(
            &router,
            &admin_cookie,
            "Secret-free detail client",
            "SUSPENDED",
            json!([]),
        )
        .await;
        let client_id = created["clientId"].as_str().unwrap().to_owned();
        let issued = body(issue(&router, &admin_cookie, &client_id, 1).await).await;
        assert_eq!(issued["clientId"], client_id);
        let response = send(
            &router,
            "GET",
            &format!("/api/v1/admin/api-clients/{client_id}"),
            Some(&admin_cookie),
            None,
            false,
        )
        .await;
        let detail = body(response).await;
        let text = detail.to_string();
        assert!(!text.contains(issued["clientSecret"].as_str().unwrap()));
        for forbidden in [
            "credentialId",
            "secretDigest",
            "tokenHash",
            "revocationReason",
        ] {
            assert!(!text.contains(forbidden), "detail must omit {forbidden}");
        }
    })
    .await;
}

#[tokio::test]
async fn revoke_requires_current_version() {
    let _fixture_lock = fixture_guard().await;
    let setup = setup_pool().await;
    let runtime = runtime_pool().await;
    let (actor, admin_cookie) = staff_cookie(&setup, "API_ADMIN", "revoke-version").await;
    run_fixture_body(&setup, vec![actor], move || async move {
        let router = app(runtime);
        let created = create_client(
            &router,
            &admin_cookie,
            "Credential revoke version",
            "SUSPENDED",
            json!([]),
        )
        .await;
        let client_id = created["clientId"].as_str().unwrap().to_owned();
        let _issued = issue(&router, &admin_cookie, &client_id, 1).await;

        let stale = send(
            &router,
            "POST",
            &format!("/api/v1/admin/api-clients/{client_id}/credentials/revoke"),
            Some(&admin_cookie),
            Some(json!({"version": 1})),
            true,
        )
        .await;
        assert_eq!(stale.status(), StatusCode::CONFLICT);
        assert_eq!(
            body(stale).await["error"]["code"],
            "API_CLIENT_STALE_VERSION"
        );

        let current = send(
            &router,
            "POST",
            &format!("/api/v1/admin/api-clients/{client_id}/credentials/revoke"),
            Some(&admin_cookie),
            Some(json!({"version": 2})),
            true,
        )
        .await;
        assert_eq!(current.status(), StatusCode::OK);
        let revoked = body(current).await;
        assert_eq!(revoked["hasLiveCredential"], false);
        assert!(revoked["revokedAt"].is_string());
    })
    .await;
}

#[tokio::test]
async fn admin_revoke_cannot_supply_system_reason() {
    let _fixture_lock = fixture_guard().await;
    let setup = setup_pool().await;
    let runtime = runtime_pool().await;
    let (actor, admin_cookie) = staff_cookie(&setup, "API_ADMIN", "revoke-reason").await;
    run_fixture_body(&setup, vec![actor], move || async move {
        let router = app(runtime);
        let response = send(
            &router,
            "POST",
            "/api/v1/admin/api-clients/XFCAAAAAAAAAAAAAAAA/credentials/revoke",
            Some(&admin_cookie),
            Some(json!({"version": 1, "reason": "CLIENT_REVOKED"})),
            true,
        )
        .await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body(response).await["error"]["code"],
            "API_CLIENT_VALIDATION_FAILED"
        );
    })
    .await;
}

#[tokio::test]
async fn revoke_is_audited_with_credential_context() {
    let _fixture_lock = fixture_guard().await;
    let setup = setup_pool().await;
    let runtime = runtime_pool().await;
    let (actor, admin_cookie) = staff_cookie(&setup, "API_ADMIN", "revoke-audit").await;
    let setup_for_body = setup.clone();
    run_fixture_body(&setup, vec![actor], move || async move {
        let router = app(runtime);
        let created = create_client(
            &router,
            &admin_cookie,
            "Credential revoke audit",
            "SUSPENDED",
            json!([]),
        )
        .await;
        let client_id = created["clientId"].as_str().unwrap().to_owned();
        let _issued = issue(&router, &admin_cookie, &client_id, 1).await;
        let revoked = send(
            &router,
            "POST",
            &format!("/api/v1/admin/api-clients/{client_id}/credentials/revoke"),
            Some(&admin_cookie),
            Some(json!({"version": 2})),
            true,
        )
        .await;
        assert_eq!(revoked.status(), StatusCode::OK);
        let audit: (i64, i64) = sqlx::query_as(
            "SELECT COUNT(*) FILTER (WHERE action='CREDENTIAL_ISSUED'),
                    COUNT(*) FILTER (WHERE action='CREDENTIAL_REVOKED')
             FROM api_client_management_audit audit
             JOIN api_clients client ON client.id=audit.api_client_id
             WHERE client.client_id=$1 AND audit.actor_staff_user_id=$2
               AND audit.credential_id IS NOT NULL",
        )
        .bind(client_id)
        .bind(actor)
        .fetch_one(&setup_for_body)
        .await
        .unwrap();
        assert_eq!(audit, (1, 1));
    })
    .await;
}

#[tokio::test]
async fn client_suspend_and_revoke_audit_correlates_credential_when_present() {
    let _fixture_lock = fixture_guard().await;
    let setup = setup_pool().await;
    let runtime = runtime_pool().await;
    let (actor, admin_cookie) = staff_cookie(&setup, "API_ADMIN", "suspend-audit").await;
    let setup_for_body = setup.clone();
    run_fixture_body(&setup, vec![actor], move || async move {
        let router = app(runtime);
        let created = create_client(
            &router,
            &admin_cookie,
            "Lifecycle credential audit",
            "ACTIVE",
            json!(["analytics:read"]),
        )
        .await;
        let client_id = created["clientId"].as_str().unwrap().to_owned();
        let _issued = issue(&router, &admin_cookie, &client_id, 1).await;
        let suspended = send(
            &router,
            "POST",
            &format!("/api/v1/admin/api-clients/{client_id}/suspend"),
            Some(&admin_cookie),
            Some(json!({"version": 2})),
            true,
        )
        .await;
        assert_eq!(suspended.status(), StatusCode::OK);
        let correlated: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM api_client_management_audit audit
             JOIN api_clients client ON client.id=audit.api_client_id
             WHERE client.client_id=$1 AND audit.action='CREDENTIAL_REVOKED'
               AND audit.actor_staff_user_id=$2 AND audit.credential_id IS NOT NULL",
        )
        .bind(client_id)
        .bind(actor)
        .fetch_one(&setup_for_body)
        .await
        .unwrap();
        assert_eq!(correlated, 1);
    })
    .await;
}

#[tokio::test]
async fn revoked_client_cannot_issue() {
    let _fixture_lock = fixture_guard().await;
    let setup = setup_pool().await;
    let runtime = runtime_pool().await;
    let (actor, admin_cookie) = staff_cookie(&setup, "API_ADMIN", "revoked-client").await;
    run_fixture_body(&setup, vec![actor], move || async move {
        let router = app(runtime);
        let created = create_client(
            &router,
            &admin_cookie,
            "Terminal credential client",
            "SUSPENDED",
            json!([]),
        )
        .await;
        let client_id = created["clientId"].as_str().unwrap().to_owned();
        let revoked = send(
            &router,
            "POST",
            &format!("/api/v1/admin/api-clients/{client_id}/revoke"),
            Some(&admin_cookie),
            Some(json!({"version": 1})),
            true,
        )
        .await;
        assert_eq!(revoked.status(), StatusCode::OK);
        let issue_response = issue(&router, &admin_cookie, &client_id, 2).await;
        assert_eq!(issue_response.status(), StatusCode::CONFLICT);
        assert_eq!(
            body(issue_response).await["error"]["code"],
            "API_CLIENT_STATUS_CONFLICT"
        );
    })
    .await;
}

#[tokio::test]
async fn existing_branch24_audit_actions_still_render() {
    let _fixture_lock = fixture_guard().await;
    let setup = setup_pool().await;
    let runtime = runtime_pool().await;
    let (actor, admin_cookie) = staff_cookie(&setup, "API_ADMIN", "audit-actions").await;
    run_fixture_body(&setup, vec![actor], move || async move {
        let router = app(runtime);
        let created = create_client(
            &router,
            &admin_cookie,
            "Audit action compatibility",
            "SUSPENDED",
            json!(["analytics:read"]),
        )
        .await;
        let client_id = created["clientId"].as_str().unwrap().to_owned();
        let updated_name = format!(
            "Audit action compatibility updated {}",
            Uuid::new_v4().simple()
        );
        let updated = send(
            &router,
            "PUT",
            &format!("/api/v1/admin/api-clients/{client_id}"),
            Some(&admin_cookie),
            Some(json!({
                "name": updated_name,
                "description": null,
                "allowedScopes": ["analytics:read"],
                "version": 1
            })),
            true,
        )
        .await;
        assert_eq!(updated.status(), StatusCode::OK);
        let detail = send(
            &router,
            "GET",
            &format!("/api/v1/admin/api-clients/{client_id}"),
            Some(&admin_cookie),
            None,
            false,
        )
        .await;
        assert_eq!(detail.status(), StatusCode::OK);
        let detail = body(detail).await;
        let actions = detail["audit"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|entry| entry["action"].as_str())
            .collect::<Vec<_>>();
        assert!(actions.contains(&"CLIENT_CREATED"));
        assert!(actions.contains(&"CLIENT_METADATA_UPDATED"));
    })
    .await;
}
