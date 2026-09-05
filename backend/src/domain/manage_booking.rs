use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use thiserror::Error;

use crate::domain::{
    extras::{ExtraCategory, Money},
    payment::PaymentStatus,
    ticket::TicketStatus,
};

const BOOKING_REFERENCE_LENGTH: usize = 10;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManageBookingLookup {
    booking_reference: String,
    family_name: String,
}

impl ManageBookingLookup {
    pub fn new(
        booking_reference: String,
        family_name: String,
    ) -> Result<Self, ManageBookingLookupError> {
        let booking_reference = booking_reference.trim().to_ascii_uppercase();
        let reference_valid = booking_reference.len() == BOOKING_REFERENCE_LENGTH
            && booking_reference.starts_with("XF")
            && booking_reference[2..].bytes().all(|value| {
                matches!(value, b'A'..=b'Z' | b'2'..=b'9') && value != b'I' && value != b'O'
            });
        if !reference_valid {
            return Err(ManageBookingLookupError::InvalidBookingReference);
        }

        let family_name = normalize_name(&family_name).to_lowercase();
        if family_name.is_empty()
            || family_name.chars().count() > 100
            || !family_name.chars().all(|character| {
                character.is_alphabetic()
                    || character.is_whitespace()
                    || matches!(character, '-' | '\'' | '’' | '.')
            })
        {
            return Err(ManageBookingLookupError::InvalidFamilyName);
        }

        Ok(Self {
            booking_reference,
            family_name,
        })
    }

    pub fn booking_reference(&self) -> &str {
        &self.booking_reference
    }

    pub fn family_name(&self) -> &str {
        &self.family_name
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum ManageBookingLookupError {
    #[error("booking reference is invalid")]
    InvalidBookingReference,
    #[error("family name is invalid")]
    InvalidFamilyName,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BookingStatus {
    Confirmed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TravelDocumentStatus {
    Complete,
    Incomplete,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageBookingJourney {
    pub flight_number: String,
    pub origin_code: String,
    pub destination_code: String,
    pub departure_date: chrono::NaiveDate,
    pub departure_time: Option<String>,
    pub departure_time_zone: Option<String>,
    pub departure_at: Option<DateTime<Utc>>,
    pub arrival_date: Option<chrono::NaiveDate>,
    pub arrival_time: Option<String>,
    pub cabin: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageBookingPassenger {
    pub ordinal: u8,
    pub display_name: String,
    pub travel_document_status: TravelDocumentStatus,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageBookingExtra {
    pub passenger_ordinal: u8,
    pub product_code: String,
    pub category: ExtraCategory,
    pub quantity: u8,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageBookingPayment {
    pub status: PaymentStatus,
    pub amount: Money,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageBookingTicket {
    pub ticket_number: String,
    pub status: TicketStatus,
    pub issued_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageBookingCancellation {
    pub eligibility: CancellationEligibility,
    pub cutoff_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManageBooking {
    pub booking_reference: String,
    pub status: BookingStatus,
    pub journey: ManageBookingJourney,
    pub passengers: Vec<ManageBookingPassenger>,
    pub seats: Vec<String>,
    pub extras: Vec<ManageBookingExtra>,
    pub payment: ManageBookingPayment,
    pub ticket: ManageBookingTicket,
    pub cancellation: ManageBookingCancellation,
}

#[derive(Clone, Debug)]
pub struct ManageBookingRecord {
    pub ticket_id: uuid::Uuid,
    pub booking: ManageBooking,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CancellationEligibility {
    Eligible,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TravelEligibility {
    pub cancellation: CancellationEligibility,
    pub cancellation_cutoff_at: Option<DateTime<Utc>>,
}

pub fn derive_travel_eligibility(
    departure_at: Option<DateTime<Utc>>,
    ticket_status: TicketStatus,
    now: DateTime<Utc>,
) -> TravelEligibility {
    let Some(departure_at) = departure_at else {
        return unavailable_eligibility(None);
    };
    let cutoff = departure_at - Duration::hours(24);
    if ticket_status == TicketStatus::Cancelled || departure_at <= now {
        return unavailable_eligibility(Some(cutoff));
    }
    TravelEligibility {
        cancellation: if now <= cutoff {
            CancellationEligibility::Eligible
        } else {
            CancellationEligibility::Unavailable
        },
        cancellation_cutoff_at: Some(cutoff),
    }
}

fn unavailable_eligibility(cutoff: Option<DateTime<Utc>>) -> TravelEligibility {
    TravelEligibility {
        cancellation: CancellationEligibility::Unavailable,
        cancellation_cutoff_at: cutoff,
    }
}

fn normalize_name(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
