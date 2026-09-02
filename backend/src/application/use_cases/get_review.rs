use uuid::Uuid;

use crate::domain::{
    repositories::{ReviewRepository, ReviewRepositoryError},
    review::ReviewContext,
};

pub async fn execute(
    repository: &dyn ReviewRepository,
    hold_id: Uuid,
    token_hash: [u8; 32],
) -> Result<ReviewContext, ReviewRepositoryError> {
    repository.get_review(hold_id, token_hash).await
}
