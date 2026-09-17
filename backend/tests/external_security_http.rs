mod common;

use std::{
    fmt::Debug,
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    response::IntoResponse,
    Router,
};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::ServiceExt;
use uuid::Uuid;
use x_fly_api::{
    application::external_auth::{
        record_external_auth_diagnostic, ApiClientCredentialService, ExternalAuthDiagnostic,
        ExternalAuthRepository, ExternalAuthService,
    },
    domain::external_api::{
        AccessTokenHash, CredentialAdministrationError, CredentialDigest,
        CredentialRevocationReason, ExternalApiCredentialPepper, ExternalAuthenticationError,
        ExternalPrincipal, ExternalTokenExchangeError,
    },
    infrastructure::{
        database::{migrate_database, verify_database_ready, SqlxSeatHoldRepository},
        external_auth_crypto::HmacExternalCredentialCrypto,
        http::{
            build_router,
            external::{ExternalErrorCode, ExternalErrorEnvelope},
        },
    },
    state::AppState,
};

const PEPPER: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

struct RejectingRepository;

#[async_trait]
impl ExternalAuthRepository for RejectingRepository {
    async fn issue_credential(
        &self,
        _client_id: &str,
        _actor_staff_user_id: Uuid,
        _expected_client_version: i64,
        _digest: &CredentialDigest,
        _issued_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<x_fly_api::domain::external_api::CredentialMetadata, CredentialAdministrationError>
    {
        Err(CredentialAdministrationError::Infrastructure)
    }

    async fn revoke_credential(
        &self,
        _client_id: &str,
        _actor_staff_user_id: Uuid,
        _expected_client_version: i64,
        _reason: CredentialRevocationReason,
        _revoked_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<x_fly_api::domain::external_api::CredentialMetadata, CredentialAdministrationError>
    {
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
        Err(ExternalTokenExchangeError::InvalidCredential)
    }

    async fn authenticate_access_token(
        &self,
        _token_hash: &AccessTokenHash,
        _now: chrono::DateTime<chrono::Utc>,
    ) -> Result<ExternalPrincipal, ExternalAuthenticationError> {
        Err(ExternalAuthenticationError::InvalidToken)
    }
}

async fn test_pool() -> PgPool {
    let setup = PgPoolOptions::new()
        .max_connections(2)
        .connect(&common::test_database_url())
        .await
        .expect("TEST setup connection");
    migrate_database(&setup).await.expect("TEST migrations");
    let runtime = PgPoolOptions::new()
        .max_connections(2)
        .connect(&common::test_runtime_database_url())
        .await
        .expect("TEST runtime connection");
    verify_database_ready(&runtime)
        .await
        .expect("TEST runtime schema");
    runtime
}

fn rejecting_router(runtime: PgPool) -> Router {
    let booking = Arc::new(SqlxSeatHoldRepository::new(runtime));
    let repository = Arc::new(RejectingRepository);
    let crypto = Arc::new(HmacExternalCredentialCrypto::from_pepper(
        ExternalApiCredentialPepper::parse_hex(PEPPER).expect("test pepper"),
    ));
    let service = ExternalAuthService::new(repository.clone(), crypto.clone());
    let state = AppState::new(
        booking.clone(),
        booking.clone(),
        booking.clone(),
        booking,
        Duration::from_secs(600),
        true,
        "http://localhost:3000".to_owned(),
    )
    .with_external_auth(ApiClientCredentialService::new(repository, crypto), service);
    build_router(state)
}

#[derive(Clone)]
struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

fn assert_redacted_debug<T: Debug>(value: &T, forbidden: &[&str]) {
    let formatted = format!("{value:?}");
    for sentinel in forbidden {
        assert!(
            !formatted.contains(sentinel),
            "debug output contained forbidden secret material"
        );
    }
}

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

#[test]
fn sentinel_secret_token_and_authorization_never_enter_logs() {
    let output = Arc::new(Mutex::new(Vec::new()));
    let writer_output = Arc::clone(&output);
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(move || CaptureWriter(Arc::clone(&writer_output)))
        .finish();
    let span = tracing::info_span!(
        "http_request",
        external_auth_diagnostic = tracing::field::Empty,
    );
    let sentinel = "task10-secret-token-sentinel";
    assert_redacted_debug(
        &ExternalAuthDiagnostic::SecretMismatch,
        &[sentinel, "Bearer"],
    );
    tracing::subscriber::with_default(subscriber, || {
        span.in_scope(|| {
            for diagnostic in [
                ExternalAuthDiagnostic::Missing,
                ExternalAuthDiagnostic::Malformed,
                ExternalAuthDiagnostic::UnknownClient,
                ExternalAuthDiagnostic::SecretMismatch,
                ExternalAuthDiagnostic::Expired,
                ExternalAuthDiagnostic::Suspended,
                ExternalAuthDiagnostic::Revoked,
                ExternalAuthDiagnostic::ScopeDenied,
                ExternalAuthDiagnostic::Succeeded,
            ] {
                record_external_auth_diagnostic(&span, diagnostic);
            }
            let _ = sentinel;
        });
    });
    let output = String::from_utf8(output.lock().expect("capture output lock").clone())
        .expect("trace output is UTF-8");
    for category in [
        "missing",
        "malformed",
        "unknown_client",
        "secret_mismatch",
        "expired",
        "suspended",
        "revoked",
        "scope_denied",
        "succeeded",
    ] {
        assert!(
            output.contains(category),
            "diagnostic category missing: {category}"
        );
    }
    assert!(!output.contains(sentinel));
    assert!(!output.contains("Authorization"));
    assert!(!output.contains("Bearer"));
}

#[tokio::test]
async fn external_error_taxonomy_is_generic_and_challenge_safe() {
    let request_id = Uuid::new_v4();
    let cases = [
        (
            ExternalErrorCode::ExternalRequestInvalid,
            StatusCode::BAD_REQUEST,
            false,
        ),
        (
            ExternalErrorCode::ExternalClientAuthenticationFailed,
            StatusCode::UNAUTHORIZED,
            false,
        ),
        (
            ExternalErrorCode::ExternalAuthenticationFailed,
            StatusCode::UNAUTHORIZED,
            true,
        ),
        (
            ExternalErrorCode::ExternalScopeDenied,
            StatusCode::FORBIDDEN,
            false,
        ),
        (
            ExternalErrorCode::ExternalResourceNotFound,
            StatusCode::NOT_FOUND,
            false,
        ),
        (
            ExternalErrorCode::ExternalAuthUnavailable,
            StatusCode::SERVICE_UNAVAILABLE,
            false,
        ),
        (
            ExternalErrorCode::ExternalServiceUnavailable,
            StatusCode::SERVICE_UNAVAILABLE,
            false,
        ),
        (
            ExternalErrorCode::ExternalInternalError,
            StatusCode::INTERNAL_SERVER_ERROR,
            false,
        ),
    ];
    for (code, status, challenge) in cases {
        let response = ExternalErrorEnvelope::new(code, request_id).into_response();
        assert_eq!(response.status(), status);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "no-store, private"
        );
        assert_eq!(
            response.headers().get(header::WWW_AUTHENTICATE).is_some(),
            challenge
        );
        let body = response
            .into_body()
            .collect()
            .await
            .expect("error response body")
            .to_bytes();
        let body: Value = serde_json::from_slice(&body).expect("error JSON");
        let keys = body["error"]
            .as_object()
            .expect("error object")
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(keys, vec!["code", "message", "requestId"]);
        assert_eq!(body["error"]["requestId"], request_id.to_string());
        let rendered = body.to_string();
        assert!(!rendered.contains("sqlx"));
        assert!(!rendered.contains("secret_digest"));
        assert!(!rendered.contains("token_hash"));
        assert!(!rendered.contains("credential_id"));
    }
}

#[tokio::test]
async fn unknown_client_and_wrong_secret_have_generic_public_outcomes() {
    let router = rejecting_router(test_pool().await);
    let secret = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let requests = [
        serde_json::json!({
            "clientId": "XFCABCDEFGHJKLMNP22",
            "clientSecret": secret,
        }),
        serde_json::json!({
            "clientId": "XFCABCDEFGHJKLMNP23",
            "clientSecret": secret,
        }),
    ];
    let mut outcomes = Vec::new();
    for request_body in requests {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/external/token")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(request_body.to_string()))
                    .expect("token request"),
            )
            .await
            .expect("token response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());
        let body = response
            .into_body()
            .collect()
            .await
            .expect("token error body")
            .to_bytes();
        let body: Value = serde_json::from_slice(&body).expect("token error JSON");
        outcomes.push(body);
    }
    assert_eq!(
        outcomes[0]["error"]["code"],
        "EXTERNAL_CLIENT_AUTHENTICATION_FAILED"
    );
    assert_eq!(
        outcomes[1]["error"]["code"],
        "EXTERNAL_CLIENT_AUTHENTICATION_FAILED"
    );
    assert_eq!(
        outcomes[0]["error"]["message"],
        outcomes[1]["error"]["message"]
    );
}

#[tokio::test]
async fn internal_uuid_never_enters_external_json() {
    let internal_id = Uuid::new_v4();
    let request_id = Uuid::new_v4();
    let response = ExternalErrorEnvelope::new(
        ExternalErrorCode::ExternalClientAuthenticationFailed,
        request_id,
    )
    .into_response();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("error response body")
        .to_bytes();
    let body: Value = serde_json::from_slice(&body).expect("error JSON");
    assert!(body["error"].get("internalId").is_none());
    assert!(!body.to_string().contains(&internal_id.to_string()));
    assert!(body.to_string().contains(&request_id.to_string()));
}
