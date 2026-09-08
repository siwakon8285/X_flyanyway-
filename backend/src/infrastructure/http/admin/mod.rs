use axum::{
    extract::{
        rejection::{JsonRejection, QueryRejection},
        FromRequestParts, Path, Query, State,
    },
    http::{header, request::Parts, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    application::staff_auth::StaffAuthError,
    application::{analytics::AnalyticsFilter, flight::FlightListFilter},
    domain::{
        flight::{FlightCommand, FlightManagementError, FlightStatus},
        staff::{PermissionCode, StaffPrincipal},
    },
    state::AppState,
};

use super::browser_security::browser_mutation_is_trusted;

const STAFF_COOKIE: &str = "x_fly_staff_session";
const STAFF_SESSION_TTL_SECONDS: i64 = 60 * 60;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/auth/login", post(login))
        .route("/api/v1/admin/auth/session", get(session))
        .route("/api/v1/admin/auth/logout", post(logout))
        .route("/api/v1/admin/dashboard", get(dashboard))
        .route(
            "/api/v1/admin/flights",
            get(list_flights).post(create_flight),
        )
        .route(
            "/api/v1/admin/flights/reference-data",
            get(flight_reference_data),
        )
        .route(
            "/api/v1/admin/flights/{flight_id}",
            get(flight_detail).put(update_flight),
        )
        .route(
            "/api/v1/admin/flights/{flight_id}/cancel",
            post(cancel_flight),
        )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FlightListQuery {
    search: Option<String>,
    origin: Option<String>,
    destination: Option<String>,
    date: Option<String>,
    status: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn list_flights(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    query: Result<Query<FlightListQuery>, QueryRejection>,
) -> Response {
    let result = async {
        staff
            .require(PermissionCode::FlightsRead)
            .map_err(|_| AdminApiError::permission_denied())?;
        let query = query.map_err(|_| AdminApiError::flight_validation())?.0;
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);
        if !(1..=100).contains(&limit)
            || offset < 0
            || query.search.as_ref().is_some_and(|value| value.len() > 40)
        {
            return Err(AdminApiError::flight_validation());
        }
        let airport = |value: Option<String>| -> Result<Option<String>, AdminApiError> {
            value
                .map(|value| {
                    let normalized = value.trim().to_uppercase();
                    (normalized.len() == 3
                        && normalized.bytes().all(|byte| byte.is_ascii_uppercase()))
                    .then_some(normalized)
                    .ok_or_else(AdminApiError::flight_validation)
                })
                .transpose()
        };
        let date = query
            .date
            .as_deref()
            .map(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d"))
            .transpose()
            .map_err(|_| AdminApiError::flight_validation())?;
        let status = query
            .status
            .as_deref()
            .map(|value| match value {
                "SCHEDULED" => Ok(FlightStatus::Scheduled),
                "CANCELLED" => Ok(FlightStatus::Cancelled),
                _ => Err(AdminApiError::flight_validation()),
            })
            .transpose()?;
        let flights = state
            .flights
            .as_ref()
            .ok_or_else(AdminApiError::flight_unavailable)?;
        let page = flights
            .list(FlightListFilter {
                search: query
                    .search
                    .map(|value| value.trim().to_owned())
                    .filter(|value| !value.is_empty()),
                origin: airport(query.origin)?,
                destination: airport(query.destination)?,
                date,
                status,
                limit,
                offset,
            })
            .await
            .map_err(AdminApiError::from_flight)?;
        Ok::<_, AdminApiError>(Json(page).into_response())
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

async fn flight_detail(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    Path(flight_id): Path<uuid::Uuid>,
) -> Response {
    let result = async {
        staff
            .require(PermissionCode::FlightsRead)
            .map_err(|_| AdminApiError::permission_denied())?;
        let flights = state
            .flights
            .as_ref()
            .ok_or_else(AdminApiError::flight_unavailable)?;
        Ok::<_, AdminApiError>(
            Json(
                flights
                    .detail(flight_id)
                    .await
                    .map_err(AdminApiError::from_flight)?,
            )
            .into_response(),
        )
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

async fn flight_reference_data(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
) -> Response {
    let result = async {
        staff
            .require(PermissionCode::FlightsRead)
            .map_err(|_| AdminApiError::permission_denied())?;
        let flights = state
            .flights
            .as_ref()
            .ok_or_else(AdminApiError::flight_unavailable)?;
        Ok::<_, AdminApiError>(
            Json(
                flights
                    .reference_data()
                    .await
                    .map_err(AdminApiError::from_flight)?,
            )
            .into_response(),
        )
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

async fn create_flight(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    headers: HeaderMap,
    payload: Result<Json<FlightCommand>, JsonRejection>,
) -> Response {
    let result = async {
        let actor = staff
            .require(PermissionCode::FlightsWrite)
            .map_err(|_| AdminApiError::permission_denied())?;
        trusted_flight_mutation(&state, &headers)?;
        let command = payload.map_err(|_| AdminApiError::flight_validation())?.0;
        let flights = state
            .flights
            .as_ref()
            .ok_or_else(AdminApiError::flight_unavailable)?;
        Ok::<_, AdminApiError>(
            (
                StatusCode::CREATED,
                Json(
                    flights
                        .create(actor.staff_user_id(), command)
                        .await
                        .map_err(AdminApiError::from_flight)?,
                ),
            )
                .into_response(),
        )
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateFlightRequest {
    version: i64,
    #[serde(flatten)]
    command: FlightCommand,
}

async fn update_flight(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    Path(flight_id): Path<uuid::Uuid>,
    headers: HeaderMap,
    payload: Result<Json<UpdateFlightRequest>, JsonRejection>,
) -> Response {
    let result = async {
        let actor = staff
            .require(PermissionCode::FlightsWrite)
            .map_err(|_| AdminApiError::permission_denied())?;
        trusted_flight_mutation(&state, &headers)?;
        let request = payload.map_err(|_| AdminApiError::flight_validation())?.0;
        if request.version < 1 {
            return Err(AdminApiError::flight_validation());
        }
        let flights = state
            .flights
            .as_ref()
            .ok_or_else(AdminApiError::flight_unavailable)?;
        Ok::<_, AdminApiError>(
            Json(
                flights
                    .update(
                        actor.staff_user_id(),
                        flight_id,
                        request.version,
                        request.command,
                    )
                    .await
                    .map_err(AdminApiError::from_flight)?,
            )
            .into_response(),
        )
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CancelFlightRequest {
    version: i64,
}

async fn cancel_flight(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    Path(flight_id): Path<uuid::Uuid>,
    headers: HeaderMap,
    payload: Result<Json<CancelFlightRequest>, JsonRejection>,
) -> Response {
    let result = async {
        let actor = staff
            .require(PermissionCode::FlightsWrite)
            .map_err(|_| AdminApiError::permission_denied())?;
        trusted_flight_mutation(&state, &headers)?;
        let request = payload.map_err(|_| AdminApiError::flight_validation())?.0;
        if request.version < 1 {
            return Err(AdminApiError::flight_validation());
        }
        let flights = state
            .flights
            .as_ref()
            .ok_or_else(AdminApiError::flight_unavailable)?;
        Ok::<_, AdminApiError>(
            Json(
                flights
                    .cancel(actor.staff_user_id(), flight_id, request.version)
                    .await
                    .map_err(AdminApiError::from_flight)?,
            )
            .into_response(),
        )
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

fn trusted_flight_mutation(state: &AppState, headers: &HeaderMap) -> Result<(), AdminApiError> {
    browser_mutation_is_trusted(headers, &state.frontend_origin)
        .then_some(())
        .ok_or_else(AdminApiError::forbidden_origin)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DashboardQuery {
    from: Option<String>,
    to: Option<String>,
    route: Option<String>,
    cabin: Option<String>,
    provider: Option<String>,
}

async fn dashboard(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    query: Result<Query<DashboardQuery>, QueryRejection>,
) -> Response {
    let result = async {
        for permission in [
            PermissionCode::DashboardRead,
            PermissionCode::AnalyticsRead,
            PermissionCode::ReportsRead,
        ] {
            staff
                .require(permission)
                .map_err(|_| AdminApiError::permission_denied())?;
        }
        let query = query
            .map_err(|_| AdminApiError::dashboard_filter_invalid())?
            .0;
        let filter = AnalyticsFilter::parse(
            query.from.as_deref(),
            query.to.as_deref(),
            query.route.as_deref(),
            query.cabin.as_deref(),
            query.provider.as_deref(),
            Utc::now(),
        )
        .map_err(|_| AdminApiError::dashboard_filter_invalid())?;
        let repository = state
            .analytics
            .as_ref()
            .ok_or_else(AdminApiError::dashboard_unavailable)?;
        let report = repository
            .dashboard(&filter)
            .await
            .map_err(|_| AdminApiError::dashboard_unavailable())?;
        Ok::<_, AdminApiError>(Json(report).into_response())
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LoginRequest {
    email: String,
    password: String,
}

async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Response {
    let result = async {
        if !browser_mutation_is_trusted(&headers, &state.frontend_origin) {
            return Err(AdminApiError::forbidden_origin());
        }
        let request = payload.map_err(|_| AdminApiError::validation())?.0;
        if request.email.len() > 254 || request.password.is_empty() || request.password.len() > 1024
        {
            return Err(AdminApiError::validation());
        }
        let auth = state
            .staff_auth
            .as_ref()
            .ok_or_else(AdminApiError::internal)?;
        let login = auth
            .login(&request.email, &request.password)
            .await
            .map_err(AdminApiError::from)?;
        let cookie = staff_cookie(
            &login.token,
            STAFF_SESSION_TTL_SECONDS,
            state.secure_cookies,
        )?;
        Ok::<_, AdminApiError>(
            (
                [(header::SET_COOKIE, cookie)],
                Json(PrincipalResponse::from(&login.principal)),
            )
                .into_response(),
        )
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

async fn session(AuthenticatedStaff(principal): AuthenticatedStaff) -> Response {
    private_no_store(Json(PrincipalResponse::from(&principal)).into_response())
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let result = async {
        if !browser_mutation_is_trusted(&headers, &state.frontend_origin) {
            return Err(AdminApiError::forbidden_origin());
        }
        if let (Some(auth), Some(token)) = (state.staff_auth.as_ref(), staff_token(&headers)) {
            auth.logout(token).await.map_err(AdminApiError::from)?;
        }
        let cookie = staff_cookie("", 0, state.secure_cookies)?;
        Ok::<_, AdminApiError>(
            (StatusCode::NO_CONTENT, [(header::SET_COOKIE, cookie)]).into_response(),
        )
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

pub fn permission_response(
    principal: Option<&StaffPrincipal>,
    required: PermissionCode,
) -> Response {
    private_no_store(match principal {
        None => AdminApiError::unauthenticated().into_response(),
        Some(principal) if !principal.can(required) => {
            AdminApiError::permission_denied().into_response()
        }
        Some(_) => StatusCode::NO_CONTENT.into_response(),
    })
}

/// Durable staff authentication extractor for all future human-admin handlers.
/// Every extraction resolves the opaque session and current account/RBAC state
/// from PostgreSQL, so disable, revocation, and grant changes take effect at once.
pub struct AuthenticatedStaff(pub StaffPrincipal);

impl AuthenticatedStaff {
    pub fn require(&self, permission: PermissionCode) -> Result<&StaffPrincipal, PermissionDenied> {
        if self.0.can(permission) {
            Ok(&self.0)
        } else {
            Err(PermissionDenied)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PermissionDenied;

impl IntoResponse for PermissionDenied {
    fn into_response(self) -> Response {
        private_no_store(AdminApiError::permission_denied().into_response())
    }
}

impl FromRequestParts<AppState> for AuthenticatedStaff {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let result = async {
            let token = staff_token(&parts.headers).ok_or_else(AdminApiError::unauthenticated)?;
            let auth = state
                .staff_auth
                .as_ref()
                .ok_or_else(AdminApiError::internal)?;
            auth.authenticate(token)
                .await
                .map(AuthenticatedStaff)
                .map_err(AdminApiError::from)
        }
        .await;
        result.map_err(|error| private_no_store(error.into_response()))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PrincipalResponse {
    email: String,
    roles: Vec<&'static str>,
    permissions: Vec<&'static str>,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl From<&StaffPrincipal> for PrincipalResponse {
    fn from(principal: &StaffPrincipal) -> Self {
        Self {
            email: principal.email().to_owned(),
            roles: principal.roles().iter().map(|role| role.as_str()).collect(),
            permissions: principal
                .permissions()
                .iter()
                .map(|permission| permission.as_str())
                .collect(),
            expires_at: principal.expires_at(),
        }
    }
}

fn staff_cookie(value: &str, max_age: i64, secure: bool) -> Result<HeaderValue, AdminApiError> {
    let secure = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{STAFF_COOKIE}={value}; Path=/admin; Max-Age={max_age}; HttpOnly; SameSite=Strict{secure}"
    ))
    .map_err(|_| AdminApiError::internal())
}

fn staff_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find_map(|(name, value)| (name == STAFF_COOKIE).then_some(value))
}

#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: &'static str,
}

struct AdminApiError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
}

impl AdminApiError {
    fn validation() -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "STAFF_LOGIN_VALIDATION_FAILED",
            message: "The login request is invalid.",
        }
    }
    fn forbidden_origin() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "STAFF_ORIGIN_REJECTED",
            message: "The request origin is not allowed.",
        }
    }
    fn login_failed() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "STAFF_LOGIN_FAILED",
            message: "Email or password is incorrect.",
        }
    }
    fn throttled() -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "STAFF_LOGIN_THROTTLED",
            message: "Unable to sign in. Try again later.",
        }
    }
    fn unauthenticated() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "STAFF_AUTHENTICATION_REQUIRED",
            message: "Staff authentication is required.",
        }
    }
    fn permission_denied() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "STAFF_PERMISSION_DENIED",
            message: "You do not have permission to perform this action.",
        }
    }
    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "STAFF_AUTH_UNAVAILABLE",
            message: "Staff authentication is temporarily unavailable.",
        }
    }
    fn dashboard_filter_invalid() -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "DASHBOARD_FILTER_INVALID",
            message: "The dashboard filters are invalid.",
        }
    }
    fn dashboard_unavailable() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "DASHBOARD_UNAVAILABLE",
            message: "Executive analytics are temporarily unavailable.",
        }
    }
    fn flight_validation() -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "FLIGHT_VALIDATION_FAILED",
            message: "Review the flight fields and try again.",
        }
    }
    fn flight_unavailable() -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "FLIGHT_MANAGEMENT_UNAVAILABLE",
            message: "Flight management is temporarily unavailable.",
        }
    }
    fn from_flight(error: FlightManagementError) -> Self {
        match error {
            FlightManagementError::NotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "FLIGHT_NOT_FOUND",
                message: "The flight was not found.",
            },
            FlightManagementError::Validation => Self::flight_validation(),
            FlightManagementError::Duplicate => Self {
                status: StatusCode::CONFLICT,
                code: "FLIGHT_DUPLICATE",
                message: "That flight number is already in use.",
            },
            FlightManagementError::Conflict => Self {
                status: StatusCode::CONFLICT,
                code: "FLIGHT_STALE_VERSION",
                message: "The flight changed after this view was loaded.",
            },
            FlightManagementError::StructuralConflict => Self {
                status: StatusCode::CONFLICT,
                code: "FLIGHT_STRUCTURAL_CONFLICT",
                message: "Structural fields cannot change after inventory has been materialized.",
            },
            FlightManagementError::InvalidStatus => Self {
                status: StatusCode::CONFLICT,
                code: "FLIGHT_STATUS_CONFLICT",
                message: "The flight status does not allow this operation.",
            },
            FlightManagementError::Infrastructure => Self::flight_unavailable(),
        }
    }
}

impl From<StaffAuthError> for AdminApiError {
    fn from(error: StaffAuthError) -> Self {
        match error {
            StaffAuthError::InvalidCredentials => Self::login_failed(),
            StaffAuthError::Throttled => Self::throttled(),
            StaffAuthError::Unauthenticated => Self::unauthenticated(),
            _ => Self::internal(),
        }
    }
}

impl IntoResponse for AdminApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorEnvelope {
                error: ErrorBody {
                    code: self.code,
                    message: self.message,
                },
            }),
        )
            .into_response()
    }
}

fn private_no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, private"),
    );
    response
}
