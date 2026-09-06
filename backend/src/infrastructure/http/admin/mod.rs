use axum::{
    extract::{rejection::JsonRejection, FromRequestParts, State},
    http::{header, request::Parts, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    application::staff_auth::StaffAuthError,
    domain::staff::{PermissionCode, StaffPrincipal},
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
