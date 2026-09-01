use uuid::Uuid;

use crate::domain::{
    extras::{ExtraContext, ExtraSelectionInput},
    repositories::{ExtraRepository, ExtraRepositoryError},
};

pub async fn execute(
    repository: &dyn ExtraRepository,
    hold_id: Uuid,
    token_hash: [u8; 32],
    selections: Vec<ExtraSelectionInput>,
) -> Result<ExtraContext, ExtraRepositoryError> {
    repository
        .save_extras(hold_id, token_hash, selections)
        .await
}
