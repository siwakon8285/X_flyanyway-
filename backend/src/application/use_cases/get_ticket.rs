use uuid::Uuid;

use crate::domain::{
    repositories::{TicketRepository, TicketRepositoryError},
    ticket::Ticket,
};

pub async fn execute(
    repository: &dyn TicketRepository,
    hold_id: Uuid,
    token_hash: [u8; 32],
    payment_attempt_id: Uuid,
) -> Result<Ticket, TicketRepositoryError> {
    repository
        .issue_ticket(hold_id, token_hash, payment_attempt_id)
        .await
}
