use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::{
    manage_booking::ManageBookingRecord,
    repositories::{ManageBookingRepository, ManageBookingRepositoryError},
};

pub async fn execute(
    repository: &dyn ManageBookingRepository,
    ticket_id: Uuid,
    now: DateTime<Utc>,
) -> Result<Option<ManageBookingRecord>, ManageBookingRepositoryError> {
    repository.get_manage_booking(ticket_id, now).await
}
