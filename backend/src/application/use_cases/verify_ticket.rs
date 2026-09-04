use crate::{
    domain::{
        repositories::{TicketRepository, TicketRepositoryError},
        ticket::TicketVerification,
    },
    infrastructure::ticket::qr::{verify, QrTokenError},
};

pub async fn execute(
    repository: &dyn TicketRepository,
    token: &str,
    secret: &str,
) -> Result<TicketVerification, TicketRepositoryError> {
    let ticket_id = match verify(token, secret) {
        Ok(ticket_id) => ticket_id,
        Err(QrTokenError::Invalid) => return Ok(invalid()),
        Err(QrTokenError::InvalidSecret) => return Err(TicketRepositoryError::IdentityGeneration),
    };
    Ok(repository
        .verify_ticket(ticket_id)
        .await?
        .unwrap_or_else(invalid))
}

fn invalid() -> TicketVerification {
    TicketVerification {
        valid: false,
        ticket_status: None,
        flight_number: None,
        origin_code: None,
        destination_code: None,
        departure_date: None,
        departure_time: None,
        seats: None,
    }
}
