use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_PRICE_AMOUNT: i64 = 100_000_000;
const MAX_CABIN_CAPACITY: u16 = 200;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlightNumber(String);

impl FlightNumber {
    pub fn parse(value: &str) -> Result<Self, FlightValidationError> {
        let compact = value
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_uppercase();
        let valid = compact.strip_prefix("XF ").is_some_and(|digits| {
            digits.len() == 3 && digits.bytes().all(|byte| byte.is_ascii_digit())
        });
        valid
            .then_some(Self(compact))
            .ok_or(FlightValidationError::FlightNumber)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlightCommand {
    pub flight_number: String,
    pub origin_code: String,
    pub destination_code: String,
    pub operating_date: Option<NaiveDate>,
    pub departure_time: NaiveTime,
    pub arrival_time: NaiveTime,
    pub arrival_day_offset: i16,
    pub aircraft_code: String,
    pub business_price_amount: i64,
    pub first_price_amount: i64,
    pub currency_code: String,
    pub business_capacity: u16,
    pub first_capacity: u16,
}

impl FlightCommand {
    pub fn validate(self) -> Result<ValidatedFlightCommand, FlightValidationError> {
        let flight_number = FlightNumber::parse(&self.flight_number)?;
        let origin_code = normalize_airport(&self.origin_code)?;
        let destination_code = normalize_airport(&self.destination_code)?;
        if origin_code == destination_code {
            return Err(FlightValidationError::Route);
        }
        if !(0..=1).contains(&self.arrival_day_offset) {
            return Err(FlightValidationError::Schedule);
        }
        let aircraft_code = self.aircraft_code.trim().to_owned();
        if aircraft_code.is_empty() || aircraft_code.chars().count() > 80 {
            return Err(FlightValidationError::Aircraft);
        }
        if self.currency_code != "THB"
            || !(1..=MAX_PRICE_AMOUNT).contains(&self.business_price_amount)
            || !(1..=MAX_PRICE_AMOUNT).contains(&self.first_price_amount)
        {
            return Err(FlightValidationError::Price);
        }
        if self.business_capacity == 0
            || self.business_capacity > MAX_CABIN_CAPACITY
            || !self.business_capacity.is_multiple_of(4)
            || self.first_capacity == 0
            || self.first_capacity > MAX_CABIN_CAPACITY
            || !self.first_capacity.is_multiple_of(2)
        {
            return Err(FlightValidationError::Capacity);
        }
        Ok(ValidatedFlightCommand {
            flight_number,
            origin_code,
            destination_code,
            operating_date: self.operating_date,
            departure_time: self.departure_time,
            arrival_time: self.arrival_time,
            arrival_day_offset: self.arrival_day_offset,
            aircraft_code,
            business_price_amount: self.business_price_amount,
            first_price_amount: self.first_price_amount,
            currency_code: self.currency_code,
            business_capacity: self.business_capacity,
            first_capacity: self.first_capacity,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedFlightCommand {
    pub flight_number: FlightNumber,
    pub origin_code: String,
    pub destination_code: String,
    pub operating_date: Option<NaiveDate>,
    pub departure_time: NaiveTime,
    pub arrival_time: NaiveTime,
    pub arrival_day_offset: i16,
    pub aircraft_code: String,
    pub business_price_amount: i64,
    pub first_price_amount: i64,
    pub currency_code: String,
    pub business_capacity: u16,
    pub first_capacity: u16,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FlightStatus {
    Scheduled,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlightValidationError {
    FlightNumber,
    Route,
    Schedule,
    TimeZone,
    Aircraft,
    Price,
    Capacity,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedCabin {
    pub available: bool,
    pub price_amount: Option<i64>,
    pub currency_code: Option<String>,
    pub capacity: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlightRecord {
    pub id: Uuid,
    pub public_id: String,
    pub flight_number: String,
    pub origin_code: String,
    pub destination_code: String,
    pub origin_time_zone: String,
    pub destination_time_zone: String,
    pub operating_date: Option<NaiveDate>,
    pub departure_time: Option<NaiveTime>,
    pub arrival_time: Option<NaiveTime>,
    pub arrival_day_offset: Option<i16>,
    pub aircraft_code: String,
    pub status: FlightStatus,
    pub business: ManagedCabin,
    pub first: ManagedCabin,
    pub version: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlightManagementError {
    NotFound,
    Validation,
    Duplicate,
    Conflict,
    StructuralConflict,
    InvalidStatus,
    Infrastructure,
}

fn normalize_airport(value: &str) -> Result<String, FlightValidationError> {
    let value = value.trim().to_uppercase();
    (value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_uppercase()))
        .then_some(value)
        .ok_or(FlightValidationError::Route)
}
