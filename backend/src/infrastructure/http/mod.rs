use std::str::FromStr;

use axum::{
    body::Bytes,
    extract::{rejection::JsonRejection, Path, Query, State},
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
        extras::ExtraSelectionInput,
        passengers::{PassengerFieldError, PassengerInput},
        payment::{CreatePaymentRequest, PaymentMethod, PaymentSimulationOutcome},
        repositories::{
            ExtraRepositoryError, PassengerRepositoryError, PaymentRepositoryError,
            ReviewRepositoryError, SeatHoldRepositoryError, TicketRepositoryError,
        },
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
            "/api/v1/seat-holds/{hold_id}/passengers",
            get(get_passengers).put(save_passengers),
        )
        .route(
            "/api/v1/seat-holds/{hold_id}/extras",
            get(get_extras).put(save_extras),
        )
        .route("/api/v1/seat-holds/{hold_id}/review", get(get_review))
        .route("/api/v1/seat-holds/{hold_id}/payment", get(get_payment))
        .route(
            "/api/v1/seat-holds/{hold_id}/payment-attempts",
            post(create_payment_attempt),
        )
        .route(
            "/api/v1/seat-holds/{hold_id}/payment-attempts/{attempt_id}/simulate",
            post(simulate_payment_attempt),
        )
        .route(
            "/api/v1/seat-holds/{hold_id}/payment-attempts/{attempt_id}/ticket",
            get(get_ticket),
        )
        .route(
            "/api/v1/seat-holds/{hold_id}/validation",
            post(validate_hold_for_continue),
        )
        .route(
            "/api/v1/seat-holds/{hold_id}",
            get(get_hold).put(replace_seats).delete(release_hold),
        )
        .route("/api/v1/payments/stripe/webhook", post(stripe_webhook))
        .route("/api/v1/tickets/verify/{token}", get(verify_ticket))
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

async fn get_passengers(
    State(state): State<AppState>,
    Path(hold_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let token_hash = token_hash_from_cookie(&headers, hold_id)?;
    let context = state.passengers.get_passengers(hold_id, token_hash).await?;
    Ok((
        [(header::CACHE_CONTROL, "no-store, private")],
        Json(context),
    ))
}

#[derive(Deserialize)]
struct SavePassengersRequest {
    passengers: Vec<PassengerInput>,
}

async fn save_passengers(
    State(state): State<AppState>,
    Path(hold_id): Path<Uuid>,
    headers: HeaderMap,
    payload: Result<Json<SavePassengersRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let token_hash = token_hash_from_cookie(&headers, hold_id)?;
    let request = payload.map_err(|_| ApiError::passenger_bad_request())?.0;
    let context = state
        .passengers
        .save_passengers(hold_id, token_hash, request.passengers)
        .await?;
    Ok((
        [(header::CACHE_CONTROL, "no-store, private")],
        Json(context),
    ))
}

async fn get_extras(
    State(state): State<AppState>,
    Path(hold_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let token_hash = token_hash_from_cookie(&headers, hold_id)?;
    let context = state.extras.get_extras(hold_id, token_hash).await?;
    Ok((
        [(header::CACHE_CONTROL, "no-store, private")],
        Json(context),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveExtrasRequest {
    selections: Vec<ExtraSelectionInput>,
}

async fn save_extras(
    State(state): State<AppState>,
    Path(hold_id): Path<Uuid>,
    headers: HeaderMap,
    payload: Result<Json<SaveExtrasRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let token_hash = token_hash_from_cookie(&headers, hold_id)?;
    let request = payload.map_err(|_| ApiError::extras_bad_request())?.0;
    let context = state
        .extras
        .save_extras(hold_id, token_hash, request.selections)
        .await?;
    Ok((
        [(header::CACHE_CONTROL, "no-store, private")],
        Json(context),
    ))
}

async fn get_review(
    State(state): State<AppState>,
    Path(hold_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let result = match token_hash_from_cookie(&headers, hold_id) {
        Ok(token_hash) => state
            .reviews
            .get_review(hold_id, token_hash)
            .await
            .map_err(ApiError::from),
        Err(error) => Err(error),
    };
    let mut response = match result {
        Ok(context) => Json(context).into_response(),
        Err(error) => error.into_response(),
    };
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, private"),
    );
    response
}

async fn get_payment(
    State(state): State<AppState>,
    Path(hold_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let result = match (
        state.payments.as_ref(),
        token_hash_from_cookie(&headers, hold_id),
    ) {
        (Some(payments), Ok(token_hash)) => payments
            .get_payment(hold_id, token_hash)
            .await
            .map_err(ApiError::from),
        (None, _) => Err(ApiError::internal()),
        (_, Err(error)) => Err(error),
    };
    private_no_store(match result {
        Ok(context) => Json(context).into_response(),
        Err(error) => error.into_response(),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreatePaymentAttemptRequest {
    request_id: Uuid,
    method: PaymentMethod,
}

async fn create_payment_attempt(
    State(state): State<AppState>,
    Path(hold_id): Path<Uuid>,
    headers: HeaderMap,
    payload: Result<Json<CreatePaymentAttemptRequest>, JsonRejection>,
) -> Response {
    let result = async {
        let payments = state.payments.as_ref().ok_or_else(ApiError::internal)?;
        let token_hash = token_hash_from_cookie(&headers, hold_id)?;
        let request = payload.map_err(|_| ApiError::payment_bad_request())?.0;
        payments
            .create_attempt(
                hold_id,
                token_hash,
                CreatePaymentRequest {
                    request_id: request.request_id,
                    method: request.method,
                },
            )
            .await
            .map_err(ApiError::from)
    }
    .await;
    private_no_store(match result {
        Ok(attempt) => (StatusCode::CREATED, Json(attempt)).into_response(),
        Err(error) => error.into_response(),
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SimulatePaymentAttemptRequest {
    outcome: PaymentSimulationOutcome,
}

async fn simulate_payment_attempt(
    State(state): State<AppState>,
    Path((hold_id, attempt_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    payload: Result<Json<SimulatePaymentAttemptRequest>, JsonRejection>,
) -> Response {
    let result = async {
        let payments = state.payments.as_ref().ok_or_else(ApiError::internal)?;
        let token_hash = token_hash_from_cookie(&headers, hold_id)?;
        let request = payload.map_err(|_| ApiError::payment_bad_request())?.0;
        payments
            .simulate(hold_id, token_hash, attempt_id, request.outcome)
            .await
            .map_err(ApiError::from)
    }
    .await;
    private_no_store(match result {
        Ok(attempt) => Json(attempt).into_response(),
        Err(error) => error.into_response(),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TicketResponse {
    ticket: crate::domain::ticket::Ticket,
    qr_token: String,
}

async fn get_ticket(
    State(state): State<AppState>,
    Path((hold_id, attempt_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Response {
    let result = async {
        let tickets = state.tickets.as_ref().ok_or_else(ApiError::internal)?;
        let token_hash = token_hash_from_cookie(&headers, hold_id)?;
        let (ticket, qr_token) = tickets
            .get_ticket(hold_id, token_hash, attempt_id)
            .await
            .map_err(ApiError::from)?;
        Ok::<_, ApiError>(TicketResponse { ticket, qr_token })
    }
    .await;
    private_no_store(match result {
        Ok(ticket) => Json(ticket).into_response(),
        Err(error) => error.into_response(),
    })
}

async fn verify_ticket(State(state): State<AppState>, Path(token): Path<String>) -> Response {
    let result = async {
        let tickets = state.tickets.as_ref().ok_or_else(ApiError::internal)?;
        tickets.verify_ticket(&token).await.map_err(ApiError::from)
    }
    .await;
    private_no_store(match result {
        Ok(verification) => Json(verification).into_response(),
        Err(error) => error.into_response(),
    })
}

async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let result: Result<serde_json::Value, ApiError> = async {
        let secret = state
            .stripe_webhook_secret
            .as_deref()
            .ok_or_else(|| {
                ApiError::stripe_webhook_bad_request(
                    "STRIPE_WEBHOOK_UNCONFIGURED",
                    "Stripe webhook is not configured.",
                )
            })?;

        let signature_header = headers
            .get("stripe-signature")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                ApiError::stripe_webhook_bad_request(
                    "STRIPE_SIGNATURE_MISSING",
                    "Missing Stripe-Signature header.",
                )
            })?;

        let now = chrono::Utc::now().timestamp();
        crate::infrastructure::payment::stripe::verify_stripe_signature(
            signature_header,
            &body,
            secret,
            now,
            crate::infrastructure::payment::stripe::STRIPE_SIGNATURE_TOLERANCE_SECONDS,
        )
        .map_err(|err| match err {
            crate::infrastructure::payment::stripe::WebhookVerificationError::MissingHeader => {
                ApiError::stripe_webhook_bad_request(
                    "STRIPE_SIGNATURE_MISSING",
                    "Missing Stripe-Signature header.",
                )
            }
            crate::infrastructure::payment::stripe::WebhookVerificationError::MalformedHeader => {
                ApiError::stripe_webhook_bad_request(
                    "STRIPE_SIGNATURE_MALFORMED",
                    "Malformed Stripe-Signature header.",
                )
            }
            crate::infrastructure::payment::stripe::WebhookVerificationError::TimestampOutOfTolerance => {
                ApiError::stripe_webhook_bad_request(
                    "STRIPE_TIMESTAMP_OUT_OF_TOLERANCE",
                    "Stripe webhook timestamp is outside tolerance.",
                )
            }
            crate::infrastructure::payment::stripe::WebhookVerificationError::InvalidSignature => {
                ApiError::stripe_webhook_bad_request(
                    "STRIPE_SIGNATURE_INVALID",
                    "Invalid Stripe signature.",
                )
            }
            crate::infrastructure::payment::stripe::WebhookVerificationError::InvalidSecret => {
                ApiError::internal()
            }
        })?;

        let envelope = crate::infrastructure::payment::stripe::parse_stripe_webhook(&body)
            .map_err(|_| {
                ApiError::stripe_webhook_bad_request(
                    "STRIPE_PAYLOAD_INVALID",
                    "Invalid Stripe webhook payload.",
                )
            })?;

        if envelope.event_type != "payment_intent.succeeded"
            && envelope.event_type != "payment_intent.payment_failed"
        {
            return Ok(json!({ "received": true }));
        }

        let intent = crate::infrastructure::payment::stripe::parse_payment_intent_object(
            &envelope.data.object,
        )
        .map_err(|_| {
            ApiError::stripe_webhook_bad_request(
                "STRIPE_PAYLOAD_INVALID",
                "Invalid PaymentIntent object in webhook.",
            )
        })?;

        let payments = state.payments.as_ref().ok_or_else(ApiError::internal)?;

        let (failure_code, failure_message) =
            crate::infrastructure::payment::stripe::map_stripe_failure(
                intent.last_payment_error.as_ref(),
            );

        let command = crate::domain::payment::ProcessStripeWebhookCommand {
            event_id: envelope.id,
            event_type: envelope.event_type,
            payment_intent_id: intent.id,
            amount: intent.amount,
            currency: intent.currency,
            status: intent.status,
            failure_code: Some(failure_code),
            failure_message: Some(failure_message),
        };

        match payments.process_stripe_webhook(command).await {
            Ok(_) => {}
            // A signed event for an internal PaymentIntent can race the
            // transaction that persists provider_reference. Returning 5xx
            // asks Stripe to retry; we neither create a booking nor mark the
            // event processed, so the later retry remains safe.
            Err(PaymentRepositoryError::AttemptNotFound) => return Err(ApiError::internal()),
            Err(error) => return Err(ApiError::from(error)),
        }

        Ok(json!({ "received": true }))
    }
    .await;

    private_no_store(match result {
        Ok(payload) => (StatusCode::OK, Json(payload)).into_response(),
        Err(error) => error.into_response(),
    })
}

fn private_no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, private"),
    );
    response
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    field_errors: Vec<PassengerFieldError>,
}

struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
    conflicting_seats: Vec<String>,
    field_errors: Vec<PassengerFieldError>,
}

impl ApiError {
    fn validation(_error: DomainError) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "VALIDATION_ERROR",
            message: "The seat hold request is invalid.",
            conflicting_seats: Vec::new(),
            field_errors: Vec::new(),
        }
    }

    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "HOLD_UNAUTHORIZED",
            message: "Seat hold authorization is missing or invalid.",
            conflicting_seats: Vec::new(),
            field_errors: Vec::new(),
        }
    }

    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL_ERROR",
            message: "The seat hold service is temporarily unavailable.",
            conflicting_seats: Vec::new(),
            field_errors: Vec::new(),
        }
    }

    fn passenger_bad_request() -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "PASSENGER_VALIDATION_FAILED",
            message: "Passenger information is invalid.",
            conflicting_seats: Vec::new(),
            field_errors: Vec::new(),
        }
    }

    fn extras_bad_request() -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "EXTRAS_VALIDATION_FAILED",
            message: "Travel extras are invalid.",
            conflicting_seats: Vec::new(),
            field_errors: Vec::new(),
        }
    }

    fn payment_bad_request() -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "PAYMENT_REQUEST_INVALID",
            message: "The payment request is invalid.",
            conflicting_seats: Vec::new(),
            field_errors: Vec::new(),
        }
    }

    fn payment_finalization_conflict() -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "PAYMENT_IN_PROGRESS",
            message: "Payment finalization is in progress for this seat hold.",
            conflicting_seats: Vec::new(),
            field_errors: Vec::new(),
        }
    }

    fn stripe_webhook_bad_request(code: &'static str, message: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message,
            conflicting_seats: Vec::new(),
            field_errors: Vec::new(),
        }
    }
}

impl From<PaymentRepositoryError> for ApiError {
    fn from(error: PaymentRepositoryError) -> Self {
        let conflict = |code, message| Self {
            status: StatusCode::CONFLICT,
            code,
            message,
            conflicting_seats: Vec::new(),
            field_errors: Vec::new(),
        };
        match error {
            PaymentRepositoryError::HoldNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "HOLD_NOT_FOUND",
                message: "The seat hold could not be found.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            PaymentRepositoryError::Unauthorized => Self::unauthorized(),
            PaymentRepositoryError::HoldExpired => Self {
                status: StatusCode::GONE,
                code: "HOLD_EXPIRED",
                message: "The seat hold expired before payment completed.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            PaymentRepositoryError::HoldReleased => Self {
                status: StatusCode::GONE,
                code: "HOLD_RELEASED",
                message: "The seat hold has been released.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            PaymentRepositoryError::HoldConsumed => {
                conflict("HOLD_CONSUMED", "The seat hold has already been finalized.")
            }
            PaymentRepositoryError::PaymentFinalizationInProgress => {
                Self::payment_finalization_conflict()
            }
            PaymentRepositoryError::AlreadySucceeded => conflict(
                "PAYMENT_ALREADY_SUCCEEDED",
                "A payment has already succeeded for this seat hold.",
            ),
            PaymentRepositoryError::SeatsNotReady => {
                conflict("SEATS_NOT_READY", "Held seats are not ready for payment.")
            }
            PaymentRepositoryError::PassengersNotReady => conflict(
                "PASSENGERS_NOT_READY",
                "Passenger information must be completed before payment.",
            ),
            PaymentRepositoryError::ExtrasNotReady => conflict(
                "EXTRAS_NOT_READY",
                "Travel extras must be saved before payment.",
            ),
            PaymentRepositoryError::ReviewNotReady => conflict(
                "REVIEW_NOT_READY",
                "Return to Review to confirm the current authoritative price.",
            ),
            PaymentRepositoryError::AttemptInProgress => conflict(
                "PAYMENT_IN_PROGRESS",
                "Another payment attempt is still in progress.",
            ),
            PaymentRepositoryError::AttemptNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "PAYMENT_ATTEMPT_NOT_FOUND",
                message: "The payment attempt could not be found.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            PaymentRepositoryError::InvalidTransition => conflict(
                "PAYMENT_TRANSITION_INVALID",
                "The payment attempt cannot make that status transition.",
            ),
            PaymentRepositoryError::IdempotencyKeyReused => conflict(
                "IDEMPOTENCY_KEY_REUSED",
                "The payment request ID was already used for different input.",
            ),
            PaymentRepositoryError::InvalidRequest => Self::payment_bad_request(),
            PaymentRepositoryError::AmountMismatch => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "AMOUNT_MISMATCH",
                message: "Payment amount or currency mismatch.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            PaymentRepositoryError::Infrastructure(_error) => {
                tracing::error!("payment repository failure");
                Self::internal()
            }
        }
    }
}

impl From<ExtraRepositoryError> for ApiError {
    fn from(error: ExtraRepositoryError) -> Self {
        let validation = |code, message| Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code,
            message,
            conflicting_seats: Vec::new(),
            field_errors: Vec::new(),
        };
        match error {
            ExtraRepositoryError::HoldNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "HOLD_NOT_FOUND",
                message: "The seat hold could not be found.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            ExtraRepositoryError::Unauthorized => Self::unauthorized(),
            ExtraRepositoryError::HoldExpired => Self {
                status: StatusCode::GONE,
                code: "HOLD_EXPIRED",
                message: "The seat hold has expired.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            ExtraRepositoryError::HoldReleased => Self {
                status: StatusCode::GONE,
                code: "HOLD_RELEASED",
                message: "The seat hold has been released.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            ExtraRepositoryError::HoldConsumed => Self {
                status: StatusCode::CONFLICT,
                code: "HOLD_CONSUMED",
                message: "The seat hold has already been consumed.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            ExtraRepositoryError::PaymentFinalizationInProgress => {
                Self::payment_finalization_conflict()
            }
            ExtraRepositoryError::SeatCountMismatch => {
                validation("SEAT_COUNT_MISMATCH", "Held seats are incomplete.")
            }
            ExtraRepositoryError::PassengersNotReady => Self {
                status: StatusCode::CONFLICT,
                code: "PASSENGERS_NOT_READY",
                message: "Passenger information must be completed before extras.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            ExtraRepositoryError::UnknownProduct => {
                validation("EXTRA_PRODUCT_UNKNOWN", "The extra product is unknown.")
            }
            ExtraRepositoryError::InvalidQuantity => validation(
                "EXTRA_QUANTITY_INVALID",
                "The extra product quantity is invalid.",
            ),
            ExtraRepositoryError::InvalidPassenger => validation(
                "EXTRA_PASSENGER_INVALID",
                "The passenger does not belong to this hold.",
            ),
            ExtraRepositoryError::PassengerIneligible => validation(
                "EXTRA_PASSENGER_INELIGIBLE",
                "The passenger is not eligible for this extra.",
            ),
            ExtraRepositoryError::CategoryConflict => validation(
                "EXTRA_SELECTION_CONFLICT",
                "The passenger has conflicting extra selections.",
            ),
            ExtraRepositoryError::Infrastructure(_error) => {
                tracing::error!("extras repository failure");
                Self::internal()
            }
        }
    }
}

impl From<ReviewRepositoryError> for ApiError {
    fn from(error: ReviewRepositoryError) -> Self {
        let state_error = |code, message| Self {
            status: StatusCode::CONFLICT,
            code,
            message,
            conflicting_seats: Vec::new(),
            field_errors: Vec::new(),
        };
        match error {
            ReviewRepositoryError::HoldNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "HOLD_NOT_FOUND",
                message: "The seat hold could not be found.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            ReviewRepositoryError::Unauthorized => Self::unauthorized(),
            ReviewRepositoryError::HoldExpired => Self {
                status: StatusCode::GONE,
                code: "HOLD_EXPIRED",
                message: "The seat hold has expired.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            ReviewRepositoryError::HoldReleased => Self {
                status: StatusCode::GONE,
                code: "HOLD_RELEASED",
                message: "The seat hold has been released.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            ReviewRepositoryError::HoldConsumed => {
                state_error("HOLD_CONSUMED", "The seat hold has already been consumed.")
            }
            ReviewRepositoryError::SeatsNotReady => {
                state_error("SEATS_NOT_READY", "Held seats are not ready for review.")
            }
            ReviewRepositoryError::PassengersNotReady => state_error(
                "PASSENGERS_NOT_READY",
                "Passenger information must be completed before review.",
            ),
            ReviewRepositoryError::ExtrasNotReady => state_error(
                "EXTRAS_NOT_READY",
                "Travel extras must be explicitly saved before review.",
            ),
            ReviewRepositoryError::PricingUnavailable => Self {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "REVIEW_PRICING_UNAVAILABLE",
                message: "Authoritative review pricing is temporarily unavailable.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            ReviewRepositoryError::Infrastructure(_error) => {
                tracing::error!("review repository failure");
                Self::internal()
            }
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
                field_errors: Vec::new(),
            },
            SeatHoldRepositoryError::CabinUnavailable => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "CABIN_UNAVAILABLE",
                message: "The selected cabin is not available for this flight.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            SeatHoldRepositoryError::SeatNotFound(seats) => Self {
                status: StatusCode::NOT_FOUND,
                code: "SEAT_NOT_FOUND",
                message: "One or more seats do not exist for this flight and cabin.",
                conflicting_seats: seats,
                field_errors: Vec::new(),
            },
            SeatHoldRepositoryError::SeatCountMismatch => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "SEAT_COUNT_MISMATCH",
                message: "The seat count must match adults plus children.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            SeatHoldRepositoryError::SeatConflict(seats) => Self {
                status: StatusCode::CONFLICT,
                code: "SEAT_UNAVAILABLE",
                message: "One or more seats were just taken.",
                conflicting_seats: seats,
                field_errors: Vec::new(),
            },
            SeatHoldRepositoryError::HoldNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "HOLD_NOT_FOUND",
                message: "The seat hold could not be found.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            SeatHoldRepositoryError::Unauthorized => Self::unauthorized(),
            SeatHoldRepositoryError::HoldExpired => Self {
                status: StatusCode::GONE,
                code: "HOLD_EXPIRED",
                message: "The seat hold has expired.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            SeatHoldRepositoryError::HoldReleased => Self {
                status: StatusCode::GONE,
                code: "HOLD_RELEASED",
                message: "The seat hold has been released.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            SeatHoldRepositoryError::HoldConsumed => Self {
                status: StatusCode::CONFLICT,
                code: "HOLD_CONSUMED",
                message: "The seat hold has already been consumed.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            SeatHoldRepositoryError::PaymentFinalizationInProgress => {
                Self::payment_finalization_conflict()
            }
            SeatHoldRepositoryError::Infrastructure(error) => {
                tracing::error!(?error, "seat hold repository failure");
                Self::internal()
            }
        }
    }
}

impl From<PassengerRepositoryError> for ApiError {
    fn from(error: PassengerRepositoryError) -> Self {
        match error {
            PassengerRepositoryError::HoldNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "HOLD_NOT_FOUND",
                message: "The seat hold could not be found.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            PassengerRepositoryError::Unauthorized => Self::unauthorized(),
            PassengerRepositoryError::HoldExpired => Self {
                status: StatusCode::GONE,
                code: "HOLD_EXPIRED",
                message: "The seat hold has expired.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            PassengerRepositoryError::HoldReleased => Self {
                status: StatusCode::GONE,
                code: "HOLD_RELEASED",
                message: "The seat hold has been released.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            PassengerRepositoryError::HoldConsumed => Self {
                status: StatusCode::CONFLICT,
                code: "HOLD_CONSUMED",
                message: "The seat hold has already been consumed.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            PassengerRepositoryError::PaymentFinalizationInProgress => {
                Self::payment_finalization_conflict()
            }
            PassengerRepositoryError::SeatCountMismatch => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "SEAT_COUNT_MISMATCH",
                message: "The seat count must match adults plus children.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            PassengerRepositoryError::CountMismatch => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "PASSENGER_COUNT_MISMATCH",
                message: "Passenger count does not match the active hold.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            PassengerRepositoryError::TypeMismatch => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "PASSENGER_TYPE_MISMATCH",
                message: "Passenger types do not match the active hold.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            PassengerRepositoryError::Validation(field_errors) => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "PASSENGER_VALIDATION_FAILED",
                message: "Passenger information is invalid.",
                conflicting_seats: Vec::new(),
                field_errors,
            },
            PassengerRepositoryError::Infrastructure(_error) => {
                tracing::error!("passenger repository failure");
                Self::internal()
            }
        }
    }
}

impl From<TicketRepositoryError> for ApiError {
    fn from(error: TicketRepositoryError) -> Self {
        match error {
            TicketRepositoryError::HoldNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "HOLD_NOT_FOUND",
                message: "The seat hold could not be found.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            TicketRepositoryError::Unauthorized => Self::unauthorized(),
            TicketRepositoryError::PaymentNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "PAYMENT_ATTEMPT_NOT_FOUND",
                message: "The payment attempt could not be found.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            TicketRepositoryError::PaymentIncomplete => Self {
                status: StatusCode::CONFLICT,
                code: "TICKET_PAYMENT_INCOMPLETE",
                message: "A ticket is available after payment succeeds.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            TicketRepositoryError::FinalizationInconsistent => Self {
                status: StatusCode::CONFLICT,
                code: "TICKET_FINALIZATION_INCONSISTENT",
                message: "The finalized booking is not ready for ticket issuance.",
                conflicting_seats: Vec::new(),
                field_errors: Vec::new(),
            },
            TicketRepositoryError::IdentityGeneration
            | TicketRepositoryError::Infrastructure(_) => {
                tracing::error!("ticket repository failure");
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
                    field_errors: self.field_errors,
                },
            }),
        )
            .into_response()
    }
}
