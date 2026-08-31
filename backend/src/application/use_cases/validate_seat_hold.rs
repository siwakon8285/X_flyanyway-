use uuid::Uuid;

use crate::domain::{
    entities::SeatHold,
    repositories::{SeatHoldRepository, SeatHoldRepositoryError},
};

pub async fn execute(
    repository: &dyn SeatHoldRepository,
    hold_id: Uuid,
    token_hash: [u8; 32],
) -> Result<SeatHold, SeatHoldRepositoryError> {
    let hold = repository.get_hold(hold_id, token_hash).await?;
    if hold.seats.len() != hold.passengers.required_seats() {
        return Err(SeatHoldRepositoryError::SeatCountMismatch);
    }
    Ok(hold)
}
