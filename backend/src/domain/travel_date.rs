use chrono::{Duration, NaiveDate};

/// The inclusive customer travel-date horizon. Historical/admin date filters
/// are separate concerns and must not reuse this policy implicitly.
pub const CUSTOMER_TRAVEL_DATE_HORIZON_DAYS: i64 = 365;

pub struct CustomerTravelDatePolicy;

impl CustomerTravelDatePolicy {
    pub fn is_supported(requested_date: NaiveDate, origin_local_today: NaiveDate) -> bool {
        let Some(latest_allowed) = origin_local_today
            .checked_add_signed(Duration::days(CUSTOMER_TRAVEL_DATE_HORIZON_DAYS))
        else {
            return false;
        };
        requested_date >= origin_local_today && requested_date <= latest_allowed
    }
}
