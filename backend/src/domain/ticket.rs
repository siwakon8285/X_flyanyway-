use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TicketStatus {
    Issued,
    Cancelled,
}

impl TicketStatus {
    pub fn parse_database(value: &str) -> Option<Self> {
        match value {
            "ISSUED" => Some(Self::Issued),
            "CANCELLED" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketJourney {
    pub flight_number: String,
    pub origin_code: String,
    pub destination_code: String,
    pub departure_date: NaiveDate,
    pub departure_time: Option<String>,
    pub arrival_time: Option<String>,
    pub arrival_day_offset: Option<u8>,
    pub cabin: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketPassenger {
    pub display_name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    pub id: Uuid,
    pub booking_reference: String,
    pub ticket_number: String,
    pub status: TicketStatus,
    pub issued_at: DateTime<Utc>,
    pub payment_status: String,
    pub amount: i64,
    pub currency_code: String,
    pub journey: TicketJourney,
    pub passengers: Vec<TicketPassenger>,
    pub seats: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketVerification {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_status: Option<TicketStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flight_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub departure_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub departure_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seats: Option<Vec<String>>,
}
