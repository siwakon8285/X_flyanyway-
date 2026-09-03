use crate::domain::{
    payment::{ProcessStripeWebhookCommand, StripeWebhookResult},
    repositories::{PaymentRepository, PaymentRepositoryError},
};

pub async fn execute(
    repository: &dyn PaymentRepository,
    command: ProcessStripeWebhookCommand,
) -> Result<StripeWebhookResult, PaymentRepositoryError> {
    repository.process_stripe_webhook(command).await
}
