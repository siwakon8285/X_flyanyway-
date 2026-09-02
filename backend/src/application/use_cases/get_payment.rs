use crate::domain::{
    payment::PaymentContext,
    repositories::{PaymentRepository, PaymentRepositoryError},
};
use uuid::Uuid;

pub async fn execute(
    repository: &dyn PaymentRepository,
    hold_id: Uuid,
    token_hash: [u8; 32],
) -> Result<PaymentContext, PaymentRepositoryError> {
    repository.get_payment(hold_id, token_hash).await
}
