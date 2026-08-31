use uuid::Uuid;

use crate::domain::{
    entities::SeatHold,
    repositories::{SeatHoldRepository, SeatHoldRepositoryError},
    value_objects::SeatNumber,
};

pub async fn execute(
    repository: &dyn SeatHoldRepository,
    hold_id: Uuid,
    token_hash: [u8; 32],
    seats: Vec<SeatNumber>,
) -> Result<SeatHold, SeatHoldRepositoryError> {
    repository.replace_seats(hold_id, token_hash, seats).await
}
