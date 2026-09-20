use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::{flight::FlightStatus, ticket::TicketStatus};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckInStatus {
    Ready,
    CheckedIn,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckInUnavailableReason {
    TooEarly,
    AlreadyDeparted,
    TicketCancelled,
    FlightCancelled,
    NoSeatAssignment,
    BookingInvalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckInState {
    Ready,
    CheckedIn,
    Unavailable(CheckInUnavailableReason),
}

impl CheckInState {
    pub fn status(&self) -> CheckInStatus {
        match self {
            Self::Ready => CheckInStatus::Ready,
            Self::CheckedIn => CheckInStatus::CheckedIn,
            Self::Unavailable(_) => CheckInStatus::Unavailable,
        }
    }

    pub fn reason(&self) -> Option<CheckInUnavailableReason> {
        match self {
            Self::Unavailable(reason) => Some(*reason),
            Self::Ready | Self::CheckedIn => None,
        }
    }
}

pub fn derive_check_in_state(
    now: DateTime<Utc>,
    departure_at: Option<DateTime<Utc>>,
    ticket_status: TicketStatus,
    flight_status: FlightStatus,
    booking_valid: bool,
    has_seat_assignment: bool,
    already_issued: bool,
) -> CheckInState {
    if ticket_status != TicketStatus::Issued {
        return CheckInState::Unavailable(CheckInUnavailableReason::TicketCancelled);
    }
    if flight_status != FlightStatus::Scheduled {
        return CheckInState::Unavailable(CheckInUnavailableReason::FlightCancelled);
    }
    if !booking_valid {
        return CheckInState::Unavailable(CheckInUnavailableReason::BookingInvalid);
    }
    if !has_seat_assignment {
        return CheckInState::Unavailable(CheckInUnavailableReason::NoSeatAssignment);
    }
    if already_issued {
        return CheckInState::CheckedIn;
    }
    let Some(departure_at) = departure_at else {
        return CheckInState::Unavailable(CheckInUnavailableReason::BookingInvalid);
    };
    let cutoff = departure_at - Duration::hours(24);
    if now <= cutoff {
        return CheckInState::Unavailable(CheckInUnavailableReason::TooEarly);
    }
    if now >= departure_at {
        return CheckInState::Unavailable(CheckInUnavailableReason::AlreadyDeparted);
    }
    CheckInState::Ready
}

pub fn derive_boarding_pass_invalid_reason(
    now: DateTime<Utc>,
    departure_at: Option<DateTime<Utc>>,
    ticket_status: TicketStatus,
    flight_status: FlightStatus,
    booking_valid: bool,
) -> Option<BoardingPassInvalidReason> {
    if ticket_status != TicketStatus::Issued {
        return Some(BoardingPassInvalidReason::TicketCancelled);
    }
    if flight_status != FlightStatus::Scheduled {
        return Some(BoardingPassInvalidReason::FlightCancelled);
    }
    if !booking_valid {
        return Some(BoardingPassInvalidReason::BookingInvalid);
    }
    if departure_at.is_none_or(|departure_at| departure_at <= now) {
        return Some(BoardingPassInvalidReason::Expired);
    }
    None
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckInStateResponse {
    pub status: CheckInStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<CheckInUnavailableReason>,
}

impl From<CheckInState> for CheckInStateResponse {
    fn from(state: CheckInState) -> Self {
        Self {
            status: state.status(),
            reason: state.reason(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardingPassSummary {
    pub id: Uuid,
    pub seat: String,
    pub cabin: String,
    pub checked_in_at: DateTime<Utc>,
    pub issued_at: DateTime<Utc>,
    pub valid_for_travel: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardingPassDocument {
    pub boarding_pass_id: Uuid,
    pub ticket_number: String,
    pub booking_reference: String,
    pub passenger_ordinal: u8,
    pub passenger_name: String,
    pub flight_number: String,
    pub origin_code: String,
    pub destination_code: String,
    pub departure_at: DateTime<Utc>,
    pub departure_time: String,
    pub origin_time_zone: String,
    pub seat: String,
    pub cabin: String,
    pub checked_in_at: DateTime<Utc>,
    pub issued_at: DateTime<Utc>,
    pub valid_for_travel: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_reason: Option<BoardingPassInvalidReason>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BoardingPassInvalidReason {
    Expired,
    TicketCancelled,
    FlightCancelled,
    BookingInvalid,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardingPassVerification {
    pub valid: bool,
    pub boarding_pass_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_reason: Option<BoardingPassInvalidReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flight_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub departure_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_time_zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cabin: Option<String>,
}
