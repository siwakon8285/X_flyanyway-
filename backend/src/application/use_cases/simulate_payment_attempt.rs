use crate::domain::{
    payment::{
        PaymentAttempt, PaymentAttemptTransition, PaymentMethod, PaymentSimulationOutcome,
        PaymentStatus,
    },
    repositories::{PaymentRepository, PaymentRepositoryError},
};
use uuid::Uuid;

pub async fn execute(
    repository: &dyn PaymentRepository,
    hold_id: Uuid,
    token_hash: [u8; 32],
    attempt_id: Uuid,
    outcome: PaymentSimulationOutcome,
) -> Result<PaymentAttempt, PaymentRepositoryError> {
    let attempt = repository
        .get_payment_attempt(hold_id, token_hash, attempt_id)
        .await?;
    if attempt.payment_method != PaymentMethod::Bitcoin
        || attempt.status != PaymentStatus::AwaitingPayment
    {
        return Err(PaymentRepositoryError::InvalidTransition);
    }
    match outcome {
        PaymentSimulationOutcome::Received => {
            repository
                .transition_payment_attempt(
                    hold_id,
                    token_hash,
                    attempt_id,
                    PaymentAttemptTransition::processing(
                        attempt
                            .provider_reference
                            .as_deref()
                            .unwrap_or("XFBTC-DEMO"),
                    ),
                )
                .await?;
            repository
                .transition_payment_attempt(
                    hold_id,
                    token_hash,
                    attempt_id,
                    PaymentAttemptTransition::succeeded(
                        attempt
                            .provider_reference
                            .as_deref()
                            .unwrap_or("XFBTC-DEMO"),
                    ),
                )
                .await
        }
        PaymentSimulationOutcome::Failed => {
            repository
                .transition_payment_attempt(
                    hold_id,
                    token_hash,
                    attempt_id,
                    PaymentAttemptTransition::failed(
                        "MOCK_BITCOIN_FAILED",
                        "The demo Bitcoin payment failed.",
                    ),
                )
                .await
        }
        PaymentSimulationOutcome::Cancelled => {
            repository
                .transition_payment_attempt(
                    hold_id,
                    token_hash,
                    attempt_id,
                    PaymentAttemptTransition::cancelled(
                        "MOCK_BITCOIN_CANCELLED",
                        "The demo Bitcoin invoice was cancelled.",
                    ),
                )
                .await
        }
    }
}
