use uuid::Uuid;

use crate::domain::{
    passengers::{PassengerContext, PassengerInput},
    repositories::{PassengerRepository, PassengerRepositoryError},
};

pub async fn execute(
    repository: &dyn PassengerRepository,
    hold_id: Uuid,
    token_hash: [u8; 32],
    passengers: Vec<PassengerInput>,
) -> Result<PassengerContext, PassengerRepositoryError> {
    repository
        .save_passengers(hold_id, token_hash, passengers)
        .await
}
