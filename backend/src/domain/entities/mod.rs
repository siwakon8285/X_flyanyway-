use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{CabinClass, PassengerCounts, SeatNumber};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlightSelection {
    pub flight_id: String,
    pub departure_date: NaiveDate,
    pub cabin: CabinClass,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeatAvailability {
    Available,
    HeldByMe,
    Unavailable,
    Booked,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatInventoryItem {
    pub seat_number: SeatNumber,
    pub row_number: i16,
    pub column_code: String,
    pub position: String,
    pub status: SeatAvailability,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatMap {
    pub flight_id: String,
    pub departure_date: NaiveDate,
    pub cabin: CabinClass,
    pub seats: Vec<SeatInventoryItem>,
    pub server_time: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatHold {
    pub id: Uuid,
    pub flight_id: String,
    pub departure_date: NaiveDate,
    pub cabin: CabinClass,
    pub passengers: PassengerCounts,
    pub seats: Vec<SeatNumber>,
    pub expires_at: DateTime<Utc>,
    pub server_time: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct CreateSeatHold {
    pub selection: FlightSelection,
    pub passengers: PassengerCounts,
    pub seats: Vec<SeatNumber>,
    pub token_hash: [u8; 32],
}
