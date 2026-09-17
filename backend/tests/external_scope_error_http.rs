mod common;

use std::{
    collections::BTreeSet,
    future::Future,
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    body::{Body, Bytes},
    extract::Extension,
    http::{header, HeaderMap, HeaderValue, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::sync::oneshot;
use tower::ServiceExt;
use tower_http::request_id::RequestId;
use tracing::Instrument;
use uuid::Uuid;

use x_fly_api::{
    application::external_auth::{
        require_scope, ExternalAuthService, ExternalCredentialCrypto, ExternalScopeError,
    },
    domain::{
        api_client::ApiClientScope,
        external_api::{ExternalApiCredentialPepper, ExternalPrincipal, PlaintextClientSecret},
    },
    infrastructure::{
        database::{
            migrate_database, verify_database_ready, SqlxExternalAuthRepository,
            SqlxSeatHoldRepository,
        },
        external_auth_crypto::HmacExternalCredentialCrypto,
        http::{
            build_router,
            external::{
                ExternalBearerAuthLayer, ExternalErrorCode, ExternalErrorEnvelope,
                ExternalScopeLayer,
            },
        },
    },
    state::AppState,
};

const PEPPER: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const REQUEST_ID: &str = "11111111-1111-4111-8111-111111111111";

#[derive(Clone)]
struct Fixture {
    client_pk: Uuid,
    client_id: String,
    actor_id: Uuid,
    secret_text: String,
}

#[derive(Clone)]
struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

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
    let mut transaction = setup.begin().await.expect("begin external scope fixture");
    let actor_id: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash) VALUES ($1,'external-scope-http') RETURNING id",
    )
    .bind(format!("external-scope-{}@test.invalid", Uuid::new_v4()))
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
    .bind(format!("External scope {}", Uuid::new_v4()))
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
    sqlx::query(
        "INSERT INTO api_client_credentials
         (api_client_id,secret_digest,digest_version,issued_at,issued_by_staff_user_id)
         VALUES ($1,$2,1,clock_timestamp(),$3)",
    )
    .bind(client_pk)
    .bind(digest.as_bytes().as_slice())
    .bind(actor_id)
    .execute(&mut *transaction)
    .await
    .expect("fixture credential");
    transaction
        .commit()
        .await
        .expect("commit external scope fixture");

    Fixture {
        client_pk,
        client_id,
        actor_id,
        secret_text,
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

async fn run_fixture_test<F, Fut>(scopes: Vec<ApiClientScope>, body: F)
where
    F: FnOnce(PgPool, PgPool, Fixture) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &scopes, "ACTIVE").await;
    let cleanup_setup = setup.clone();
    let cleanup_target = fixture.clone();
    common::run_fixture_body_with_cleanup(
        move || body(setup, runtime, fixture),
        move || async move { cleanup_fixture(&cleanup_setup, &cleanup_target).await },
    )
    .await;
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

fn state_without_auth(runtime: PgPool) -> AppState {
    let booking = Arc::new(SqlxSeatHoldRepository::new(runtime));
    AppState::new(
        booking.clone(),
        booking.clone(),
        booking.clone(),
        booking,
        Duration::from_secs(600),
        true,
        "http://localhost:3000".to_owned(),
    )
}

async fn response_body(response: Response) -> (StatusCode, HeaderMap, Value) {
    let status = response.status();
    let headers = response.headers().clone();
    let bytes: Bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, body)
}

async fn probe(Extension(principal): Extension<ExternalPrincipal>) -> Json<Value> {
    Json(json!({
        "authorized": true,
        "clientId": principal.client_id(),
        "scopeCount": principal.scopes().len(),
    }))
}

fn scope_app(required: ApiClientScope) -> Router {
    let routes: Router = Router::new().route("/probe", get(probe));
    ExternalScopeLayer::new(required).layer(routes)
}

fn authenticated_scope_app(state: AppState, required: ApiClientScope) -> Router {
    let routes: Router<AppState> = Router::new().route("/probe", get(probe));
    let routes = ExternalScopeLayer::new(required).layer(routes);
    ExternalBearerAuthLayer::new(state.clone())
        .layer(routes)
        .with_state(state)
}

fn authenticated_app(state: AppState) -> Router {
    let routes: Router<AppState> = Router::new().route("/probe", get(probe));
    ExternalBearerAuthLayer::new(state.clone())
        .layer(routes)
        .with_state(state)
}

fn request_id() -> RequestId {
    RequestId::new(HeaderValue::from_static(REQUEST_ID))
}

async fn send_principal(
    router: &Router,
    principal: ExternalPrincipal,
) -> (StatusCode, HeaderMap, Value) {
    let mut request = Request::builder()
        .method("GET")
        .uri("/probe")
        .body(Body::empty())
        .expect("scope request");
    request.extensions_mut().insert(principal);
    request.extensions_mut().insert(request_id());
    let response = router
        .clone()
        .oneshot(request)
        .await
        .expect("scope response");
    response_body(response).await
}

async fn send_bearer(router: &Router, token: Option<&str>) -> (StatusCode, HeaderMap, Value) {
    let mut request = Request::builder()
        .method("GET")
        .uri("/probe")
        .body(Body::empty())
        .expect("bearer request");
    if let Some(token) = token {
        request.headers_mut().insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(token).expect("bearer header"),
        );
    }
    request.extensions_mut().insert(request_id());
    let response = router
        .clone()
        .oneshot(request)
        .await
        .expect("bearer response");
    response_body(response).await
}

async fn send_bearer_captured(
    router: &Router,
    token: Option<&str>,
) -> ((StatusCode, HeaderMap, Value), String) {
    let output = Arc::new(Mutex::new(Vec::new()));
    let writer_output = Arc::clone(&output);
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(move || CaptureWriter(Arc::clone(&writer_output)))
        .finish();
    let guard = tracing::subscriber::set_default(subscriber);
    let span = tracing::info_span!(
        "http_request",
        request_id = REQUEST_ID,
        external_auth_diagnostic = tracing::field::Empty,
    );
    let response = send_bearer(router, token).instrument(span).await;
    drop(guard);
    let output = String::from_utf8(output.lock().expect("capture output lock").clone())
        .expect("trace output is UTF-8");
    (response, output)
}

async fn exchange(state: &AppState, fixture: &Fixture) -> String {
    let token_router = build_router(state.clone());
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/external/token")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({"clientId": fixture.client_id, "clientSecret": fixture.secret_text}).to_string(),
        ))
        .expect("token request");
    let response = token_router.oneshot(request).await.expect("token response");
    let (status, _, body) = response_body(response).await;
    assert_eq!(status, StatusCode::OK);
    body["accessToken"]
        .as_str()
        .expect("access token")
        .to_owned()
}

async fn issue_and_check_scope(
    scopes: &[ApiClientScope],
    required: ApiClientScope,
) -> (StatusCode, HeaderMap, Value) {
    let (result_tx, result_rx) = oneshot::channel();
    run_fixture_test(
        scopes.to_vec(),
        move |_setup, runtime, fixture| async move {
            let state = state_with_auth(runtime);
            let token = exchange(&state, &fixture).await;
            let bearer = format!("Bearer {token}");
            let result =
                send_bearer(&authenticated_scope_app(state, required), Some(&bearer)).await;
            result_tx.send(result).expect("scope result receiver");
        },
    )
    .await;
    result_rx.await.expect("scope result")
}

fn principal(scopes: &[ApiClientScope]) -> ExternalPrincipal {
    ExternalPrincipal::new(
        Uuid::new_v4(),
        "XFCABCDEFGHJKLMNP2".to_owned(),
        scopes.iter().copied().collect::<BTreeSet<_>>(),
    )
}

fn assert_error(body: &Value, code: &str) {
    assert_eq!(body["error"]["code"].as_str(), Some(code));
    assert!(body["error"]["message"].as_str().is_some());
    assert_eq!(body["error"]["requestId"].as_str(), Some(REQUEST_ID));
}

#[test]
fn require_scope_accepts_only_flights_read() {
    let principal = principal(&[ApiClientScope::FlightsRead]);
    assert!(require_scope(&principal, ApiClientScope::FlightsRead).is_ok());
    assert_eq!(
        require_scope(&principal, ApiClientScope::AnalyticsRead),
        Err(ExternalScopeError::Missing)
    );
}

#[test]
fn require_scope_accepts_only_analytics_read() {
    let principal = principal(&[ApiClientScope::AnalyticsRead]);
    assert!(require_scope(&principal, ApiClientScope::AnalyticsRead).is_ok());
    assert_eq!(
        require_scope(&principal, ApiClientScope::FlightsRead),
        Err(ExternalScopeError::Missing)
    );
}

#[tokio::test]
async fn flights_scope_allows_only_flights_read() {
    let (status, _, body) =
        issue_and_check_scope(&[ApiClientScope::FlightsRead], ApiClientScope::FlightsRead).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authorized"], true);
}

#[tokio::test]
async fn analytics_scope_allows_only_analytics_read() {
    let (status, _, body) = issue_and_check_scope(
        &[ApiClientScope::AnalyticsRead],
        ApiClientScope::AnalyticsRead,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authorized"], true);
}

#[tokio::test]
async fn flights_scope_is_denied_for_analytics_requirement() {
    let (status, headers, body) = issue_and_check_scope(
        &[ApiClientScope::FlightsRead],
        ApiClientScope::AnalyticsRead,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
    assert_error(&body, "EXTERNAL_SCOPE_DENIED");
}

#[tokio::test]
async fn analytics_scope_is_denied_for_flights_requirement() {
    let (status, headers, body) = issue_and_check_scope(
        &[ApiClientScope::AnalyticsRead],
        ApiClientScope::FlightsRead,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
    assert_error(&body, "EXTERNAL_SCOPE_DENIED");
}

#[tokio::test]
async fn both_scopes_satisfy_either_required_scope() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead, ApiClientScope::AnalyticsRead],
        move |_setup, runtime, fixture| async move {
            let state = state_with_auth(runtime);
            let token = exchange(&state, &fixture).await;
            let bearer = format!("Bearer {token}");
            for required in [ApiClientScope::FlightsRead, ApiClientScope::AnalyticsRead] {
                let (status, _, body) = send_bearer(
                    &authenticated_scope_app(state.clone(), required),
                    Some(&bearer),
                )
                .await;
                assert_eq!(status, StatusCode::OK);
                assert_eq!(body["authorized"], true);
            }
        },
    )
    .await;
}

#[tokio::test]
async fn missing_scope_is_403() {
    let (status, _, body) = issue_and_check_scope(&[], ApiClientScope::FlightsRead).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&body, "EXTERNAL_SCOPE_DENIED");
}

#[tokio::test]
async fn zero_scope_authenticated_then_denied_403() {
    run_fixture_test(vec![], move |_setup, runtime, fixture| async move {
        let state = state_with_auth(runtime);
        let token = exchange(&state, &fixture).await;
        let bearer = format!("Bearer {token}");
        let (status, _, body) = send_bearer(&authenticated_app(state.clone()), Some(&bearer)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["scopeCount"], 0);
        let (status, _, body) = send_bearer(
            &authenticated_scope_app(state, ApiClientScope::FlightsRead),
            Some(&bearer),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_error(&body, "EXTERNAL_SCOPE_DENIED");
    })
    .await;
}

#[tokio::test]
async fn scope_removal_is_seen_on_next_request() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        move |setup, runtime, fixture| async move {
            let state = state_with_auth(runtime);
            let token_router = build_router(state.clone());
            let request = Request::builder()
                .method("POST")
                .uri("/api/v1/external/token")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"clientId": fixture.client_id, "clientSecret": fixture.secret_text})
                        .to_string(),
                ))
                .expect("token request");
            let response = token_router.oneshot(request).await.expect("token response");
            let (status, _, body) = response_body(response).await;
            assert_eq!(status, StatusCode::OK);
            let token = body["accessToken"]
                .as_str()
                .expect("access token")
                .to_owned();
            let protected = authenticated_scope_app(state, ApiClientScope::FlightsRead);
            let bearer = format!("Bearer {token}");
            let (status, _, _) = send_bearer(&protected, Some(&bearer)).await;
            assert_eq!(status, StatusCode::OK);

            sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1")
                .bind(fixture.client_pk)
                .execute(&setup)
                .await
                .expect("remove current fixture scope");
            let (status, headers, body) = send_bearer(&protected, Some(&bearer)).await;
            assert_eq!(status, StatusCode::FORBIDDEN);
            assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
            assert_error(&body, "EXTERNAL_SCOPE_DENIED");
        },
    )
    .await;
}

#[tokio::test]
async fn unauthenticated_bearer_remains_401() {
    let (_, runtime) = pools().await;
    let (status, headers, body) = send_bearer(
        &authenticated_scope_app(state_with_auth(runtime), ApiClientScope::FlightsRead),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
    assert_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
}

#[tokio::test]
async fn unavailable_auth_remains_503_not_scope_denied() {
    let (_, runtime) = pools().await;
    let (status, headers, body) = send_bearer(
        &authenticated_scope_app(state_without_auth(runtime), ApiClientScope::FlightsRead),
        Some("Bearer xfa_v1_0000000000000000000000000000000000000000000000000000000000000000"),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
    assert_error(&body, "EXTERNAL_AUTH_UNAVAILABLE");
}

#[tokio::test]
async fn scope_denial_error_has_authoritative_request_id_and_no_sensitive_state() {
    let internal_id = Uuid::new_v4();
    let (status, headers, body) = send_principal(
        &scope_app(ApiClientScope::AnalyticsRead),
        ExternalPrincipal::new(
            internal_id,
            "XFCABCDEFGHJKLMNP2".to_owned(),
            BTreeSet::from([ApiClientScope::FlightsRead]),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
    assert_eq!(
        body.as_object()
            .map(|object| object.keys().cloned().collect::<Vec<_>>()),
        Some(vec!["error".to_owned()])
    );
    assert_eq!(
        body["error"]
            .as_object()
            .map(|object| object.keys().cloned().collect::<Vec<_>>()),
        Some(vec![
            "code".to_owned(),
            "message".to_owned(),
            "requestId".to_owned()
        ])
    );
    assert_error(&body, "EXTERNAL_SCOPE_DENIED");
    let serialized = body.to_string();
    assert!(!serialized.contains(&internal_id.to_string()));
    assert!(!serialized.contains("FlightsRead"));
    assert!(!serialized.contains("flights:read"));
}

#[tokio::test(flavor = "current_thread")]
async fn typed_scope_denial_emits_authoritative_client_and_scope() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        move |_setup, runtime, fixture| async move {
            let state = state_with_auth(runtime);
            let token = exchange(&state, &fixture).await;
            let bearer = format!("Bearer {token}");
            let required = ApiClientScope::AnalyticsRead;
            let ((status, _, body), output) =
                send_bearer_captured(&authenticated_scope_app(state, required), Some(&bearer))
                    .await;
            assert_eq!(status, StatusCode::FORBIDDEN);
            assert_error(&body, "EXTERNAL_SCOPE_DENIED");
            assert!(output.contains("external_auth_diagnostic=\"scope_denied\""));
            assert!(output.contains(&format!("external_client_id=\"{}\"", fixture.client_id)));
            assert!(output.contains("external_required_scope=\"analytics:read\""));
            assert!(!output.contains(&token));
            assert!(!output.contains("Authorization"));
        },
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn successful_typed_scope_check_emits_no_scope_denial() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        move |_setup, runtime, fixture| async move {
            let state = state_with_auth(runtime);
            let token = exchange(&state, &fixture).await;
            let bearer = format!("Bearer {token}");
            let ((status, _, body), output) = send_bearer_captured(
                &authenticated_scope_app(state, ApiClientScope::FlightsRead),
                Some(&bearer),
            )
            .await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body["authorized"], true);
            assert!(!output.contains("external_auth_diagnostic=\"scope_denied\""));
            assert!(output.contains("external_auth_diagnostic=\"succeeded\""));
            assert!(!output.contains(&token));
        },
    )
    .await;
}

async fn error_response(code: ExternalErrorCode) -> (StatusCode, HeaderMap, Value) {
    response_body(
        ExternalErrorEnvelope::new(code, Uuid::parse_str(REQUEST_ID).unwrap()).into_response(),
    )
    .await
}

#[tokio::test]
async fn invalid_request_is_400() {
    let (status, _, body) = error_response(ExternalErrorCode::ExternalRequestInvalid).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "EXTERNAL_REQUEST_INVALID");
}

#[tokio::test]
async fn not_found_is_404() {
    let (status, _, body) = error_response(ExternalErrorCode::ExternalResourceNotFound).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&body, "EXTERNAL_RESOURCE_NOT_FOUND");
}

#[tokio::test]
async fn dependency_failure_is_503() {
    let (status, _, body) = error_response(ExternalErrorCode::ExternalServiceUnavailable).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_error(&body, "EXTERNAL_SERVICE_UNAVAILABLE");
}

#[tokio::test]
async fn internal_failure_is_500() {
    let (status, _, body) = error_response(ExternalErrorCode::ExternalInternalError).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_error(&body, "EXTERNAL_INTERNAL_ERROR");
}

#[test]
fn no_branch_25_rate_limit_code_exists() {
    let source = include_str!("../src/infrastructure/http/external.rs");
    assert!(!source.contains("EXTERNAL_RATE_LIMITED"));
    assert!(!source.contains("TOO_MANY_REQUESTS"));
}
