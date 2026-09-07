use std::{fmt, str::FromStr};

use async_trait::async_trait;
use chrono::{DateTime, Duration, FixedOffset, NaiveDate, Utc};
use serde::Serialize;
use thiserror::Error;

use crate::domain::value_objects::CabinClass;

pub const DASHBOARD_TIME_ZONE: &str = "Asia/Bangkok";
pub const DASHBOARD_CURRENCY: &str = "THB";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub enum DashboardProvider {
    #[serde(rename = "STRIPE")]
    #[default]
    Stripe,
    #[serde(rename = "MOCK_BITCOIN")]
    MockBitcoin,
}

impl DashboardProvider {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stripe => "STRIPE",
            Self::MockBitcoin => "MOCK_BITCOIN",
        }
    }
}

impl FromStr for DashboardProvider {
    type Err = DashboardFilterError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "STRIPE" => Ok(Self::Stripe),
            "MOCK_BITCOIN" => Ok(Self::MockBitcoin),
            _ => Err(DashboardFilterError),
        }
    }
}

impl fmt::Display for DashboardProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyticsFilter {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub route: Option<String>,
    pub cabin: Option<CabinClass>,
    pub provider: DashboardProvider,
}

impl AnalyticsFilter {
    pub fn parse(
        from: Option<&str>,
        to: Option<&str>,
        route: Option<&str>,
        cabin: Option<&str>,
        provider: Option<&str>,
        now: DateTime<Utc>,
    ) -> Result<Self, DashboardFilterError> {
        let bangkok = FixedOffset::east_opt(7 * 60 * 60).expect("Bangkok offset is valid");
        let today = now.with_timezone(&bangkok).date_naive();
        let parsed_to = to.map(parse_date).transpose()?.unwrap_or(today);
        let parsed_from = from
            .map(parse_date)
            .transpose()?
            .unwrap_or_else(|| parsed_to - Duration::days(29));
        let inclusive_days = parsed_to.signed_duration_since(parsed_from).num_days() + 1;
        if !(1..=366).contains(&inclusive_days) {
            return Err(DashboardFilterError);
        }
        let route = route.map(str::to_owned);
        if route.as_deref().is_some_and(|value| !valid_route(value)) {
            return Err(DashboardFilterError);
        }
        let cabin = cabin
            .map(CabinClass::from_str)
            .transpose()
            .map_err(|_| DashboardFilterError)?;
        let provider = provider
            .map(DashboardProvider::from_str)
            .transpose()?
            .unwrap_or_default();
        Ok(Self {
            from: parsed_from,
            to: parsed_to,
            route,
            cabin,
            provider,
        })
    }
}

fn parse_date(value: &str) -> Result<NaiveDate, DashboardFilterError> {
    if value.len() != 10
        || value.as_bytes().get(4) != Some(&b'-')
        || value.as_bytes().get(7) != Some(&b'-')
    {
        return Err(DashboardFilterError);
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| DashboardFilterError)
}

fn valid_route(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 7
        && bytes[3] == b'-'
        && bytes[..3]
            .iter()
            .chain(&bytes[4..])
            .all(u8::is_ascii_uppercase)
        && bytes[..3] != bytes[4..]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DashboardFilterError;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardReport {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub time_zone: &'static str,
    pub currency: &'static str,
    pub provider: DashboardProvider,
    pub generated_at: DateTime<Utc>,
    pub summary: DashboardSummary,
    pub trends: Vec<DashboardTrend>,
    pub routes: Vec<DashboardRoute>,
    pub cabins: Vec<DashboardCabin>,
    pub flights: Vec<DashboardFlight>,
    pub revenue_flights: Vec<DashboardFlight>,
    pub inventory: DashboardInventory,
    pub available_routes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub gross_revenue: i64,
    pub total_bookings: i64,
    pub tickets_issued: i64,
    pub cancelled_bookings: i64,
    pub cancellation_rate_percent: Option<f64>,
    pub refund_count: i64,
    pub refund_value: i64,
    pub pending_refund_count: i64,
    pub pending_refund_value: i64,
    pub attention_refund_count: i64,
    pub average_booking_value: Option<f64>,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DashboardTrend {
    pub date: NaiveDate,
    pub bookings: i64,
    pub revenue: i64,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DashboardRoute {
    pub route: String,
    pub bookings: i64,
    pub revenue: i64,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DashboardCabin {
    pub cabin: String,
    pub bookings: i64,
    pub revenue: i64,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DashboardFlight {
    pub flight_number: String,
    pub route: String,
    pub departure_date: NaiveDate,
    pub bookings: i64,
    pub revenue: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardInventory {
    pub booked_seats: i64,
    pub sellable_seats: i64,
    pub occupancy_percent: Option<f64>,
    pub flights: Vec<DashboardInventoryFlight>,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DashboardInventoryFlight {
    pub flight_number: String,
    pub route: String,
    pub departure_date: NaiveDate,
    pub booked_seats: i64,
    pub sellable_seats: i64,
    pub occupancy_percent: Option<f64>,
}

#[derive(Debug, Error)]
pub enum AnalyticsRepositoryError {
    #[error("dashboard storage is unavailable")]
    Infrastructure(#[source] sqlx::Error),
}

#[async_trait]
pub trait AnalyticsRepository: Send + Sync {
    async fn dashboard(
        &self,
        filter: &AnalyticsFilter,
    ) -> Result<DashboardReport, AnalyticsRepositoryError>;
}
