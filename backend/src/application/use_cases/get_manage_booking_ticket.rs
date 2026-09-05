use uuid::Uuid;

use crate::domain::{
    repositories::{ManageBookingRepository, ManageBookingRepositoryError},
    ticket::Ticket,
};

pub async fn execute(
    repository: &dyn ManageBookingRepository,
    ticket_id: Uuid,
) -> Result<Option<Ticket>, ManageBookingRepositoryError> {
    repository.get_manage_booking_ticket(ticket_id).await
}
