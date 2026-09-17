mod common;

use std::{
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use async_trait::async_trait;
use axum::{
    body::Body,
    extract::Extension,
    http::{header, HeaderMap, HeaderValue, Request, StatusCode},
    response::Json,
    routing::get,
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::ServiceExt;
use uuid::Uuid;

use x_fly_api::{
    application::external_auth::{
        ExternalAuthRepository, ExternalAuthService, ExternalCredentialCrypto,
    },
    domain::{
        api_client::ApiClientScope,
        external_api::{
            AccessTokenHash, CredentialAdministrationError, CredentialDigest, CredentialMetadata,
            CredentialRevocationReason, ExternalApiCredentialPepper, ExternalAuthenticationError,
            ExternalPrincipal, ExternalTokenExchangeError, PlaintextAccessToken,
            PlaintextClientSecret,
        },
    },
    infrastructure::{
        database::{
            migrate_database, verify_database_ready, SqlxExternalAuthRepository,
            SqlxSeatHoldRepository,
        },
        external_auth_crypto::HmacExternalCredentialCrypto,
        http::{build_router, external::ExternalBearerAuthLayer},
    },
    state::AppState,
};

const PEPPER: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

struct Fixture {
    client_pk: Uuid,
    client_id: String,
    actor_id: Uuid,
    secret_text: String,
    credential_id: Uuid,
}

#[derive(Clone)]
struct CaptureWriter(Arc<std::sync::Mutex<Vec<u8>>>);

impl io::Write for CaptureWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("capture writer lock")
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct CountingRepository {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl ExternalAuthRepository for CountingRepository {
    async fn issue_credential(
        &self,
        _client_id: &str,
        _actor_staff_user_id: Uuid,
        _expected_client_version: i64,
        _digest: &CredentialDigest,
        _issued_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<CredentialMetadata, CredentialAdministrationError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(CredentialAdministrationError::Infrastructure)
    }

    async fn revoke_credential(
        &self,
        _client_id: &str,
        _actor_staff_user_id: Uuid,
        _expected_client_version: i64,
        _reason: CredentialRevocationReason,
        _revoked_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<CredentialMetadata, CredentialAdministrationError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(CredentialAdministrationError::Infrastructure)
    }

    async fn issue_access_token(
        &self,
        _client_id: &str,
        _digest: &CredentialDigest,
        _token_hash: &AccessTokenHash,
        _issued_at: chrono::DateTime<chrono::Utc>,
        _expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ExternalTokenExchangeError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(ExternalTokenExchangeError::Unavailable)
    }

    async fn authenticate_access_token(
        &self,
        _token_hash: &AccessTokenHash,
        _now: chrono::DateTime<chrono::Utc>,
    ) -> Result<ExternalPrincipal, ExternalAuthenticationError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(ExternalAuthenticationError::Unavailable)
    }
}

async fn cleanup_fixture(setup: &PgPool, fixture: &Fixture) -> Result<(), sqlx::Error> {
    let mut transaction = setup.begin().await?;
    sqlx::query(
        "DELETE FROM external_access_tokens
         WHERE api_client_credential_id IN (
             SELECT id FROM api_client_credentials WHERE api_client_id=$1
         )",
    )
    .bind(fixture.client_pk)
    .execute(&mut *transaction)
    .await?;
    sqlx::query("DELETE FROM api_client_management_audit WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM api_client_credentials WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM api_clients WHERE id=$1")
        .bind(fixture.client_pk)
        .execute(&mut *transaction)
        .await?;
    sqlx::query("DELETE FROM staff_users WHERE id=$1")
        .bind(fixture.actor_id)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(())
}

async fn pools() -> (PgPool, PgPool) {
    let setup = PgPoolOptions::new()
        .max_connections(8)
        .connect(&common::test_database_url())
        .await
        .expect("connect to TEST setup database");
    migrate_database(&setup)
        .await
        .expect("TEST migration chain is current");
    let runtime = PgPoolOptions::new()
        .max_connections(8)
        .connect(&common::test_runtime_database_url())
        .await
        .expect("connect to TEST runtime database");
    verify_database_ready(&runtime)
        .await
        .expect("runtime sees ready TEST schema");
    (setup, runtime)
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

fn crypto() -> Arc<HmacExternalCredentialCrypto> {
    Arc::new(HmacExternalCredentialCrypto::from_pepper(
        ExternalApiCredentialPepper::parse_hex(PEPPER).expect("test pepper"),
    ))
}

async fn fixture(setup: &PgPool, scopes: &[ApiClientScope], status: &str) -> Fixture {
    let mut transaction = setup.begin().await.expect("begin external token fixture");
    let actor_id: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash) VALUES ($1,'external-token-http') RETURNING id",
    )
    .bind(format!("external-token-{}@test.invalid", Uuid::new_v4()))
    .fetch_one(&mut *transaction)
    .await
    .expect("fixture actor");
    let client_id = unique_client_id();
    let client_pk: Uuid = sqlx::query_scalar(
        "INSERT INTO api_clients (
             client_id,display_name,description,status,
             created_by_staff_user_id,updated_by_staff_user_id
         ) VALUES ($1,$2,NULL,$3,$4,$4) RETURNING id",
    )
    .bind(&client_id)
    .bind(format!("External token {}", Uuid::new_v4()))
    .bind(status)
    .bind(actor_id)
    .fetch_one(&mut *transaction)
    .await
    .expect("fixture client");
    for scope in scopes {
        sqlx::query(
            "INSERT INTO api_client_allowed_scopes
             (api_client_id,scope_code,assigned_by_staff_user_id)
             VALUES ($1,$2,$3)",
        )
        .bind(client_pk)
        .bind(scope.as_str())
        .bind(actor_id)
        .execute(&mut *transaction)
        .await
        .expect("fixture scope");
    }
    let secret = crypto().generate_client_secret();
    let secret_text = hex::encode(secret.as_bytes());
    let digest = crypto().credential_digest(
        &client_id,
        &PlaintextClientSecret::parse_hex(&secret_text).expect("fixture secret"),
    );
    let credential_id: Uuid = sqlx::query_scalar(
        "INSERT INTO api_client_credentials
         (api_client_id,secret_digest,digest_version,issued_at,issued_by_staff_user_id)
         VALUES ($1,$2,1,clock_timestamp(),$3)
         RETURNING id",
    )
    .bind(client_pk)
    .bind(digest.as_bytes().as_slice())
    .bind(actor_id)
    .fetch_one(&mut *transaction)
    .await
    .expect("fixture credential");
    transaction
        .commit()
        .await
        .expect("commit external token fixture");
    Fixture {
        client_pk,
        client_id,
        actor_id,
        secret_text,
        credential_id,
    }
}

async fn run_fixture_test<F, Fut>(scopes: Vec<ApiClientScope>, status: &'static str, body: F)
where
    F: FnOnce(PgPool, PgPool, Fixture) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &scopes, status).await;
    let cleanup_setup = setup.clone();
    let cleanup_target = fixture.clone();
    common::run_fixture_body_with_cleanup(
        move || body(setup, runtime, fixture),
        move || async move { cleanup_fixture(&cleanup_setup, &cleanup_target).await },
    )
    .await;
}

async fn run_fixtures_test<F, Fut>(
    specifications: Vec<(Vec<ApiClientScope>, &'static str)>,
    body: F,
) where
    F: FnOnce(PgPool, PgPool, Vec<Fixture>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let mut fixtures = Vec::with_capacity(specifications.len());
    for (scopes, status) in specifications {
        fixtures.push(fixture(&setup, &scopes, status).await);
    }
    let cleanup_setup = setup.clone();
    let cleanup_fixtures = fixtures.clone();
    common::run_fixture_body_with_cleanup(
        move || body(setup, runtime, fixtures),
        move || async move {
            let mut errors = Vec::new();
            for fixture in &cleanup_fixtures {
                if let Err(error) = cleanup_fixture(&cleanup_setup, fixture).await {
                    errors.push(error.to_string());
                }
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors.join("; "))
            }
        },
    )
    .await;
}

fn app(runtime: PgPool, with_auth: bool) -> Router {
    let booking = Arc::new(SqlxSeatHoldRepository::new(runtime.clone()));
    let state = AppState::new(
        booking.clone(),
        booking.clone(),
        booking.clone(),
        booking,
        Duration::from_secs(600),
        true,
        "http://localhost:3000".to_owned(),
    );
    if with_auth {
        let repository = Arc::new(SqlxExternalAuthRepository::new(runtime));
        let crypto = crypto();
        let service = ExternalAuthService::new(repository.clone(), crypto.clone());
        build_router(state.with_external_auth(
            x_fly_api::application::external_auth::ApiClientCredentialService::new(
                repository, crypto,
            ),
            service,
        ))
    } else {
        build_router(state)
    }
}

fn app_with_counting_auth(runtime: PgPool, calls: Arc<AtomicUsize>) -> Router {
    let booking = Arc::new(SqlxSeatHoldRepository::new(runtime));
    let repository: Arc<dyn ExternalAuthRepository> = Arc::new(CountingRepository { calls });
    let crypto = crypto();
    let service = ExternalAuthService::new(repository.clone(), crypto.clone());
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
        .with_external_auth(
            x_fly_api::application::external_auth::ApiClientCredentialService::new(
                repository, crypto,
            ),
            service,
        ),
    )
}

fn state_with_auth(runtime: PgPool) -> AppState {
    let booking = Arc::new(SqlxSeatHoldRepository::new(runtime.clone()));
    let repository = Arc::new(SqlxExternalAuthRepository::new(runtime));
    let crypto = crypto();
    let service = ExternalAuthService::new(repository.clone(), crypto.clone());
    AppState::new(
        booking.clone(),
        booking.clone(),
        booking.clone(),
        booking,
        Duration::from_secs(600),
        true,
        "http://localhost:3000".to_owned(),
    )
    .with_external_auth(
        x_fly_api::application::external_auth::ApiClientCredentialService::new(repository, crypto),
        service,
    )
}

async fn response_body(response: axum::response::Response) -> (StatusCode, HeaderMap, Value) {
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, body)
}

async fn send_token(
    router: &Router,
    raw_body: &str,
    origin: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/v1/external/token")
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(origin) = origin {
        builder = builder.header(header::ORIGIN, origin);
    }
    let response = router
        .clone()
        .oneshot(
            builder
                .body(Body::from(raw_body.to_owned()))
                .expect("token request"),
        )
        .await
        .expect("token response");
    response_body(response).await
}

async fn send_token_captured(
    router: &Router,
    raw_body: &str,
    origin: Option<&str>,
) -> ((StatusCode, HeaderMap, Value), String) {
    let output = Arc::new(std::sync::Mutex::new(Vec::new()));
    let writer_output = Arc::clone(&output);
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(move || CaptureWriter(Arc::clone(&writer_output)))
        .finish();
    let guard = tracing::subscriber::set_default(subscriber);
    let response = send_token(router, raw_body, origin).await;
    drop(guard);
    let output = String::from_utf8(output.lock().expect("capture output lock").clone())
        .expect("trace output is UTF-8");
    (response, output)
}

fn token_request(fixture: &Fixture) -> String {
    json!({"clientId": fixture.client_id, "clientSecret": fixture.secret_text}).to_string()
}

async fn exchange(router: &Router, fixture: &Fixture) -> String {
    let (status, _, body) = send_token(router, &token_request(fixture), None).await;
    assert_eq!(status, StatusCode::OK);
    body["accessToken"]
        .as_str()
        .expect("access token")
        .to_owned()
}

async fn mark_client_status(setup: &PgPool, fixture: &Fixture, status: &str) {
    sqlx::query("UPDATE api_clients SET status=$2 WHERE id=$1")
        .bind(fixture.client_pk)
        .bind(status)
        .execute(setup)
        .await
        .expect("fixture status update");
}

async fn revoke_fixture_credential(setup: &PgPool, fixture: &Fixture) {
    sqlx::query(
        "UPDATE api_client_credentials
         SET revoked_at=clock_timestamp(),revoked_by_staff_user_id=$2,
             revocation_reason='ADMIN_REQUEST'
         WHERE id=$1",
    )
    .bind(fixture.credential_id)
    .bind(fixture.actor_id)
    .execute(setup)
    .await
    .expect("fixture credential revocation");
}

async fn insert_expired_token(setup: &PgPool, fixture: &Fixture) -> String {
    let token = crypto().generate_access_token();
    let token_text = format!("xfa_v1_{}", hex::encode(token.as_bytes()));
    let hash = crypto().access_token_hash(&token);
    sqlx::query(
        "INSERT INTO external_access_tokens
         (api_client_credential_id,token_hash,issued_at,expires_at)
         VALUES ($1,$2,clock_timestamp()-INTERVAL '2 hours',clock_timestamp()-INTERVAL '1 hour')",
    )
    .bind(fixture.credential_id)
    .bind(hash.as_bytes().as_slice())
    .execute(setup)
    .await
    .expect("expired token fixture");
    token_text
}

async fn assert_client_exchange_failure(router: &Router, fixture: &Fixture) -> Value {
    let (status, _, body) = send_token(router, &token_request(fixture), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_external_error(&body, "EXTERNAL_CLIENT_AUTHENTICATION_FAILED");
    body
}

async fn probe(Extension(principal): Extension<ExternalPrincipal>) -> Json<Value> {
    Json(json!({
        "clientId": principal.client_id(),
        "scopeCount": principal.scopes().len(),
        "hasFlightsRead": principal.scopes().contains(&ApiClientScope::FlightsRead),
    }))
}

fn protected_app(state: AppState) -> Router {
    let routes: Router<AppState> = Router::new().route("/probe", get(probe));
    ExternalBearerAuthLayer::new(state.clone())
        .layer(routes)
        .with_state(state)
}

async fn send_protected(
    router: &Router,
    values: &[&str],
    cookies: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method("GET").uri("/probe");
    for value in values {
        builder = builder.header(header::AUTHORIZATION, *value);
    }
    if let Some(cookies) = cookies {
        builder = builder.header(header::COOKIE, cookies);
    }
    let response = router
        .clone()
        .oneshot(builder.body(Body::empty()).expect("protected request"))
        .await
        .expect("protected response");
    response_body(response).await
}

async fn send_protected_captured(
    router: &Router,
    values: &[&str],
    cookies: Option<&str>,
) -> ((StatusCode, HeaderMap, Value), String) {
    let output = Arc::new(std::sync::Mutex::new(Vec::new()));
    let writer_output = Arc::clone(&output);
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(move || CaptureWriter(Arc::clone(&writer_output)))
        .finish();
    let guard = tracing::subscriber::set_default(subscriber);
    let response = send_protected(router, values, cookies).await;
    drop(guard);
    let output = String::from_utf8(output.lock().expect("capture output lock").clone())
        .expect("trace output is UTF-8");
    (response, output)
}

fn assert_external_error(body: &Value, code: &str) {
    assert_eq!(body["error"]["code"].as_str(), Some(code));
    assert!(body["error"]["message"].is_string());
    assert!(body["error"]["requestId"].as_str().is_some());
}

async fn assert_structural_invalid_before_auth(raw_body: &str) {
    let (_, runtime) = pools().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let router = app_with_counting_auth(runtime, calls.clone());
    let (status, _, body) = send_token(&router, raw_body, None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_external_error(&body, "EXTERNAL_REQUEST_INVALID");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

const VALID_REQUEST_SECRET: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[tokio::test]
async fn token_exchange_sets_no_store_and_pragma() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |_setup, runtime, fixture| async move {
            let router = app(runtime, true);
            let (status, headers, _) = send_token(&router, &token_request(&fixture), None).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(headers[header::CACHE_CONTROL], "no-store, private");
            assert_eq!(headers[header::PRAGMA], "no-cache");
        },
    )
    .await;
}

#[tokio::test]
async fn token_exchange_persists_only_verifiers_with_a_900_second_lifetime() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |setup, runtime, fixture| async move {
            let router = app(runtime, true);
            let token_text = exchange(&router, &fixture).await;
            let token =
                PlaintextAccessToken::parse_bearer_text(&token_text).expect("generated token");
            let (stored_hash, issued_at, expires_at): (
                Vec<u8>,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
            ) = sqlx::query_as(
                "SELECT token_hash,issued_at,expires_at
                     FROM external_access_tokens
                     WHERE api_client_credential_id=$1
                     ORDER BY issued_at DESC
                     LIMIT 1",
            )
            .bind(fixture.credential_id)
            .fetch_one(&setup)
            .await
            .expect("stored token verifier");
            assert_eq!(stored_hash, crypto().access_token_hash(&token).as_bytes());
            assert_eq!((expires_at - issued_at).num_seconds(), 900);
            let stored_digest: Vec<u8> =
                sqlx::query_scalar("SELECT secret_digest FROM api_client_credentials WHERE id=$1")
                    .bind(fixture.credential_id)
                    .fetch_one(&setup)
                    .await
                    .expect("stored credential verifier");
            let secret =
                PlaintextClientSecret::parse_hex(&fixture.secret_text).expect("fixture secret");
            assert_eq!(
                stored_digest,
                crypto()
                    .credential_digest(&fixture.client_id, &secret)
                    .as_bytes()
            );
        },
    )
    .await;
}

#[tokio::test]
async fn malformed_json_returns_400_external_request_invalid() {
    assert_structural_invalid_before_auth("{").await;
}

#[tokio::test]
async fn external_error_request_id_matches_the_response_request_id() {
    let (_, runtime) = pools().await;
    let router = app(runtime, false);
    let (status, headers, body) = send_token(&router, "{", None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let response_id = headers["x-request-id"].to_str().expect("request id header");
    assert_eq!(body["error"]["requestId"].as_str(), Some(response_id));
}

#[tokio::test]
async fn missing_client_id_returns_400_external_request_invalid() {
    assert_structural_invalid_before_auth(
        &json!({"clientSecret": VALID_REQUEST_SECRET}).to_string(),
    )
    .await;
}

#[tokio::test]
async fn missing_client_secret_returns_400_external_request_invalid() {
    assert_structural_invalid_before_auth(&json!({"clientId":"XFCABCDEFGHJKLMNP22"}).to_string())
        .await;
}

#[tokio::test]
async fn wrong_client_id_type_returns_400_external_request_invalid() {
    assert_structural_invalid_before_auth(
        &json!({"clientId":42,"clientSecret":VALID_REQUEST_SECRET}).to_string(),
    )
    .await;
}

#[tokio::test]
async fn wrong_client_secret_type_returns_400_external_request_invalid() {
    assert_structural_invalid_before_auth(
        &json!({"clientId":"XFCABCDEFGHJKLMNP22","clientSecret":42}).to_string(),
    )
    .await;
}

#[tokio::test]
async fn unknown_token_request_field_returns_400_external_request_invalid() {
    assert_structural_invalid_before_auth(
        &json!({"clientId":"XFCABCDEFGHJKLMNP22","clientSecret":VALID_REQUEST_SECRET,"grantType":"client_credentials"}).to_string(),
    )
    .await;
}

#[tokio::test]
async fn caller_scope_field_returns_400_external_request_invalid() {
    assert_structural_invalid_before_auth(
        &json!({"clientId":"XFCABCDEFGHJKLMNP22","clientSecret":VALID_REQUEST_SECRET,"scope":"flights:read"}).to_string(),
    )
    .await;
}

#[tokio::test]
async fn missing_external_auth_service_on_bearer_route_fails_closed() {
    let (_, runtime) = pools().await;
    let booking = Arc::new(SqlxSeatHoldRepository::new(runtime));
    let state = AppState::new(
        booking.clone(),
        booking.clone(),
        booking.clone(),
        booking,
        Duration::from_secs(600),
        true,
        "http://localhost:3000".to_owned(),
    );
    let protected = protected_app(state);
    let (status, headers, body) = send_protected(
        &protected,
        &["Bearer xfa_v1_0000000000000000000000000000000000000000000000000000000000000000"],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
    assert_external_error(&body, "EXTERNAL_AUTH_UNAVAILABLE");
}

#[tokio::test]
async fn token_exchange_returns_no_scope_snapshot() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |_setup, runtime, fixture| async move {
            let router = app(runtime, true);
            let (status, headers, body) = send_token(&router, &token_request(&fixture), None).await;

            assert_eq!(status, StatusCode::OK);
            assert_eq!(headers[header::CACHE_CONTROL], "no-store, private");
            assert_eq!(headers[header::PRAGMA], "no-cache");
            assert_eq!(body["tokenType"], "Bearer");
            assert_eq!(body["expiresIn"], 900);
            assert!(body.get("scopes").is_none());
            let keys = body
                .as_object()
                .expect("token response object")
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            assert_eq!(keys, vec!["accessToken", "expiresIn", "tokenType"]);
            let token = body["accessToken"].as_str().expect("access token");
            assert!(PlaintextAccessToken::parse_bearer_text(token).is_ok());
        },
    )
    .await;
}

#[tokio::test]
async fn malformed_json_returns_external_request_invalid_without_auth_service() {
    let (_, runtime) = pools().await;
    let router = app(runtime, false);
    let (status, _, body) = send_token(&router, "{", None).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_external_error(&body, "EXTERNAL_REQUEST_INVALID");
}

#[tokio::test]
async fn strict_token_request_rejects_missing_wrong_type_and_unknown_fields() {
    let (_, runtime) = pools().await;
    let router = app(runtime, false);
    for body in [
        r#"{"clientSecret":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}"#,
        r#"{"clientId":"XFCABCDEFGHJKLMNP22"}"#,
        r#"{"clientId":42,"clientSecret":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}"#,
        r#"{"clientId":"XFCABCDEFGHJKLMNP22","clientSecret":42}"#,
        r#"{"clientId":"XFCABCDEFGHJKLMNP22","clientSecret":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef","scopes":["flights:read"]}"#,
    ] {
        let (status, _, body) = send_token(&router, body, None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_external_error(&body, "EXTERNAL_REQUEST_INVALID");
    }
}

#[tokio::test]
async fn noncanonical_and_oversized_token_request_values_are_invalid_before_authentication() {
    let (_, runtime) = pools().await;
    let router = app(runtime, false);
    let cases = [
        json!({"clientId":" xfcABCDEFGHJKLMNP22","clientSecret":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}),
        json!({"clientId":"XFCABCDEFGHJKLMNP22","clientSecret":"0123456789ABCDEF0123456789abcdef0123456789abcdef0123456789abcdef"}),
        json!({"clientId":"XFCABCDEFGHJKLMNP222222222222222222222222222222222222222222222222222222","clientSecret":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}),
        json!({"clientId":"XFCABCDEFGHJKLMNP22","clientSecret":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef00"}),
    ];
    for value in cases {
        let (status, _, body) = send_token(&router, &value.to_string(), None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_external_error(&body, "EXTERNAL_REQUEST_INVALID");
    }
}

#[tokio::test]
async fn token_exchange_authentication_failures_use_one_generic_envelope() {
    run_fixtures_test(
        vec![
            (vec![ApiClientScope::FlightsRead], "ACTIVE"),
            (vec![ApiClientScope::FlightsRead], "SUSPENDED"),
            (vec![ApiClientScope::FlightsRead], "REVOKED"),
            (vec![ApiClientScope::FlightsRead], "ACTIVE"),
        ],
        move |setup, runtime, fixtures| async move {
            let valid = fixtures[0].clone();
            let wrong_secret = Fixture {
                secret_text: hex::encode([0xabu8; 32]),
                ..valid.clone()
            };
            let router = app(runtime, true);
            let (wrong_status, wrong_headers, wrong_body) =
                send_token(&router, &token_request(&wrong_secret), None).await;
            assert_eq!(wrong_status, StatusCode::UNAUTHORIZED);
            assert!(wrong_headers.get(header::WWW_AUTHENTICATE).is_none());
            assert_external_error(&wrong_body, "EXTERNAL_CLIENT_AUTHENTICATION_FAILED");
            let expected_message = wrong_body["error"]["message"].clone();

            let unknown = Fixture {
                client_id: unique_client_id(),
                ..valid.clone()
            };
            let (unknown_status, _, unknown_body) =
                send_token(&router, &token_request(&unknown), None).await;
            assert_eq!(unknown_status, StatusCode::UNAUTHORIZED);
            assert_external_error(&unknown_body, "EXTERNAL_CLIENT_AUTHENTICATION_FAILED");
            assert_eq!(unknown_body["error"]["message"], expected_message);

            let suspended_body = assert_client_exchange_failure(&router, &fixtures[1]).await;
            assert_eq!(suspended_body["error"]["message"], expected_message);
            let revoked_body = assert_client_exchange_failure(&router, &fixtures[2]).await;
            assert_eq!(revoked_body["error"]["message"], expected_message);
            revoke_fixture_credential(&setup, &fixtures[3]).await;
            let revoked_credential_body =
                assert_client_exchange_failure(&router, &fixtures[3]).await;
            assert_eq!(
                revoked_credential_body["error"]["message"],
                expected_message
            );
        },
    )
    .await;
}

#[tokio::test]
async fn valid_token_authenticates_principal_and_current_scopes() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |_setup, runtime, fixture| async move {
            let state = state_with_auth(runtime);
            let token_router = build_router(state.clone());
            let token = exchange(&token_router, &fixture).await;
            let protected = protected_app(state);
            let (status, _, body) =
                send_protected(&protected, &[&format!("Bearer {token}")], None).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body["clientId"], fixture.client_id);
            assert_eq!(body["scopeCount"], 1);
            assert_eq!(body["hasFlightsRead"], true);
        },
    )
    .await;
}

#[tokio::test]
async fn zero_scope_client_authenticates_with_empty_principal() {
    run_fixture_test(
        vec![],
        "ACTIVE",
        move |_setup, runtime, fixture| async move {
            let state = state_with_auth(runtime);
            let token_router = build_router(state.clone());
            let token = exchange(&token_router, &fixture).await;
            let protected = protected_app(state);
            let (status, _, body) =
                send_protected(&protected, &[&format!("Bearer {token}")], None).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body["scopeCount"], 0);
            assert_eq!(body["hasFlightsRead"], false);
        },
    )
    .await;
}

#[tokio::test]
async fn expired_and_malformed_bearers_are_generic_401_with_challenge() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |setup, runtime, fixture| async move {
            let expired = insert_expired_token(&setup, &fixture).await;
            let protected = protected_app(state_with_auth(runtime));
            for value in [
                "",
                "Basic abc",
                "Bearer",
                "Bearer ",
                "Bearer xfa_v2_0000000000000000000000000000000000000000000000000000000000000000",
                "Bearer xfa_v1_000000000000000000000000000000000000000000000000000000000000000",
                "Bearer xfa_v1_000000000000000000000000000000000000000000000000000000000000000A",
                "Bearer  xfa_v1_0000000000000000000000000000000000000000000000000000000000000000",
                "Bearer xfa_v1_0000000000000000000000000000000000000000000000000000000000000000 ",
                "Bearer xfa_v1_0000000000000000000000000000000000000000000000000000000000000000 extra",
            ] {
                let values = if value.is_empty() {
                    &[][..]
                } else {
                    &[value][..]
                };
                let (status, headers, body) = send_protected(&protected, values, None).await;
                assert_eq!(status, StatusCode::UNAUTHORIZED);
                assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
                assert_external_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
            }
            let value = format!("Bearer {expired}");
            let (status, headers, body) = send_protected(&protected, &[&value], None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
            assert_external_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
        },
    )
    .await;
}

#[tokio::test]
async fn duplicate_authorization_and_cookies_do_not_authenticate_external_routes() {
    let (_, runtime) = pools().await;
    let protected = protected_app(state_with_auth(runtime));
    let (status, headers, body) = send_protected(
        &protected,
        &[
            "Bearer xfa_v1_0000000000000000000000000000000000000000000000000000000000000000",
            "Bearer xfa_v1_1111111111111111111111111111111111111111111111111111111111111111",
        ],
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
    assert_external_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");

    let (status, _, body) = send_protected(
        &protected,
        &[],
        Some("x_fly_staff_session=not-an-external-token; x_fly_manage_booking=also-not-token"),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_external_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
}

#[tokio::test]
async fn revoked_token_stays_invalid_after_client_reactivation() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |setup, runtime, fixture| async move {
            let state = state_with_auth(runtime);
            let token_router = build_router(state.clone());
            let token = exchange(&token_router, &fixture).await;
            mark_client_status(&setup, &fixture, "SUSPENDED").await;
            sqlx::query(
                "UPDATE external_access_tokens SET revoked_at=clock_timestamp()
                 WHERE api_client_credential_id=$1 AND revoked_at IS NULL",
            )
            .bind(fixture.credential_id)
            .execute(&setup)
            .await
            .expect("revoke fixture token");
            mark_client_status(&setup, &fixture, "ACTIVE").await;
            let protected = protected_app(state);
            let bearer = format!("Bearer {token}");
            let (status, headers, body) = send_protected(&protected, &[&bearer], None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
            assert_external_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
        },
    )
    .await;
}

#[tokio::test]
async fn current_scope_removal_is_seen_on_the_next_authenticated_request() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |setup, runtime, fixture| async move {
            let state = state_with_auth(runtime);
            let token_router = build_router(state.clone());
            let token = exchange(&token_router, &fixture).await;
            let protected = protected_app(state);
            let bearer = format!("Bearer {token}");
            let (status, _, body) = send_protected(&protected, &[&bearer], None).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body["scopeCount"], 1);
            sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1")
                .bind(fixture.client_pk)
                .execute(&setup)
                .await
                .expect("remove current fixture scope");
            let (status, _, body) = send_protected(&protected, &[&bearer], None).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body["scopeCount"], 0);
            assert_eq!(body["hasFlightsRead"], false);
        },
    )
    .await;
}

#[tokio::test]
async fn suspended_client_and_revoked_credential_are_generic_bearer_failures() {
    run_fixtures_test(
        vec![
            (vec![ApiClientScope::FlightsRead], "ACTIVE"),
            (vec![ApiClientScope::FlightsRead], "ACTIVE"),
        ],
        move |setup, runtime, fixtures| async move {
            let suspended_fixture = fixtures[0].clone();
            let revoked_fixture = fixtures[1].clone();
            let suspended_state = state_with_auth(runtime.clone());
            let token_router = build_router(suspended_state.clone());
            let suspended_token = exchange(&token_router, &suspended_fixture).await;
            mark_client_status(&setup, &suspended_fixture, "SUSPENDED").await;
            let suspended_bearer = format!("Bearer {suspended_token}");
            let (status, headers, body) =
                send_protected(&protected_app(suspended_state), &[&suspended_bearer], None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
            assert_external_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");

            let revoked_state = state_with_auth(runtime);
            let token_router = build_router(revoked_state.clone());
            let revoked_token = exchange(&token_router, &revoked_fixture).await;
            revoke_fixture_credential(&setup, &revoked_fixture).await;
            let revoked_bearer = format!("Bearer {revoked_token}");
            let (status, headers, body) =
                send_protected(&protected_app(revoked_state), &[&revoked_bearer], None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
            assert_external_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
        },
    )
    .await;
}

#[tokio::test]
async fn external_token_origin_has_no_browser_cors_headers() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |_setup, runtime, fixture| async move {
            let router = app(runtime, true);
            let (status, headers, _) = send_token(
                &router,
                &token_request(&fixture),
                Some("https://example.test"),
            )
            .await;
            assert_eq!(status, StatusCode::OK);
            for name in [
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                header::ACCESS_CONTROL_ALLOW_METHODS,
            ] {
                assert!(headers.get(name).is_none());
            }
        },
    )
    .await;
}

#[tokio::test]
async fn external_protected_origin_has_no_browser_cors_headers() {
    let (_, runtime) = pools().await;
    let protected = protected_app(state_with_auth(runtime));
    let mut request = Request::builder()
        .method("GET")
        .uri("/probe")
        .header(header::ORIGIN, "https://example.test")
        .body(Body::empty())
        .expect("protected origin request");
    request.headers_mut().insert(
        header::AUTHORIZATION,
        HeaderValue::from_static(
            "Bearer xfa_v1_0000000000000000000000000000000000000000000000000000000000000000",
        ),
    );
    let response = protected
        .clone()
        .oneshot(request)
        .await
        .expect("protected response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    for name in [
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        header::ACCESS_CONTROL_ALLOW_METHODS,
    ] {
        assert!(response.headers().get(name).is_none());
    }
}

#[tokio::test]
async fn missing_external_auth_service_fails_closed() {
    let (_, runtime) = pools().await;
    let router = app(runtime, false);
    let raw = r#"{"clientId":"XFCABCDEFGHJKLMNP22","clientSecret":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}"#;
    let (status, _, body) = send_token(&router, raw, None).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_external_error(&body, "EXTERNAL_AUTH_UNAVAILABLE");
}

#[tokio::test(flavor = "current_thread")]
async fn token_exchange_success_emits_authoritative_diagnostic() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |_setup, runtime, fixture| async move {
            let router = app(runtime, true);
            let ((status, headers, body), output) =
                send_token_captured(&router, &token_request(&fixture), None).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(headers[header::CACHE_CONTROL], "no-store, private");
            assert_eq!(body["tokenType"], "Bearer");
            assert!(output.contains("external_auth_diagnostic=\"succeeded\""));
            assert!(output.contains("external_auth_flow=\"token_exchange\""));
            assert!(output.contains(&format!("external_client_id=\"{}\"", fixture.client_id)));
            assert!(!output.contains(&fixture.secret_text));
            assert!(!output.contains("Authorization"));
        },
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn token_exchange_known_bad_secret_emits_safe_attributed_diagnostic() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |_setup, runtime, fixture| async move {
            let router = app(runtime, true);
            let secret = "ab".repeat(32);
            let request = json!({
                "clientId": fixture.client_id,
                "clientSecret": secret,
            })
            .to_string();
            let ((status, headers, body), output) =
                send_token_captured(&router, &request, None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
            assert_external_error(&body, "EXTERNAL_CLIENT_AUTHENTICATION_FAILED");
            assert!(output.contains("external_auth_diagnostic=\"secret_mismatch\""));
            assert!(output.contains(&format!("external_client_id=\"{}\"", fixture.client_id)));
            assert!(!output.contains(&secret));
            assert!(!output.contains("Authorization"));
        },
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn token_exchange_unknown_client_is_unattributed_and_diagnostic() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |_setup, runtime, _fixture| async move {
            let router = app(runtime, true);
            let unknown_client_id = unique_client_id();
            let secret = "cd".repeat(32);
            let request = json!({
                "clientId": unknown_client_id,
                "clientSecret": secret,
            })
            .to_string();
            let ((status, _, body), output) = send_token_captured(&router, &request, None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_external_error(&body, "EXTERNAL_CLIENT_AUTHENTICATION_FAILED");
            assert!(output.contains("external_auth_diagnostic=\"unknown_client\""));
            assert!(!output.contains(&unknown_client_id));
            assert!(!output.contains(&secret));
        },
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn malformed_token_exchange_emits_only_bounded_diagnostic() {
    let (_, runtime) = pools().await;
    let router = app(runtime, false);
    let sentinel = "malformed-client-sentinel";
    let ((status, _, body), output) = send_token_captured(
        &router,
        &json!({"clientId": sentinel, "clientSecret": "not-a-secret"}).to_string(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_external_error(&body, "EXTERNAL_REQUEST_INVALID");
    assert!(output.contains("external_auth_diagnostic=\"malformed\""));
    assert!(!output.contains(sentinel));
    assert!(!output.contains("not-a-secret"));
}

#[tokio::test(flavor = "current_thread")]
async fn bearer_success_emits_authoritative_diagnostic() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |_setup, runtime, fixture| async move {
            let state = state_with_auth(runtime);
            let token_router = build_router(state.clone());
            let token = exchange(&token_router, &fixture).await;
            let protected = protected_app(state);
            let ((status, _, body), output) =
                send_protected_captured(&protected, &[&format!("Bearer {token}")], None).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body["clientId"], fixture.client_id);
            assert!(output.contains("external_auth_diagnostic=\"succeeded\""));
            assert!(output.contains("external_auth_flow=\"bearer\""));
            assert!(output.contains(&format!("external_client_id=\"{}\"", fixture.client_id)));
            assert!(!output.contains(&token));
            assert!(!output.contains("Authorization"));
        },
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn expired_bearer_emits_attributed_diagnostic_without_token_material() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |setup, runtime, fixture| async move {
            let token = insert_expired_token(&setup, &fixture).await;
            let protected = protected_app(state_with_auth(runtime));
            let authorization = format!("Bearer {token}");
            let ((status, headers, body), output) =
                send_protected_captured(&protected, &[&authorization], None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
            assert_external_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
            assert!(output.contains("external_auth_diagnostic=\"expired\""));
            assert!(output.contains(&format!("external_client_id=\"{}\"", fixture.client_id)));
            assert!(!output.contains(&token));
            assert!(!output.contains("Authorization"));
        },
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn suspended_bearer_emits_attributed_diagnostic_without_changing_contract() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |setup, runtime, fixture| async move {
            let state = state_with_auth(runtime);
            let token_router = build_router(state.clone());
            let token = exchange(&token_router, &fixture).await;
            mark_client_status(&setup, &fixture, "SUSPENDED").await;
            let protected = protected_app(state);
            let authorization = format!("Bearer {token}");
            let ((status, headers, body), output) =
                send_protected_captured(&protected, &[&authorization], None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
            assert_external_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
            assert!(output.contains("external_auth_diagnostic=\"suspended\""));
            assert!(output.contains(&format!("external_client_id=\"{}\"", fixture.client_id)));
            assert!(!output.contains(&token));
        },
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn revoked_client_token_exchange_emits_attributed_diagnostic() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "REVOKED",
        move |_setup, runtime, fixture| async move {
            let router = app(runtime, true);
            let ((status, headers, body), output) =
                send_token_captured(&router, &token_request(&fixture), None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
            assert_external_error(&body, "EXTERNAL_CLIENT_AUTHENTICATION_FAILED");
            assert!(output.contains("external_auth_diagnostic=\"revoked\""));
            assert!(output.contains(&format!("external_client_id=\"{}\"", fixture.client_id)));
            assert!(!output.contains(&fixture.secret_text));
        },
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn token_exchange_without_live_credential_has_no_dedicated_diagnostic() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |setup, runtime, fixture| async move {
            revoke_fixture_credential(&setup, &fixture).await;
            let router = app(runtime, true);
            let ((status, headers, body), output) =
                send_token_captured(&router, &token_request(&fixture), None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
            assert_external_error(&body, "EXTERNAL_CLIENT_AUTHENTICATION_FAILED");
            assert!(!output.contains("credential_unavailable"));
            assert!(!output.contains("secret_mismatch"));
            assert!(!output.contains(&fixture.secret_text));
        },
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn revoked_bearer_emits_attributed_diagnostic_without_token_material() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        move |setup, runtime, fixture| async move {
            let state = state_with_auth(runtime);
            let token_router = build_router(state.clone());
            let token = exchange(&token_router, &fixture).await;
            revoke_fixture_credential(&setup, &fixture).await;
            let protected = protected_app(state);
            let authorization = format!("Bearer {token}");
            let ((status, headers, body), output) =
                send_protected_captured(&protected, &[&authorization], None).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
            assert_external_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
            assert!(output.contains("external_auth_diagnostic=\"revoked\""));
            assert!(output.contains(&format!("external_client_id=\"{}\"", fixture.client_id)));
            assert!(!output.contains(&token));
        },
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn unknown_valid_format_bearer_has_no_extra_diagnostic() {
    let (_, runtime) = pools().await;
    let protected = protected_app(state_with_auth(runtime));
    let unknown = "Bearer xfa_v1_ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    let ((status, headers, body), output) =
        send_protected_captured(&protected, &[unknown], None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
    assert_external_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
    assert!(!output.contains("external_auth_diagnostic"));
    assert!(!output.contains(unknown));
}

#[tokio::test(flavor = "current_thread")]
async fn missing_and_malformed_bearer_emit_bounded_diagnostics() {
    let (_, runtime) = pools().await;
    let protected = protected_app(state_with_auth(runtime));
    let ((missing_status, _, missing_body), missing_output) =
        send_protected_captured(&protected, &[], None).await;
    assert_eq!(missing_status, StatusCode::UNAUTHORIZED);
    assert_external_error(&missing_body, "EXTERNAL_AUTHENTICATION_FAILED");
    assert!(missing_output.contains("external_auth_diagnostic=\"missing\""));

    let ((malformed_status, _, malformed_body), malformed_output) =
        send_protected_captured(&protected, &["Basic malformed-sentinel"], None).await;
    assert_eq!(malformed_status, StatusCode::UNAUTHORIZED);
    assert_external_error(&malformed_body, "EXTERNAL_AUTHENTICATION_FAILED");
    assert!(malformed_output.contains("external_auth_diagnostic=\"malformed\""));
    assert!(!malformed_output.contains("malformed-sentinel"));
}

impl Clone for Fixture {
    fn clone(&self) -> Self {
        Self {
            client_pk: self.client_pk,
            client_id: self.client_id.clone(),
            actor_id: self.actor_id,
            secret_text: self.secret_text.clone(),
            credential_id: self.credential_id,
        }
    }
}
