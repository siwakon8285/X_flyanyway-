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
        booking_management::BookingListFilter,
        cancellation::StaffCancellationActor,
        flight::{FlightCommand, FlightManagementError, FlightStatus},
        manage_booking::BookingStatus,
        repositories::{
            BookingManagementRepositoryError, CancellationRepositoryError,
            TicketOperationsRepositoryError,
        },
        staff::{PermissionCode, StaffPrincipal},
        ticket::TicketStatus,
        ticket_operations::{PrintableTicketOperationsDetail, TicketOperationsFilter},
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
        .route("/api/v1/admin/bookings", get(list_bookings))
        .route("/api/v1/admin/bookings/search", post(search_bookings))
        .route(
            "/api/v1/admin/bookings/{booking_reference}",
            get(booking_detail),
        )
        .route(
            "/api/v1/admin/bookings/{booking_reference}/cancel",
            post(cancel_booking),
        )
        .route("/api/v1/admin/tickets", get(list_tickets))
        .route("/api/v1/admin/tickets/search", post(search_tickets))
        .route("/api/v1/admin/tickets/{ticket_number}", get(ticket_detail))
        .route(
            "/api/v1/admin/tickets/{ticket_number}/print",
            get(print_ticket),
        )
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
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct TicketListQuery {
    ticket_number: Option<String>,
    booking_reference: Option<String>,
    passenger_name: Option<String>,
    flight_number: Option<String>,
    origin: Option<String>,
    destination: Option<String>,
    travel_date: Option<String>,
    ticket_status: Option<String>,
    cabin: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn list_tickets(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    query: Result<Query<TicketListQuery>, QueryRejection>,
) -> Response {
    let result = async {
        require_ticket_passenger_read(&staff)?;
        let query = query.map_err(|_| AdminApiError::ticket_filter_invalid())?.0;
        if query
            .passenger_name
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        {
            return Err(AdminApiError::ticket_filter_invalid());
        }
        let filter = parse_ticket_filter(query)?;
        let repository = state
            .ticket_operations
            .as_ref()
            .ok_or_else(AdminApiError::ticket_unavailable)?;
        Ok::<_, AdminApiError>(
            Json(
                repository
                    .list_tickets(&filter)
                    .await
                    .map_err(AdminApiError::from_ticket_operations)?,
            )
            .into_response(),
        )
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

async fn search_tickets(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    headers: HeaderMap,
    payload: Result<Json<TicketListQuery>, JsonRejection>,
) -> Response {
    let result = async {
        require_ticket_passenger_read(&staff)?;
        trusted_booking_mutation(&state, &headers)?;
        let filter = parse_ticket_filter(
            payload
                .map_err(|_| AdminApiError::ticket_filter_invalid())?
                .0,
        )?;
        let repository = state
            .ticket_operations
            .as_ref()
            .ok_or_else(AdminApiError::ticket_unavailable)?;
        Ok::<_, AdminApiError>(
            Json(
                repository
                    .list_tickets(&filter)
                    .await
                    .map_err(AdminApiError::from_ticket_operations)?,
            )
            .into_response(),
        )
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

async fn ticket_detail(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    Path(ticket_number): Path<String>,
) -> Response {
    let result = async {
        require_ticket_passenger_read(&staff)?;
        let number = normalize_ticket_number(&ticket_number)?;
        let repository = state
            .ticket_operations
            .as_ref()
            .ok_or_else(AdminApiError::ticket_unavailable)?;
        let record = repository
            .get_ticket_operations(&number)
            .await
            .map_err(AdminApiError::from_ticket_operations)?
            .ok_or_else(AdminApiError::ticket_not_found)?;
        Ok::<_, AdminApiError>(Json(record.detail).into_response())
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

async fn print_ticket(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    Path(ticket_number): Path<String>,
) -> Response {
    let result = async {
        staff
            .require(PermissionCode::TicketsPrint)
            .map_err(|_| AdminApiError::permission_denied())?;
        let number = normalize_ticket_number(&ticket_number)?;
        let repository = state
            .ticket_operations
            .as_ref()
            .ok_or_else(AdminApiError::ticket_unavailable)?;
        let record = repository
            .get_ticket_operations(&number)
            .await
            .map_err(AdminApiError::from_ticket_operations)?
            .ok_or_else(AdminApiError::ticket_not_found)?;
        if record.detail.ticket_status == TicketStatus::Cancelled {
            return Err(AdminApiError::cancelled_ticket_not_printable());
        }
        let qr_token = state
            .tickets
            .as_ref()
            .ok_or_else(AdminApiError::ticket_unavailable)?
            .sign_ticket_id(record.ticket_id)
            .map_err(|_| AdminApiError::ticket_unavailable())?;
        Ok::<_, AdminApiError>(
            Json(PrintableTicketOperationsDetail {
                ticket: record.detail,
                qr_token,
                printable: true,
            })
            .into_response(),
        )
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

fn require_ticket_passenger_read(staff: &AuthenticatedStaff) -> Result<(), AdminApiError> {
    staff
        .require(PermissionCode::TicketsRead)
        .map_err(|_| AdminApiError::permission_denied())?;
    staff
        .require(PermissionCode::PassengersRead)
        .map_err(|_| AdminApiError::permission_denied())?;
    Ok(())
}

fn normalize_ticket_number(value: &str) -> Result<String, AdminApiError> {
    let value = value.trim().to_ascii_uppercase();
    let valid = value.len() == 15
        && value.starts_with("XFT")
        && value[3..]
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || matches!(byte, b'2'..=b'9'));
    valid
        .then_some(value)
        .ok_or_else(AdminApiError::ticket_filter_invalid)
}

fn parse_ticket_filter(query: TicketListQuery) -> Result<TicketOperationsFilter, AdminApiError> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    if !(1..=50).contains(&limit) || offset < 0 {
        return Err(AdminApiError::ticket_filter_invalid());
    }
    let name = query
        .passenger_name
        .map(|v| v.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|v| !v.is_empty())
        .map(|v| {
            if v.chars().count() > 100 || v.chars().any(char::is_control) {
                return Err(AdminApiError::ticket_filter_invalid());
            }
            Ok(format!(
                "%{}%",
                v.to_lowercase()
                    .replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_")
            ))
        })
        .transpose()?;
    let flight = query
        .flight_number
        .filter(|v| !v.trim().is_empty())
        .map(|v| {
            let compact = v
                .split_whitespace()
                .collect::<String>()
                .to_ascii_uppercase();
            let digits = compact
                .strip_prefix("XF")
                .filter(|d| d.len() == 3 && d.bytes().all(|b| b.is_ascii_digit()))
                .ok_or_else(AdminApiError::ticket_filter_invalid)?;
            Ok::<String, AdminApiError>(format!("XF {digits}"))
        })
        .transpose()?;
    let airport = |v: Option<String>| {
        v.filter(|v| !v.trim().is_empty())
            .map(|v| {
                let n = v.trim().to_ascii_uppercase();
                (n.len() == 3 && n.bytes().all(|b| b.is_ascii_uppercase()))
                    .then_some(n)
                    .ok_or_else(AdminApiError::ticket_filter_invalid)
            })
            .transpose()
    };
    let cabin = query
        .cabin
        .filter(|v| !v.trim().is_empty())
        .map(
            |v| match v.trim().to_ascii_uppercase().replace('_', "-").as_str() {
                "BUSINESS" => Ok("business".into()),
                "FIRST" => Ok("first".into()),
                "ECONOMY" => Ok("economy".into()),
                "PREMIUM-ECONOMY" => Ok("premium-economy".into()),
                _ => Err(AdminApiError::ticket_filter_invalid()),
            },
        )
        .transpose()?;
    Ok(TicketOperationsFilter {
        ticket_number: query
            .ticket_number
            .filter(|v| !v.trim().is_empty())
            .map(|v| normalize_ticket_number(&v))
            .transpose()?,
        booking_reference: query
            .booking_reference
            .filter(|v| !v.trim().is_empty())
            .map(|v| normalize_booking_reference(&v))
            .transpose()?,
        passenger_name: name,
        flight_number: flight,
        origin: airport(query.origin)?,
        destination: airport(query.destination)?,
        travel_date: query
            .travel_date
            .filter(|v| !v.trim().is_empty())
            .map(|v| {
                NaiveDate::parse_from_str(&v, "%Y-%m-%d")
                    .map_err(|_| AdminApiError::ticket_filter_invalid())
            })
            .transpose()?,
        ticket_status: query
            .ticket_status
            .filter(|v| !v.trim().is_empty())
            .map(|v| match v.as_str() {
                "ISSUED" => Ok(TicketStatus::Issued),
                "CANCELLED" => Ok(TicketStatus::Cancelled),
                _ => Err(AdminApiError::ticket_filter_invalid()),
            })
            .transpose()?,
        cabin,
        limit,
        offset,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct BookingListQuery {
    booking_reference: Option<String>,
    passenger_name: Option<String>,
    flight_number: Option<String>,
    travel_date: Option<String>,
    booking_status: Option<String>,
    cabin: Option<String>,
    origin: Option<String>,
    destination: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn list_bookings(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    query: Result<Query<BookingListQuery>, QueryRejection>,
) -> Response {
    let result = async {
        staff
            .require(PermissionCode::BookingsRead)
            .map_err(|_| AdminApiError::permission_denied())?;
        let query = query
            .map_err(|_| AdminApiError::booking_filter_invalid())?
            .0;
        let filter = parse_booking_filter(query)?;
        let repository = state
            .booking_management
            .as_ref()
            .ok_or_else(AdminApiError::booking_unavailable)?;
        Ok::<_, AdminApiError>(
            Json(
                repository
                    .list_bookings(&filter)
                    .await
                    .map_err(AdminApiError::from_booking_repository)?,
            )
            .into_response(),
        )
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

async fn search_bookings(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    headers: HeaderMap,
    payload: Result<Json<BookingListQuery>, JsonRejection>,
) -> Response {
    let result = async {
        staff
            .require(PermissionCode::BookingsRead)
            .map_err(|_| AdminApiError::permission_denied())?;
        trusted_booking_mutation(&state, &headers)?;
        let filter = parse_booking_filter(
            payload
                .map_err(|_| AdminApiError::booking_filter_invalid())?
                .0,
        )?;
        let repository = state
            .booking_management
            .as_ref()
            .ok_or_else(AdminApiError::booking_unavailable)?;
        Ok::<_, AdminApiError>(
            Json(
                repository
                    .list_bookings(&filter)
                    .await
                    .map_err(AdminApiError::from_booking_repository)?,
            )
            .into_response(),
        )
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

async fn booking_detail(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    Path(booking_reference): Path<String>,
) -> Response {
    let result = async {
        staff
            .require(PermissionCode::BookingsRead)
            .map_err(|_| AdminApiError::permission_denied())?;
        let booking_reference = normalize_booking_reference(&booking_reference)?;
        let repository = state
            .booking_management
            .as_ref()
            .ok_or_else(AdminApiError::booking_unavailable)?;
        let detail = repository
            .get_booking_detail(&booking_reference, Utc::now())
            .await
            .map_err(AdminApiError::from_booking_repository)?
            .ok_or_else(AdminApiError::booking_not_found)?;
        Ok::<_, AdminApiError>(Json(detail).into_response())
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

async fn cancel_booking(
    State(state): State<AppState>,
    staff: AuthenticatedStaff,
    Path(booking_reference): Path<String>,
    headers: HeaderMap,
) -> Response {
    let result = async {
        let actor = staff
            .require(PermissionCode::BookingsManage)
            .map_err(|_| AdminApiError::permission_denied())?;
        trusted_booking_mutation(&state, &headers)?;
        let booking_reference = normalize_booking_reference(&booking_reference)?;
        let repository = state
            .booking_management
            .as_ref()
            .ok_or_else(AdminApiError::booking_unavailable)?;
        let ticket_id = repository
            .ticket_id_for_booking_reference(&booking_reference)
            .await
            .map_err(AdminApiError::from_booking_repository)?
            .ok_or_else(AdminApiError::booking_not_found)?;
        let cancellations = state
            .cancellations
            .as_ref()
            .ok_or_else(AdminApiError::booking_unavailable)?;
        cancellations
            .cancel_for_staff(
                ticket_id,
                &StaffCancellationActor {
                    staff_user_id: actor.staff_user_id(),
                    email: actor.email().to_owned(),
                },
            )
            .await
            .map_err(AdminApiError::from_cancellation)?;
        let detail = repository
            .get_booking_detail(&booking_reference, Utc::now())
            .await
            .map_err(AdminApiError::from_booking_repository)?
            .ok_or_else(AdminApiError::booking_not_found)?;
        Ok::<_, AdminApiError>(Json(detail).into_response())
    }
    .await;
    private_no_store(result.unwrap_or_else(IntoResponse::into_response))
}

fn parse_booking_filter(query: BookingListQuery) -> Result<BookingListFilter, AdminApiError> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    if !(1..=50).contains(&limit) || offset < 0 {
        return Err(AdminApiError::booking_filter_invalid());
    }
    let optional_reference = query
        .booking_reference
        .filter(|value| !value.trim().is_empty())
        .map(|value| normalize_booking_reference(&value))
        .transpose()?;
    let passenger_name = query
        .passenger_name
        .map(|value| value.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value.chars().count() > 100 || value.chars().any(char::is_control) {
                return Err(AdminApiError::booking_filter_invalid());
            }
            let escaped = value
                .to_lowercase()
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            Ok(format!("%{escaped}%"))
        })
        .transpose()?;
    let flight_number = query
        .flight_number
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            let mut compact = value.split_whitespace().collect::<String>().to_uppercase();
            let digits = compact
                .strip_prefix("XF")
                .filter(|digits| {
                    digits.len() == 3 && digits.bytes().all(|byte| byte.is_ascii_digit())
                })
                .ok_or_else(AdminApiError::booking_filter_invalid)?
                .to_owned();
            compact = format!("XF {digits}");
            Ok::<String, AdminApiError>(compact)
        })
        .transpose()?;
    let airport = |value: Option<String>| -> Result<Option<String>, AdminApiError> {
        value
            .filter(|value| !value.trim().is_empty())
            .map(|value| {
                let value = value.trim().to_uppercase();
                (value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_uppercase()))
                    .then_some(value)
                    .ok_or_else(AdminApiError::booking_filter_invalid)
            })
            .transpose()
    };
    let travel_date = query
        .travel_date
        .filter(|value| !value.trim().is_empty())
        .map(|value| NaiveDate::parse_from_str(&value, "%Y-%m-%d"))
        .transpose()
        .map_err(|_| AdminApiError::booking_filter_invalid())?;
    let booking_status = query
        .booking_status
        .filter(|value| !value.trim().is_empty())
        .map(|value| match value.as_str() {
            "CONFIRMED" => Ok(BookingStatus::Confirmed),
            "CANCELLED" => Ok(BookingStatus::Cancelled),
            _ => Err(AdminApiError::booking_filter_invalid()),
        })
        .transpose()?;
    let cabin = query
        .cabin
        .filter(|value| !value.trim().is_empty())
        .map(
            |value| match value.trim().to_ascii_uppercase().replace('_', "-").as_str() {
                "BUSINESS" => Ok("business".to_owned()),
                "FIRST" => Ok("first".to_owned()),
                "ECONOMY" => Ok("economy".to_owned()),
                "PREMIUM-ECONOMY" => Ok("premium-economy".to_owned()),
                _ => Err(AdminApiError::booking_filter_invalid()),
            },
        )
        .transpose()?;
    Ok(BookingListFilter {
        booking_reference: optional_reference,
        passenger_name,
        flight_number,
        travel_date,
        booking_status,
        cabin,
        origin: airport(query.origin)?,
        destination: airport(query.destination)?,
        limit,
        offset,
    })
}

fn normalize_booking_reference(value: &str) -> Result<String, AdminApiError> {
    let normalized = value.trim().to_ascii_uppercase();
    let valid = normalized.len() == 10
        && normalized.starts_with("XF")
        && normalized[2..]
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || matches!(byte, b'2'..=b'9'));
    valid
        .then_some(normalized)
        .ok_or_else(AdminApiError::booking_filter_invalid)
}

fn trusted_booking_mutation(state: &AppState, headers: &HeaderMap) -> Result<(), AdminApiError> {
    browser_mutation_is_trusted(headers, &state.frontend_origin)
        .then_some(())
        .ok_or_else(AdminApiError::forbidden_origin)
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
    fn booking_filter_invalid() -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "BOOKING_FILTER_INVALID",
            message: "Review the booking filters and try again.",
        }
    }
    fn booking_not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "BOOKING_NOT_FOUND",
            message: "The booking was not found.",
        }
    }
    fn booking_unavailable() -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "BOOKING_MANAGEMENT_UNAVAILABLE",
            message: "Booking management is temporarily unavailable.",
        }
    }
    fn ticket_filter_invalid() -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "TICKET_FILTER_INVALID",
            message: "Review the ticket filters and try again.",
        }
    }
    fn ticket_not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "TICKET_NOT_FOUND",
            message: "The exact ticket was not found.",
        }
    }
    fn ticket_unavailable() -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "TICKET_OPERATIONS_UNAVAILABLE",
            message: "Ticket operations are temporarily unavailable.",
        }
    }
    fn cancelled_ticket_not_printable() -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "CANCELLED_TICKET_NOT_PRINTABLE",
            message: "A cancelled ticket cannot be printed as a travel document.",
        }
    }
    fn from_ticket_operations(error: TicketOperationsRepositoryError) -> Self {
        match error {
            TicketOperationsRepositoryError::InconsistentState => Self {
                status: StatusCode::CONFLICT,
                code: "TICKET_STATE_CONFLICT",
                message: "The authoritative ticket state is inconsistent.",
            },
            TicketOperationsRepositoryError::Infrastructure(_) => Self::ticket_unavailable(),
        }
    }
    fn from_booking_repository(error: BookingManagementRepositoryError) -> Self {
        match error {
            BookingManagementRepositoryError::InconsistentState => Self {
                status: StatusCode::CONFLICT,
                code: "BOOKING_STATE_CONFLICT",
                message: "The authoritative booking state is inconsistent.",
            },
            BookingManagementRepositoryError::Infrastructure(_) => Self::booking_unavailable(),
        }
    }
    fn from_cancellation(error: CancellationRepositoryError) -> Self {
        match error {
            CancellationRepositoryError::NotFound => Self::booking_not_found(),
            CancellationRepositoryError::Ineligible => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "BOOKING_CANCELLATION_INELIGIBLE",
                message: "This booking is not eligible for cancellation.",
            },
            CancellationRepositoryError::InconsistentState => Self {
                status: StatusCode::CONFLICT,
                code: "BOOKING_STATE_CONFLICT",
                message: "The authoritative booking state is inconsistent.",
            },
            CancellationRepositoryError::Infrastructure(_) => Self::booking_unavailable(),
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
