use uuid::Uuid;

use crate::domain::{
    passengers::PassengerContext,
    repositories::{PassengerRepository, PassengerRepositoryError},
};

pub async fn execute(
    repository: &dyn PassengerRepository,
    hold_id: Uuid,
    token_hash: [u8; 32],
) -> Result<PassengerContext, PassengerRepositoryError> {
    repository.get_passengers(hold_id, token_hash).await
}
