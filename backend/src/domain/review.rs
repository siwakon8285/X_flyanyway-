use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::{
    entities::SeatHold,
    extras::{BaggageAllowances, IncludedBenefits, Money, PricedExtraSelection},
    passengers::{PassengerType, Title},
    value_objects::SeatNumber,
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StopType {
    Direct,
    OneStop,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewJourney {
    pub flight_number: String,
    pub origin_code: String,
    pub destination_code: String,
    pub aircraft_code: String,
    pub departure_time: String,
    pub arrival_time: String,
    pub arrival_day_offset: u8,
    pub duration_minutes: u16,
    pub stops: StopType,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewPassenger {
    pub ordinal: u8,
    pub passenger_type: PassengerType,
    pub title: Title,
    pub display_name: String,
    pub nationality_code: String,
    pub travel_document_complete: bool,
    pub extras: Vec<PricedExtraSelection>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSeat {
    pub seat_number: SeatNumber,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewIncludedBenefits {
    pub allowances: BaggageAllowances,
    pub benefits: IncludedBenefits,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewBaseFareLine {
    pub passenger_type: PassengerType,
    pub quantity: u8,
    pub unit_amount: Money,
    pub amount: Money,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewBaseFare {
    pub lines: Vec<ReviewBaseFareLine>,
    pub amount: Money,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewPricingLine {
    pub code: &'static str,
    pub quantity: u8,
    pub unit_amount: Money,
    pub amount: Money,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewPricing {
    pub currency_code: String,
    pub base_fare: ReviewBaseFare,
    pub extras: Money,
    pub taxes: Vec<ReviewPricingLine>,
    pub fees: Vec<ReviewPricingLine>,
    pub grand_total: Money,
    pub priced_at: DateTime<Utc>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoFareConditions {
    pub code: &'static str,
    pub fixture: bool,
    pub refundable: bool,
    pub changes_allowed: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewContext {
    pub hold: SeatHold,
    pub journey: ReviewJourney,
    pub passengers: Vec<ReviewPassenger>,
    pub seats: Vec<ReviewSeat>,
    pub included_benefits: ReviewIncludedBenefits,
    pub pricing: ReviewPricing,
    pub fare_conditions: DemoFareConditions,
    pub ready_for_payment: bool,
}
