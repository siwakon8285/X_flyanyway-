use chrono::{DateTime, Utc};

use crate::domain::{
    manage_booking::{ManageBookingLookup, ManageBookingRecord},
    repositories::{ManageBookingRepository, ManageBookingRepositoryError},
};

pub async fn execute(
    repository: &dyn ManageBookingRepository,
    lookup: &ManageBookingLookup,
    now: DateTime<Utc>,
) -> Result<Option<ManageBookingRecord>, ManageBookingRepositoryError> {
    repository.lookup_manage_booking(lookup, now).await
}
