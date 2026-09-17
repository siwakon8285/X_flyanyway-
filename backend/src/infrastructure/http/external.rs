use axum::{
    extract::{Extension, Json, Path, Query, Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tower_http::request_id::RequestId;
use tracing::Span;
use uuid::Uuid;

use crate::{
    application::{
        api_client::validate_client_id,
        external_analytics::{
            ExternalAnalyticsFilter, ExternalAnalyticsFilterError, ExternalAnalyticsService,
        },
        external_auth::{
            record_external_auth_anonymous, record_external_auth_attributed,
            record_external_auth_succeeded, record_external_scope_denied, require_scope,
            AnonymousExternalAuthDiagnostic, AttributedExternalAuthDiagnostic, BearerRejection,
            ExternalAuthFlow, ExternalAuthService, ExternalBearerDecision, ExternalScopeError,
            ExternalTokenExchangeDecision, TokenExchangeRejection,
        },
        external_flights::ExternalFlightService,
        flight::PublicFlightFilter,
    },
    domain::{
        api_client::ApiClientScope,
        external_api::{
            ExternalAuthenticationError, ExternalPrincipal, ExternalTokenExchangeError,
            IssuedAccessToken, PlaintextAccessToken, PlaintextClientSecret,
        },
        flight::FlightManagementError,
        value_objects::CabinClass,
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

fn record_auth_diagnostic(flow: ExternalAuthFlow, diagnostic: AnonymousExternalAuthDiagnostic) {
    record_external_auth_anonymous(&Span::current(), flow, diagnostic);
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
    let protected: Router<AppState> = Router::new()
        .route("/api/v1/external/flights", get(search_external_flights))
        .route(
            "/api/v1/external/flights/{flight_public_id}",
            get(detail_external_flight),
        );
    let protected = ExternalScopeLayer::new(ApiClientScope::FlightsRead).layer(protected);
    let protected = ExternalBearerAuthLayer::new(state.clone()).layer(protected);
    let analytics: Router<AppState> = Router::new().route(
        "/api/v1/external/analytics/summary",
        get(external_analytics_summary),
    );
    let analytics = ExternalScopeLayer::new(ApiClientScope::AnalyticsRead).layer(analytics);
    let analytics = ExternalBearerAuthLayer::new(state.clone()).layer(analytics);
    Router::new()
        .route("/api/v1/external/token", post(exchange_token))
        .merge(protected)
        .merge(analytics)
        .with_state(state)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExternalFlightQuery {
    origin: String,
    destination: String,
    departure: NaiveDate,
    cabin: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExternalFlightDetailQuery {
    departure: NaiveDate,
    cabin: String,
}

fn external_flight_filter(query: ExternalFlightQuery) -> Result<PublicFlightFilter, ()> {
    let origin = query.origin.trim().to_uppercase();
    let destination = query.destination.trim().to_uppercase();
    if origin.len() != 3
        || destination.len() != 3
        || origin == destination
        || !origin.bytes().all(|byte| byte.is_ascii_uppercase())
        || !destination.bytes().all(|byte| byte.is_ascii_uppercase())
    {
        return Err(());
    }
    let cabin = CabinClass::parse_customer_booking(&query.cabin).map_err(|_| ())?;
    Ok(PublicFlightFilter {
        origin,
        destination,
        departure: query.departure,
        cabin,
    })
}

fn external_flight_service(state: &AppState) -> Result<&ExternalFlightService, ExternalErrorCode> {
    state
        .external_flights
        .as_ref()
        .ok_or(ExternalErrorCode::ExternalServiceUnavailable)
}

fn external_analytics_service(
    state: &AppState,
) -> Result<&ExternalAnalyticsService, ExternalErrorCode> {
    state
        .external_analytics
        .as_ref()
        .ok_or(ExternalErrorCode::ExternalServiceUnavailable)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExternalAnalyticsQuery {
    from: Option<String>,
    to: Option<String>,
    route: Option<String>,
    cabin: Option<String>,
}

async fn external_analytics_summary(
    State(state): State<AppState>,
    request_id: Option<Extension<RequestId>>,
    query: Result<Query<ExternalAnalyticsQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let request_id = request_id_from_extension(request_id.as_ref().map(|extension| &extension.0));
    let Ok(Query(query)) = query else {
        return external_error(ExternalErrorCode::ExternalRequestInvalid, request_id);
    };
    let filter = match ExternalAnalyticsFilter::parse(
        query.from.as_deref(),
        query.to.as_deref(),
        query.route.as_deref(),
        query.cabin.as_deref(),
        Utc::now(),
    ) {
        Ok(filter) => filter,
        Err(
            ExternalAnalyticsFilterError::InvalidDate
            | ExternalAnalyticsFilterError::InvalidRange
            | ExternalAnalyticsFilterError::InvalidRoute
            | ExternalAnalyticsFilterError::InvalidCabin,
        ) => return external_error(ExternalErrorCode::ExternalRequestInvalid, request_id),
    };
    let Ok(service) = external_analytics_service(&state) else {
        return external_error(ExternalErrorCode::ExternalServiceUnavailable, request_id);
    };
    match service.summary(filter, Utc::now()).await {
        Ok(summary) => Json(summary).into_response(),
        Err(
            crate::application::external_analytics::ExternalAnalyticsRepositoryError::Infrastructure(
                _,
            ),
        ) => external_error(ExternalErrorCode::ExternalServiceUnavailable, request_id),
        Err(crate::application::external_analytics::ExternalAnalyticsRepositoryError::InconsistentAggregate) => {
            external_error(ExternalErrorCode::ExternalInternalError, request_id)
        }
    }
}

async fn search_external_flights(
    State(state): State<AppState>,
    request_id: Option<Extension<RequestId>>,
    query: Result<Query<ExternalFlightQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let request_id = request_id_from_extension(request_id.as_ref().map(|extension| &extension.0));
    let Ok(Query(query)) = query else {
        return external_error(ExternalErrorCode::ExternalRequestInvalid, request_id);
    };
    let Ok(filter) = external_flight_filter(query) else {
        return external_error(ExternalErrorCode::ExternalRequestInvalid, request_id);
    };
    let Ok(service) = external_flight_service(&state) else {
        return external_error(ExternalErrorCode::ExternalServiceUnavailable, request_id);
    };
    match service.search(filter).await {
        Ok(page) => Json(page).into_response(),
        Err(FlightManagementError::TravelDateOutsideWindow) => {
            external_error(ExternalErrorCode::ExternalRequestInvalid, request_id)
        }
        Err(_) => external_error(ExternalErrorCode::ExternalServiceUnavailable, request_id),
    }
}

async fn detail_external_flight(
    State(state): State<AppState>,
    request_id: Option<Extension<RequestId>>,
    Path(flight_public_id): Path<String>,
    query: Result<Query<ExternalFlightDetailQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let request_id = request_id_from_extension(request_id.as_ref().map(|extension| &extension.0));
    let Ok(Query(query)) = query else {
        return external_error(ExternalErrorCode::ExternalRequestInvalid, request_id);
    };
    let Ok(cabin) = CabinClass::parse_customer_booking(&query.cabin) else {
        return external_error(ExternalErrorCode::ExternalRequestInvalid, request_id);
    };
    let Ok(service) = external_flight_service(&state) else {
        return external_error(ExternalErrorCode::ExternalServiceUnavailable, request_id);
    };
    match service
        .detail(&flight_public_id, query.departure, cabin)
        .await
    {
        Ok(flight) => Json(flight).into_response(),
        Err(FlightManagementError::NotFound) => {
            external_error(ExternalErrorCode::ExternalResourceNotFound, request_id)
        }
        Err(FlightManagementError::TravelDateOutsideWindow) => {
            external_error(ExternalErrorCode::ExternalRequestInvalid, request_id)
        }
        Err(_) => external_error(ExternalErrorCode::ExternalServiceUnavailable, request_id),
    }
}

async fn exchange_token(
    State(state): State<AppState>,
    request_id: Option<Extension<RequestId>>,
    payload: Result<Json<TokenExchangeRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let request_id = request_id_from_extension(request_id.as_ref().map(|extension| &extension.0));
    let request = match payload {
        Ok(Json(request)) if valid_token_exchange_request(&request) => request,
        _ => {
            record_auth_diagnostic(
                ExternalAuthFlow::TokenExchange,
                AnonymousExternalAuthDiagnostic::Malformed,
            );
            return external_error(ExternalErrorCode::ExternalRequestInvalid, request_id);
        }
    };

    let Some(service) = state.external_auth.as_ref() else {
        return external_error(ExternalErrorCode::ExternalAuthUnavailable, request_id);
    };
    match service
        .exchange_with_outcome(&request.client_id, &request.client_secret, Utc::now())
        .await
    {
        Ok(ExternalTokenExchangeDecision::Issued { issued, client_id }) => {
            if let Some(client_id) = client_id {
                record_external_auth_succeeded(
                    &Span::current(),
                    ExternalAuthFlow::TokenExchange,
                    &client_id,
                );
            }
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
        Ok(ExternalTokenExchangeDecision::Rejected { reason, client_id }) => {
            match reason {
                TokenExchangeRejection::UnknownClient => record_auth_diagnostic(
                    ExternalAuthFlow::TokenExchange,
                    AnonymousExternalAuthDiagnostic::UnknownClient,
                ),
                TokenExchangeRejection::SecretMismatch => {
                    if let Some(client_id) = client_id.as_ref() {
                        record_external_auth_attributed(
                            &Span::current(),
                            ExternalAuthFlow::TokenExchange,
                            AttributedExternalAuthDiagnostic::SecretMismatch,
                            client_id,
                        );
                    }
                }
                TokenExchangeRejection::Suspended => {
                    if let Some(client_id) = client_id.as_ref() {
                        record_external_auth_attributed(
                            &Span::current(),
                            ExternalAuthFlow::TokenExchange,
                            AttributedExternalAuthDiagnostic::Suspended,
                            client_id,
                        );
                    }
                }
                TokenExchangeRejection::Revoked => {
                    if let Some(client_id) = client_id.as_ref() {
                        record_external_auth_attributed(
                            &Span::current(),
                            ExternalAuthFlow::TokenExchange,
                            AttributedExternalAuthDiagnostic::Revoked,
                            client_id,
                        );
                    }
                }
                TokenExchangeRejection::CredentialUnavailable => {}
            }
            external_error(
                ExternalErrorCode::ExternalClientAuthenticationFailed,
                request_id,
            )
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
    match authenticate_external_request_with_outcome(state, authorization_header, now).await? {
        ExternalBearerDecision::Authenticated { principal, .. } => Ok(principal),
        ExternalBearerDecision::Rejected { .. } => Err(ExternalAuthenticationError::InvalidToken),
    }
}

async fn authenticate_external_request_with_outcome(
    state: &AppState,
    authorization_header: &str,
    now: DateTime<Utc>,
) -> Result<ExternalBearerDecision, ExternalAuthenticationError> {
    if authorization_header.len() > MAX_AUTHORIZATION_LENGTH || !authorization_header.is_ascii() {
        record_auth_diagnostic(
            ExternalAuthFlow::Bearer,
            AnonymousExternalAuthDiagnostic::Malformed,
        );
        return Err(ExternalAuthenticationError::InvalidToken);
    }
    let token_text = authorization_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            record_auth_diagnostic(
                ExternalAuthFlow::Bearer,
                AnonymousExternalAuthDiagnostic::Malformed,
            );
            ExternalAuthenticationError::InvalidToken
        })?;
    PlaintextAccessToken::parse_bearer_text(token_text).map_err(|_| {
        record_auth_diagnostic(
            ExternalAuthFlow::Bearer,
            AnonymousExternalAuthDiagnostic::Malformed,
        );
        ExternalAuthenticationError::InvalidToken
    })?;
    let service: &ExternalAuthService = state
        .external_auth
        .as_ref()
        .ok_or(ExternalAuthenticationError::Unavailable)?;
    service
        .authenticate_bearer_with_outcome(authorization_header, now)
        .await
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
        record_external_scope_denied(&Span::current(), principal.client_id(), required);
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
        record_auth_diagnostic(
            ExternalAuthFlow::Bearer,
            AnonymousExternalAuthDiagnostic::Missing,
        );
        return external_error(ExternalErrorCode::ExternalAuthenticationFailed, request_id);
    };
    if values.next().is_some() {
        record_auth_diagnostic(
            ExternalAuthFlow::Bearer,
            AnonymousExternalAuthDiagnostic::Malformed,
        );
        return external_error(ExternalErrorCode::ExternalAuthenticationFailed, request_id);
    }
    let Ok(value) = value.to_str() else {
        record_auth_diagnostic(
            ExternalAuthFlow::Bearer,
            AnonymousExternalAuthDiagnostic::Malformed,
        );
        return external_error(ExternalErrorCode::ExternalAuthenticationFailed, request_id);
    };
    match authenticate_external_request_with_outcome(&state, value, Utc::now()).await {
        Ok(ExternalBearerDecision::Authenticated {
            principal,
            client_id,
        }) => {
            record_external_auth_succeeded(&Span::current(), ExternalAuthFlow::Bearer, &client_id);
            request.extensions_mut().insert(principal);
            next.run(request).await
        }
        Ok(ExternalBearerDecision::Rejected { reason, client_id }) => {
            match reason {
                BearerRejection::UnknownToken => {}
                BearerRejection::Expired => {
                    if let Some(client_id) = client_id.as_ref() {
                        record_external_auth_attributed(
                            &Span::current(),
                            ExternalAuthFlow::Bearer,
                            AttributedExternalAuthDiagnostic::Expired,
                            client_id,
                        );
                    }
                }
                BearerRejection::Suspended => {
                    if let Some(client_id) = client_id.as_ref() {
                        record_external_auth_attributed(
                            &Span::current(),
                            ExternalAuthFlow::Bearer,
                            AttributedExternalAuthDiagnostic::Suspended,
                            client_id,
                        );
                    }
                }
                BearerRejection::Revoked => {
                    if let Some(client_id) = client_id.as_ref() {
                        record_external_auth_attributed(
                            &Span::current(),
                            ExternalAuthFlow::Bearer,
                            AttributedExternalAuthDiagnostic::Revoked,
                            client_id,
                        );
                    }
                }
            }
            external_error(ExternalErrorCode::ExternalAuthenticationFailed, request_id)
        }
        Err(ExternalAuthenticationError::InvalidToken) => {
            external_error(ExternalErrorCode::ExternalAuthenticationFailed, request_id)
        }
        Err(ExternalAuthenticationError::Unavailable) => {
            external_error(ExternalErrorCode::ExternalAuthUnavailable, request_id)
        }
    }
}
