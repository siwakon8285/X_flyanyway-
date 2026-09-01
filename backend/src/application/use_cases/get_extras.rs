use uuid::Uuid;

use crate::domain::{
    extras::ExtraContext,
    repositories::{ExtraRepository, ExtraRepositoryError},
};

pub async fn execute(
    repository: &dyn ExtraRepository,
    hold_id: Uuid,
    token_hash: [u8; 32],
) -> Result<ExtraContext, ExtraRepositoryError> {
    repository.get_extras(hold_id, token_hash).await
}
