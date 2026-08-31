use uuid::Uuid;

use crate::domain::{
    entities::{FlightSelection, SeatMap},
    repositories::{SeatHoldRepository, SeatHoldRepositoryError},
};

pub async fn execute(
    repository: &dyn SeatHoldRepository,
    selection: &FlightSelection,
    owner: Option<(Uuid, [u8; 32])>,
) -> Result<SeatMap, SeatHoldRepositoryError> {
    repository.seat_map(selection, owner).await
}
