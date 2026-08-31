use std::time::Duration;

use crate::domain::{
    entities::{CreateSeatHold, SeatHold},
    repositories::{SeatHoldRepository, SeatHoldRepositoryError},
};

pub async fn execute(
    repository: &dyn SeatHoldRepository,
    command: CreateSeatHold,
    hold_ttl: Duration,
) -> Result<SeatHold, SeatHoldRepositoryError> {
    repository.create_hold(command, hold_ttl).await
}
