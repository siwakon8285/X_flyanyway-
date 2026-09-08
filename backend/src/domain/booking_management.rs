use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

use crate::domain::{
    cancellation::RefundStatus,
    extras::Money,
    manage_booking::{BookingStatus, CancellationEligibility},
    passengers::{Gender, PassengerType},
    payment::{PaymentMethod, PaymentProvider, PaymentStatus},
    ticket::TicketStatus,
};

#[derive(Clone, Debug)]
pub struct BookingListFilter {
    pub booking_reference: Option<String>,
    pub passenger_name: Option<String>,
    pub flight_number: Option<String>,
    pub travel_date: Option<NaiveDate>,
    pub booking_status: Option<BookingStatus>,
    pub cabin: Option<String>,
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingListPage {
    pub items: Vec<BookingListItem>,
    pub next_offset: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingListItem {
    pub booking_reference: String,
    pub lead_passenger_name: String,
    pub passenger_count: i64,
    pub flight_number: String,
    pub origin_code: String,
    pub destination_code: String,
    pub travel_date: NaiveDate,
    pub departure_at: Option<DateTime<Utc>>,
    pub cabin: String,
    pub booking_status: BookingStatus,
    pub flight_status: String,
    pub payment_status: PaymentStatus,
    pub ticket_status: TicketStatus,
    pub refund_status: Option<RefundStatus>,
    pub booked_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingDetail {
    pub booking_reference: String,
    pub booking_status: BookingStatus,
    pub created_at: DateTime<Utc>,
    pub journey: BookingJourney,
    pub passengers: Vec<BookingPassenger>,
    pub seats: Vec<String>,
    pub contact: Option<BookingContactSummary>,
    pub payment: BookingPaymentSummary,
    pub ticket: BookingTicketSummary,
    pub cancellation: BookingCancellationSummary,
    pub audit: Vec<BookingAuditEntry>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingJourney {
    pub flight_number: String,
    pub origin_code: String,
    pub destination_code: String,
    pub travel_date: NaiveDate,
    pub departure_at: Option<DateTime<Utc>>,
    pub departure_time: Option<String>,
    pub arrival_date: Option<NaiveDate>,
    pub arrival_time: Option<String>,
    pub origin_time_zone: Option<String>,
    pub aircraft_code: String,
    pub cabin: String,
    pub flight_status: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingPassenger {
    pub ordinal: u8,
    pub passenger_type: PassengerType,
    pub display_name: String,
    pub gender: Gender,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingContactSummary {
    pub phone_country_code: String,
    pub phone_number: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingPaymentSummary {
    pub method: PaymentMethod,
    pub provider: PaymentProvider,
    pub status: PaymentStatus,
    pub amount: Money,
    pub succeeded_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingTicketSummary {
    pub ticket_number: String,
    pub status: TicketStatus,
    pub issued_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingCancellationSummary {
    pub eligibility: CancellationEligibility,
    pub cutoff_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub refund_status: Option<RefundStatus>,
    pub refund_amount: Option<Money>,
    pub refunded_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingAuditEntry {
    pub action: String,
    pub actor_email: String,
    pub created_at: DateTime<Utc>,
}
