use uuid::Uuid;

use crate::domain::repositories::{SeatHoldRepository, SeatHoldRepositoryError};

pub async fn execute(
    repository: &dyn SeatHoldRepository,
    hold_id: Uuid,
    token_hash: [u8; 32],
) -> Result<(), SeatHoldRepositoryError> {
    repository.release_hold(hold_id, token_hash).await
}
