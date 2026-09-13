use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Duration, FixedOffset, NaiveDate, Utc};
use serde::Serialize;
use thiserror::Error;

use crate::domain::value_objects::CabinClass;

pub const EXTERNAL_ANALYTICS_TIME_ZONE: &str = "Asia/Bangkok";
const MAX_PERIOD_DAYS: i64 = 366;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalAnalyticsFilter {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub route: Option<String>,
    pub cabin: Option<CabinClass>,
}

impl ExternalAnalyticsFilter {
    pub fn parse(
        from: Option<&str>,
        to: Option<&str>,
        route: Option<&str>,
        cabin: Option<&str>,
        now: DateTime<Utc>,
    ) -> Result<Self, ExternalAnalyticsFilterError> {
        let bangkok = FixedOffset::east_opt(7 * 60 * 60).expect("Bangkok offset is valid");
        let today = now.with_timezone(&bangkok).date_naive();
        let parsed_to = to.map(parse_date).transpose()?.unwrap_or(today);
        let parsed_from = from
            .map(parse_date)
            .transpose()?
            .unwrap_or_else(|| parsed_to - Duration::days(29));
        let inclusive_days = parsed_to.signed_duration_since(parsed_from).num_days() + 1;
        if !(1..=MAX_PERIOD_DAYS).contains(&inclusive_days) {
            return Err(ExternalAnalyticsFilterError::InvalidRange);
        }

        let route = route.map(str::to_owned);
        if route.as_deref().is_some_and(|value| !valid_route(value)) {
            return Err(ExternalAnalyticsFilterError::InvalidRoute);
        }

        let cabin = cabin
            .map(CabinClass::from_str)
            .transpose()
            .map_err(|_| ExternalAnalyticsFilterError::InvalidCabin)?;
        if cabin.is_some_and(|value| !value.is_customer_bookable()) {
            return Err(ExternalAnalyticsFilterError::InvalidCabin);
        }

        Ok(Self {
            from: parsed_from,
            to: parsed_to,
            route,
            cabin,
        })
    }
}

fn parse_date(value: &str) -> Result<NaiveDate, ExternalAnalyticsFilterError> {
    if value.len() != 10
        || value.as_bytes().get(4) != Some(&b'-')
        || value.as_bytes().get(7) != Some(&b'-')
    {
        return Err(ExternalAnalyticsFilterError::InvalidDate);
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| ExternalAnalyticsFilterError::InvalidDate)
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

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ExternalAnalyticsFilterError {
    #[error("analytics date is invalid")]
    InvalidDate,
    #[error("analytics date range is invalid")]
    InvalidRange,
    #[error("analytics route is invalid")]
    InvalidRoute,
    #[error("analytics cabin is invalid")]
    InvalidCabin,
}

#[derive(Debug, Error)]
pub enum ExternalAnalyticsRepositoryError {
    #[error("external analytics storage is unavailable")]
    Infrastructure(#[source] sqlx::Error),
    #[error("external analytics aggregate is inconsistent")]
    InconsistentAggregate,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAnalyticsPeriod {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAnalyticsSummary {
    pub period: ExternalAnalyticsPeriod,
    pub generated_at: DateTime<Utc>,
    pub total_bookings: i64,
    pub tickets_issued: i64,
    pub cancelled_bookings: i64,
    pub booked_seats: i64,
    pub sellable_seats: i64,
    pub occupancy_percent: f64,
}

#[async_trait]
pub trait ExternalAnalyticsRepository: Send + Sync {
    async fn summary(
        &self,
        filter: &ExternalAnalyticsFilter,
    ) -> Result<ExternalAnalyticsSummary, ExternalAnalyticsRepositoryError>;
}

#[derive(Clone)]
pub struct ExternalAnalyticsService {
    repository: Arc<dyn ExternalAnalyticsRepository>,
}

impl ExternalAnalyticsService {
    pub fn new(repository: Arc<dyn ExternalAnalyticsRepository>) -> Self {
        Self { repository }
    }

    pub async fn summary(
        &self,
        filter: ExternalAnalyticsFilter,
        now: DateTime<Utc>,
    ) -> Result<ExternalAnalyticsSummary, ExternalAnalyticsRepositoryError> {
        let mut summary = self.repository.summary(&filter).await?;
        summary.generated_at = now;
        Ok(summary)
    }
}
