use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::{
    cancellation::RefundStatus,
    manage_booking::BookingStatus,
    passengers::{Gender, PassengerType},
    payment::PaymentStatus,
    ticket::TicketStatus,
};

#[derive(Clone, Debug)]
pub struct TicketOperationsFilter {
    pub ticket_number: Option<String>,
    pub booking_reference: Option<String>,
    pub passenger_name: Option<String>,
    pub flight_number: Option<String>,
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub travel_date: Option<NaiveDate>,
    pub ticket_status: Option<TicketStatus>,
    pub cabin: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketOperationsPage {
    pub items: Vec<TicketOperationsListItem>,
    pub next_offset: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketOperationsListItem {
    pub ticket_number: String,
    pub booking_reference: String,
    pub passenger_names: Vec<String>,
    pub flight_number: String,
    pub origin_code: String,
    pub destination_code: String,
    pub travel_date: NaiveDate,
    pub cabin: String,
    pub seats: Vec<String>,
    pub ticket_status: TicketStatus,
}

#[derive(Clone, Debug)]
pub struct TicketOperationsRecord {
    pub ticket_id: Uuid,
    pub detail: TicketOperationsDetail,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketOperationsDetail {
    pub ticket_number: String,
    pub ticket_status: TicketStatus,
    pub issued_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub booking_reference: String,
    pub booking_status: BookingStatus,
    pub payment_status: PaymentStatus,
    pub refund_status: Option<RefundStatus>,
    pub journey: TicketOperationsJourney,
    pub passengers: Vec<TicketOperationsPassenger>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketOperationsJourney {
    pub flight_number: String,
    pub origin_code: String,
    pub destination_code: String,
    pub travel_date: NaiveDate,
    pub departure_at: Option<DateTime<Utc>>,
    pub departure_time: Option<String>,
    pub origin_time_zone: Option<String>,
    pub cabin: String,
    pub flight_status: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketOperationsPassenger {
    pub ordinal: u8,
    pub display_name: String,
    pub passenger_type: PassengerType,
    pub gender: Gender,
    pub seat: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintableTicketOperationsDetail {
    #[serde(flatten)]
    pub ticket: TicketOperationsDetail,
    pub qr_token: String,
    pub printable: bool,
}
