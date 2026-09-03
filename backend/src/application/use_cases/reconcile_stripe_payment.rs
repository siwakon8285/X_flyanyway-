use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    payment::{
        PaymentAttempt, PaymentAttemptTransition, PaymentFailure, PaymentProviderReconciler,
        PaymentProviderState, PaymentReconciliationStatus, PaymentStatus,
    },
    repositories::{PaymentRepository, PaymentRepositoryError},
};

/// Reconciles one authorized, open Stripe Card attempt. Provider outages,
/// missing references, and invariant mismatches deliberately preserve the open
/// local attempt: an external charge might still exist, so inventory remains
/// protected and no second PaymentIntent is created here.
pub async fn execute(
    repository: &dyn PaymentRepository,
    provider: &dyn PaymentProviderReconciler,
    hold_id: Uuid,
    token_hash: [u8; 32],
    attempt: &PaymentAttempt,
) -> Result<bool, PaymentRepositoryError> {
    let Some(reference) = attempt.provider_reference.as_deref() else {
        tracing::warn!(attempt_id = %attempt.id, "open Stripe attempt has no provider reference");
        return Ok(false);
    };

    let mut state = match provider.retrieve_payment_intent(reference).await {
        Ok(state) => state,
        Err(_) => return Ok(false),
    };
    if !matches_amount_and_currency(attempt, &state) || state.provider_reference != reference {
        tracing::warn!(attempt_id = %attempt.id, "Stripe reconciliation invariant mismatch");
        return Ok(false);
    }

    if unresolved(state.status)
        && attempt
            .payment_finalization_deadline
            .is_some_and(|deadline| deadline <= Utc::now())
    {
        // Cancellation is a best effort request, never a local terminal state
        // by itself. Only the returned confirmed terminal state can close this
        // protected attempt.
        if let Ok(cancelled) = provider.cancel_payment_intent(reference).await {
            if matches_amount_and_currency(attempt, &cancelled)
                && cancelled.provider_reference == reference
            {
                state = cancelled;
            }
        }
    }

    let next = match state.status {
        PaymentReconciliationStatus::AwaitingCustomer => PaymentStatus::AwaitingPayment,
        PaymentReconciliationStatus::Processing => PaymentStatus::Processing,
        PaymentReconciliationStatus::Succeeded => PaymentStatus::Succeeded,
        PaymentReconciliationStatus::Failed => PaymentStatus::Failed,
        PaymentReconciliationStatus::Cancelled => PaymentStatus::Cancelled,
    };
    if next == attempt.status {
        return Ok(false);
    }
    let transition = match next {
        PaymentStatus::Succeeded => PaymentAttemptTransition::succeeded(reference),
        PaymentStatus::Failed => failure_transition(PaymentStatus::Failed, state.failure),
        PaymentStatus::Cancelled => failure_transition(PaymentStatus::Cancelled, state.failure),
        PaymentStatus::Processing => PaymentAttemptTransition::processing(reference),
        PaymentStatus::AwaitingPayment => PaymentAttemptTransition::awaiting_payment(reference),
        PaymentStatus::Created => return Ok(false),
    };
    match repository
        .transition_payment_attempt(hold_id, token_hash, attempt.id, transition)
        .await
    {
        Ok(_) => Ok(true),
        // Webhooks and polling deliberately race; reload the authoritative row.
        Err(PaymentRepositoryError::InvalidTransition) => Ok(true),
        Err(error) => Err(error),
    }
}

fn unresolved(status: PaymentReconciliationStatus) -> bool {
    matches!(
        status,
        PaymentReconciliationStatus::AwaitingCustomer | PaymentReconciliationStatus::Processing
    )
}

fn matches_amount_and_currency(attempt: &PaymentAttempt, state: &PaymentProviderState) -> bool {
    attempt.amount.currency_code == "THB"
        && attempt
            .amount
            .amount
            .checked_mul(100)
            .is_some_and(|expected| expected == state.amount)
        && state.currency.eq_ignore_ascii_case("thb")
}

fn failure_transition(
    status: PaymentStatus,
    failure: Option<PaymentFailure>,
) -> PaymentAttemptTransition {
    let failure = failure.unwrap_or(PaymentFailure {
        code: if status == PaymentStatus::Cancelled {
            "PAYMENT_CANCELLED".to_owned()
        } else {
            "PROCESSING_ERROR".to_owned()
        },
        message: if status == PaymentStatus::Cancelled {
            "The payment was cancelled.".to_owned()
        } else {
            "The payment could not be processed.".to_owned()
        },
    });
    PaymentAttemptTransition {
        status,
        provider_reference: None,
        failure: Some(failure),
    }
}
