use axum::{
    extract::{Extension, Json, Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tower_http::request_id::RequestId;
use uuid::Uuid;

use crate::{
    application::{
        api_client::validate_client_id,
        external_auth::{require_scope, ExternalAuthService, ExternalScopeError},
    },
    domain::{
        api_client::ApiClientScope,
        external_api::{
            ExternalAuthenticationError, ExternalPrincipal, ExternalTokenExchangeError,
            IssuedAccessToken, PlaintextAccessToken, PlaintextClientSecret,
        },
    },
    state::AppState,
};

const MAX_CLIENT_ID_LENGTH: usize = 19;
const MAX_CLIENT_SECRET_LENGTH: usize = 64;
const MAX_AUTHORIZATION_LENGTH: usize = 256;
const EXTERNAL_ERROR_MESSAGE: &str = "The external request is invalid.";
const CLIENT_AUTH_FAILURE_MESSAGE: &str = "Client authentication failed.";
const AUTH_FAILURE_MESSAGE: &str = "External authentication failed.";
const AUTH_UNAVAILABLE_MESSAGE: &str = "External authentication is temporarily unavailable.";
const SCOPE_DENIED_MESSAGE: &str = "The required external scope is not available.";
const RESOURCE_NOT_FOUND_MESSAGE: &str = "The external resource was not found.";
const SERVICE_UNAVAILABLE_MESSAGE: &str = "The external service is temporarily unavailable.";
const INTERNAL_ERROR_MESSAGE: &str = "The external service encountered an internal error.";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TokenExchangeRequest {
    client_id: String,
    client_secret: String,
}

impl TokenExchangeRequest {
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenExchangeResponse {
    access_token: String,
    token_type: &'static str,
    expires_in: u64,
}

impl TokenExchangeResponse {
    fn from_issued(issued: IssuedAccessToken) -> Self {
        let access_token = format!("xfa_v1_{}", hex::encode(issued.token.as_bytes()));
        Self {
            access_token,
            token_type: "Bearer",
            expires_in: crate::application::external_auth::ACCESS_TOKEN_TTL.as_secs(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExternalErrorCode {
    ExternalAuthenticationFailed,
    ExternalClientAuthenticationFailed,
    ExternalScopeDenied,
    ExternalRequestInvalid,
    ExternalResourceNotFound,
    ExternalAuthUnavailable,
    ExternalServiceUnavailable,
    ExternalInternalError,
}

impl ExternalErrorCode {
    const fn status(self) -> StatusCode {
        match self {
            Self::ExternalRequestInvalid => StatusCode::BAD_REQUEST,
            Self::ExternalAuthenticationFailed | Self::ExternalClientAuthenticationFailed => {
                StatusCode::UNAUTHORIZED
            }
            Self::ExternalScopeDenied => StatusCode::FORBIDDEN,
            Self::ExternalResourceNotFound => StatusCode::NOT_FOUND,
            Self::ExternalAuthUnavailable | Self::ExternalServiceUnavailable => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            Self::ExternalInternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    const fn message(self) -> &'static str {
        match self {
            Self::ExternalRequestInvalid => EXTERNAL_ERROR_MESSAGE,
            Self::ExternalClientAuthenticationFailed => CLIENT_AUTH_FAILURE_MESSAGE,
            Self::ExternalAuthenticationFailed => AUTH_FAILURE_MESSAGE,
            Self::ExternalScopeDenied => SCOPE_DENIED_MESSAGE,
            Self::ExternalResourceNotFound => RESOURCE_NOT_FOUND_MESSAGE,
            Self::ExternalAuthUnavailable => AUTH_UNAVAILABLE_MESSAGE,
            Self::ExternalServiceUnavailable => SERVICE_UNAVAILABLE_MESSAGE,
            Self::ExternalInternalError => INTERNAL_ERROR_MESSAGE,
        }
    }

    const fn challenge(self) -> bool {
        matches!(self, Self::ExternalAuthenticationFailed)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalErrorEnvelope {
    pub error: ExternalErrorBody,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalErrorBody {
    pub code: ExternalErrorCode,
    message: &'static str,
    pub request_id: Uuid,
}

impl ExternalErrorEnvelope {
    pub fn new(code: ExternalErrorCode, request_id: Uuid) -> Self {
        Self {
            error: ExternalErrorBody {
                code,
                message: code.message(),
                request_id,
            },
        }
    }
}

impl IntoResponse for ExternalErrorEnvelope {
    fn into_response(self) -> Response {
        let challenge = self.error.code.challenge();
        let mut response = (
            self.error.code.status(),
            [(header::CACHE_CONTROL, "no-store, private")],
            Json(self),
        )
            .into_response();
        if challenge {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}

fn external_error(code: ExternalErrorCode, request_id: Uuid) -> Response {
    ExternalErrorEnvelope::new(code, request_id).into_response()
}

fn request_id_from_extension(request_id: Option<&RequestId>) -> Uuid {
    request_id
        .and_then(|request_id| request_id.header_value().to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .unwrap_or_else(Uuid::new_v4)
}

fn valid_token_exchange_request(request: &TokenExchangeRequest) -> bool {
    if request.client_id.len() > MAX_CLIENT_ID_LENGTH
        || request.client_secret.len() > MAX_CLIENT_SECRET_LENGTH
    {
        return false;
    }
    if validate_client_id(&request.client_id).is_err() {
        return false;
    }
    PlaintextClientSecret::parse_hex(&request.client_secret).is_ok()
}

pub fn external_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/external/token", post(exchange_token))
        .with_state(state)
}

async fn exchange_token(
    State(state): State<AppState>,
    request_id: Option<Extension<RequestId>>,
    payload: Result<Json<TokenExchangeRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let request_id = request_id_from_extension(request_id.as_ref().map(|extension| &extension.0));
    let request = match payload {
        Ok(Json(request)) if valid_token_exchange_request(&request) => request,
        _ => return external_error(ExternalErrorCode::ExternalRequestInvalid, request_id),
    };

    let Some(service) = state.external_auth.as_ref() else {
        return external_error(ExternalErrorCode::ExternalAuthUnavailable, request_id);
    };
    match service
        .exchange(&request.client_id, &request.client_secret, Utc::now())
        .await
    {
        Ok(issued) => {
            let payload = TokenExchangeResponse::from_issued(issued);
            (
                StatusCode::OK,
                [
                    (header::CACHE_CONTROL, "no-store, private"),
                    (header::PRAGMA, "no-cache"),
                ],
                Json(payload),
            )
                .into_response()
        }
        Err(ExternalTokenExchangeError::InvalidCredential) => external_error(
            ExternalErrorCode::ExternalClientAuthenticationFailed,
            request_id,
        ),
        Err(ExternalTokenExchangeError::Unavailable) => {
            external_error(ExternalErrorCode::ExternalAuthUnavailable, request_id)
        }
    }
}

pub async fn authenticate_external_request(
    state: &AppState,
    authorization_header: &str,
    now: DateTime<Utc>,
) -> Result<ExternalPrincipal, ExternalAuthenticationError> {
    if authorization_header.len() > MAX_AUTHORIZATION_LENGTH || !authorization_header.is_ascii() {
        return Err(ExternalAuthenticationError::InvalidToken);
    }
    let token_text = authorization_header
        .strip_prefix("Bearer ")
        .ok_or(ExternalAuthenticationError::InvalidToken)?;
    PlaintextAccessToken::parse_bearer_text(token_text)
        .map_err(|_| ExternalAuthenticationError::InvalidToken)?;
    let service: &ExternalAuthService = state
        .external_auth
        .as_ref()
        .ok_or(ExternalAuthenticationError::Unavailable)?;
    service.authenticate_bearer(authorization_header, now).await
}

#[derive(Clone)]
pub struct ExternalBearerAuthLayer {
    state: AppState,
}

impl ExternalBearerAuthLayer {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn layer<S>(self, router: Router<S>) -> Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        router.route_layer(middleware::from_fn_with_state(
            self.state,
            external_bearer_middleware,
        ))
    }
}

#[derive(Clone, Copy)]
pub struct ExternalScopeLayer {
    required: ApiClientScope,
}

impl ExternalScopeLayer {
    pub const fn new(required: ApiClientScope) -> Self {
        Self { required }
    }

    pub fn layer<S>(self, router: Router<S>) -> Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        let required = self.required;
        router.route_layer(middleware::from_fn(move |request, next| {
            external_scope_middleware(request, next, required)
        }))
    }
}

async fn external_scope_middleware(
    request: Request,
    next: Next,
    required: ApiClientScope,
) -> Response {
    let request_id = request_id_from_extension(request.extensions().get::<RequestId>());
    let Some(principal) = request.extensions().get::<ExternalPrincipal>() else {
        return external_error(ExternalErrorCode::ExternalAuthenticationFailed, request_id);
    };
    if let Err(ExternalScopeError::Missing) = require_scope(principal, required) {
        return external_error(ExternalErrorCode::ExternalScopeDenied, request_id);
    }
    next.run(request).await
}

async fn external_bearer_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = request_id_from_extension(request.extensions().get::<RequestId>());
    let mut values = request.headers().get_all(header::AUTHORIZATION).iter();
    let Some(value) = values.next() else {
        return external_error(ExternalErrorCode::ExternalAuthenticationFailed, request_id);
    };
    if values.next().is_some() {
        return external_error(ExternalErrorCode::ExternalAuthenticationFailed, request_id);
    }
    let Ok(value) = value.to_str() else {
        return external_error(ExternalErrorCode::ExternalAuthenticationFailed, request_id);
    };
    match authenticate_external_request(&state, value, Utc::now()).await {
        Ok(principal) => {
            request.extensions_mut().insert(principal);
            next.run(request).await
        }
        Err(ExternalAuthenticationError::InvalidToken) => {
            external_error(ExternalErrorCode::ExternalAuthenticationFailed, request_id)
        }
        Err(ExternalAuthenticationError::Unavailable) => {
            external_error(ExternalErrorCode::ExternalAuthUnavailable, request_id)
        }
    }
}
