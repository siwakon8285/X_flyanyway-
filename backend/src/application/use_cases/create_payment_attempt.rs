use crate::domain::{
    payment::{
        CreatePaymentRequest, PaymentAttempt, PaymentAttemptTransition, PaymentGateway,
        PaymentGatewayOutcome, PaymentGatewayRequest, PaymentMethod, PaymentProvider,
        PaymentRepositoryCommand, PaymentStatus,
    },
    repositories::{PaymentRepository, PaymentRepositoryError},
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub async fn execute(
    repository: &dyn PaymentRepository,
    card_gateway: &dyn PaymentGateway,
    bitcoin_gateway: &dyn PaymentGateway,
    hold_id: Uuid,
    token_hash: [u8; 32],
    request: CreatePaymentRequest,
) -> Result<PaymentAttempt, PaymentRepositoryError> {
    if (request.method == PaymentMethod::Card) != request.scenario.is_some() {
        return Err(PaymentRepositoryError::InvalidRequest);
    }
    let fingerprint: [u8; 32] = Sha256::digest(format!(
        "{}:{}",
        request.method.as_str(),
        request.scenario.map(|value| value.as_str()).unwrap_or("")
    ))
    .into();
    let provider = match request.method {
        PaymentMethod::Card => PaymentProvider::MockCard,
        PaymentMethod::Bitcoin => PaymentProvider::MockBitcoin,
    };
    let attempt = repository
        .create_payment_attempt(
            hold_id,
            token_hash,
            PaymentRepositoryCommand {
                request_id: request.request_id,
                request_fingerprint: fingerprint,
                method: request.method,
                provider,
            },
        )
        .await?;
    if attempt.status != PaymentStatus::Created {
        return Ok(attempt);
    }
    let gateway = match request.method {
        PaymentMethod::Card => {
            repository
                .transition_payment_attempt(
                    hold_id,
                    token_hash,
                    attempt.id,
                    PaymentAttemptTransition::processing(format!("XFCARD-{}", attempt.id.simple())),
                )
                .await?;
            card_gateway
        }
        PaymentMethod::Bitcoin => bitcoin_gateway,
    };
    let outcome = gateway
        .initiate(PaymentGatewayRequest {
            attempt_id: attempt.id,
            amount: attempt.amount,
            scenario: request.scenario,
        })
        .await;
    let transition = match outcome {
        PaymentGatewayOutcome::Succeeded { provider_reference } => {
            PaymentAttemptTransition::succeeded(provider_reference)
        }
        PaymentGatewayOutcome::Failed {
            provider_reference,
            code,
            message,
        } => PaymentAttemptTransition {
            status: PaymentStatus::Failed,
            provider_reference: Some(provider_reference),
            failure: Some(crate::domain::payment::PaymentFailure {
                code: code.to_owned(),
                message: message.to_owned(),
            }),
        },
        PaymentGatewayOutcome::AwaitingPayment { provider_reference } => {
            PaymentAttemptTransition::awaiting_payment(provider_reference)
        }
    };
    repository
        .transition_payment_attempt(hold_id, token_hash, attempt.id, transition)
        .await
}
