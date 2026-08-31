use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::NaiveDate;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use uuid::Uuid;

use crate::{
    domain::{
        entities::{CreateSeatHold, FlightSelection, SeatHold},
        repositories::SeatHoldRepositoryError,
        value_objects::{CabinClass, PassengerCounts, SeatNumber},
        DomainError,
    },
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    let frontend_origin = state
        .frontend_origin
        .parse::<HeaderValue>()
        .expect("FRONTEND_ORIGIN must be a valid HTTP header value");
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(frontend_origin))
        .allow_credentials(true)
        .allow_headers([header::CONTENT_TYPE])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]);

    Router::new()
        .route("/health", get(health))
        .route("/api/v1/flights/{flight_id}/seats", get(seat_map))
        .route("/api/v1/seat-holds", post(create_hold))
        .route(
            "/api/v1/seat-holds/{hold_id}/validation",
            post(validate_hold_for_continue),
        )
        .route(
            "/api/v1/seat-holds/{hold_id}",
            get(get_hold).put(replace_seats).delete(release_hold),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeatMapQuery {
    departure: NaiveDate,
    cabin: String,
    hold_id: Option<Uuid>,
}

async fn seat_map(
    State(state): State<AppState>,
    Path(flight_id): Path<String>,
    Query(query): Query<SeatMapQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let selection = FlightSelection {
        flight_id,
        departure_date: query.departure,
        cabin: CabinClass::from_str(&query.cabin).map_err(ApiError::validation)?,
    };
    let owner = match query.hold_id {
        Some(hold_id) => Some((hold_id, token_hash_from_cookie(&headers, hold_id)?)),
        None => None,
    };
    let map = state.seat_holds.seat_map(&selection, owner).await?;
    Ok(Json(map))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PassengerCountsRequest {
    adults: u8,
    children: u8,
    infants: u8,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateHoldRequest {
    flight_id: String,
    departure_date: NaiveDate,
    cabin: String,
    passengers: PassengerCountsRequest,
    seats: Vec<String>,
}

async fn create_hold(
    State(state): State<AppState>,
    Json(request): Json<CreateHoldRequest>,
) -> Result<Response, ApiError> {
    let cabin = CabinClass::from_str(&request.cabin).map_err(ApiError::validation)?;
    let passengers = PassengerCounts::new(
        request.passengers.adults,
        request.passengers.children,
        request.passengers.infants,
    )
    .map_err(ApiError::validation)?;
    let seats = parse_seats(request.seats)?;
    let mut access_token = [0_u8; 32];
    rand::rng().fill_bytes(&mut access_token);
    let token_hash = hash_token(&access_token);

    let hold = state
        .seat_holds
        .create_hold(CreateSeatHold {
            selection: FlightSelection {
                flight_id: request.flight_id,
                departure_date: request.departure_date,
                cabin,
            },
            passengers,
            seats,
            token_hash,
        })
        .await?;
    let max_age = (hold.expires_at - hold.server_time).num_seconds().max(0);
    let cookie = hold_cookie(
        hold.id,
        &hex::encode(access_token),
        max_age,
        state.secure_cookies,
    )?;
    Ok((
        StatusCode::CREATED,
        [(header::SET_COOKIE, cookie)],
        Json(hold),
    )
        .into_response())
}

#[derive(Deserialize)]
struct ReplaceSeatsRequest {
    seats: Vec<String>,
}

async fn replace_seats(
    State(state): State<AppState>,
    Path(hold_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<ReplaceSeatsRequest>,
) -> Result<Json<SeatHold>, ApiError> {
    let token_hash = token_hash_from_cookie(&headers, hold_id)?;
    let hold = state
        .seat_holds
        .replace_seats(hold_id, token_hash, parse_seats(request.seats)?)
        .await?;
    Ok(Json(hold))
}

async fn get_hold(
    State(state): State<AppState>,
    Path(hold_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<SeatHold>, ApiError> {
    let token_hash = token_hash_from_cookie(&headers, hold_id)?;
    Ok(Json(state.seat_holds.get_hold(hold_id, token_hash).await?))
}

async fn validate_hold_for_continue(
    State(state): State<AppState>,
    Path(hold_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<SeatHold>, ApiError> {
    let token_hash = token_hash_from_cookie(&headers, hold_id)?;
    Ok(Json(
        state
            .seat_holds
            .validate_hold_for_continue(hold_id, token_hash)
            .await?,
    ))
}

async fn release_hold(
    State(state): State<AppState>,
    Path(hold_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let token_hash = token_hash_from_cookie(&headers, hold_id)?;
    state.seat_holds.release_hold(hold_id, token_hash).await?;
    let cookie = hold_cookie(hold_id, "", 0, state.secure_cookies)?;
    Ok((StatusCode::NO_CONTENT, [(header::SET_COOKIE, cookie)]).into_response())
}

fn parse_seats(values: Vec<String>) -> Result<Vec<SeatNumber>, ApiError> {
    values
        .iter()
        .map(|value| SeatNumber::parse(value).map_err(ApiError::validation))
        .collect()
}

fn cookie_name(hold_id: Uuid) -> String {
    format!("x_fly_hold_{}", hold_id.simple())
}

fn hold_cookie(
    hold_id: Uuid,
    value: &str,
    max_age: i64,
    secure: bool,
) -> Result<HeaderValue, ApiError> {
    let secure_attribute = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{}={value}; Path=/api/v1; Max-Age={max_age}; HttpOnly; SameSite=Lax{secure_attribute}",
        cookie_name(hold_id)
    ))
    .map_err(|_| ApiError::internal())
}

fn token_hash_from_cookie(headers: &HeaderMap, hold_id: Uuid) -> Result<[u8; 32], ApiError> {
    let name = cookie_name(hold_id);
    let raw_cookie = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(ApiError::unauthorized)?;
    let value = raw_cookie
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find_map(|(cookie_name, value)| (cookie_name == name).then_some(value))
        .ok_or_else(ApiError::unauthorized)?;
    let token = hex::decode(value).map_err(|_| ApiError::unauthorized())?;
    if token.len() != 32 {
        return Err(ApiError::unauthorized());
    }
    Ok(hash_token(&token))
}

fn hash_token(token: &[u8]) -> [u8; 32] {
    Sha256::digest(token).into()
}

#[derive(Serialize)]
struct ApiErrorEnvelope {
    error: ApiErrorBody,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiErrorBody {
    code: &'static str,
    message: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    conflicting_seats: Vec<String>,
}

struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
    conflicting_seats: Vec<String>,
}

impl ApiError {
    fn validation(_error: DomainError) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "VALIDATION_ERROR",
            message: "The seat hold request is invalid.",
            conflicting_seats: Vec::new(),
        }
    }

    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "HOLD_UNAUTHORIZED",
            message: "Seat hold authorization is missing or invalid.",
            conflicting_seats: Vec::new(),
        }
    }

    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL_ERROR",
            message: "The seat hold service is temporarily unavailable.",
            conflicting_seats: Vec::new(),
        }
    }
}

impl From<SeatHoldRepositoryError> for ApiError {
    fn from(error: SeatHoldRepositoryError) -> Self {
        match error {
            SeatHoldRepositoryError::FlightNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "FLIGHT_NOT_FOUND",
                message: "The selected flight could not be found.",
                conflicting_seats: Vec::new(),
            },
            SeatHoldRepositoryError::CabinUnavailable => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "CABIN_UNAVAILABLE",
                message: "The selected cabin is not available for this flight.",
                conflicting_seats: Vec::new(),
            },
            SeatHoldRepositoryError::SeatNotFound(seats) => Self {
                status: StatusCode::NOT_FOUND,
                code: "SEAT_NOT_FOUND",
                message: "One or more seats do not exist for this flight and cabin.",
                conflicting_seats: seats,
            },
            SeatHoldRepositoryError::SeatCountMismatch => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "SEAT_COUNT_MISMATCH",
                message: "The seat count must match adults plus children.",
                conflicting_seats: Vec::new(),
            },
            SeatHoldRepositoryError::SeatConflict(seats) => Self {
                status: StatusCode::CONFLICT,
                code: "SEAT_UNAVAILABLE",
                message: "One or more seats were just taken.",
                conflicting_seats: seats,
            },
            SeatHoldRepositoryError::HoldNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "HOLD_NOT_FOUND",
                message: "The seat hold could not be found.",
                conflicting_seats: Vec::new(),
            },
            SeatHoldRepositoryError::Unauthorized => Self::unauthorized(),
            SeatHoldRepositoryError::HoldExpired => Self {
                status: StatusCode::GONE,
                code: "HOLD_EXPIRED",
                message: "The seat hold has expired.",
                conflicting_seats: Vec::new(),
            },
            SeatHoldRepositoryError::HoldReleased => Self {
                status: StatusCode::GONE,
                code: "HOLD_RELEASED",
                message: "The seat hold has been released.",
                conflicting_seats: Vec::new(),
            },
            SeatHoldRepositoryError::HoldConsumed => Self {
                status: StatusCode::CONFLICT,
                code: "HOLD_CONSUMED",
                message: "The seat hold has already been consumed.",
                conflicting_seats: Vec::new(),
            },
            SeatHoldRepositoryError::Infrastructure(error) => {
                tracing::error!(?error, "seat hold repository failure");
                Self::internal()
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorEnvelope {
                error: ApiErrorBody {
                    code: self.code,
                    message: self.message,
                    conflicting_seats: self.conflicting_seats,
                },
            }),
        )
            .into_response()
    }
}
