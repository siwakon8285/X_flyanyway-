use crate::domain::{
    cancellation::{Cancellation, Clock, RefundFailure, StripeRefundEvent, MAX_REFUND_ATTEMPTS},
    payment::PaymentProvider,
    repositories::{CancellationRepository, CancellationRepositoryError, RefundGateway},
};
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct CancellationService {
    repository: Arc<dyn CancellationRepository>,
    clock: Arc<dyn Clock>,
}
impl CancellationService {
    pub fn new(repository: Arc<dyn CancellationRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { repository, clock }
    }
    pub async fn cancel(
        &self,
        ticket_id: Uuid,
    ) -> Result<Cancellation, CancellationRepositoryError> {
        self.repository
            .cancel_booking(ticket_id, self.clock.as_ref())
            .await
    }
    pub async fn process_stripe_refund_event(
        &self,
        event: StripeRefundEvent,
    ) -> Result<(), CancellationRepositoryError> {
        self.repository.process_stripe_refund_event(event).await
    }
}

#[derive(Clone)]
pub struct RefundDispatcher {
    repository: Arc<dyn CancellationRepository>,
    stripe: Option<Arc<dyn RefundGateway>>,
    mock_bitcoin: Arc<dyn RefundGateway>,
}
impl RefundDispatcher {
    pub fn new(
        repository: Arc<dyn CancellationRepository>,
        stripe: Option<Arc<dyn RefundGateway>>,
        mock_bitcoin: Arc<dyn RefundGateway>,
    ) -> Self {
        Self {
            repository,
            stripe,
            mock_bitcoin,
        }
    }
    pub async fn dispatch_once(&self, now: DateTime<Utc>) -> Result<bool, sqlx::Error> {
        let Some(job) = self.repository.claim_due_refund(now).await? else {
            return Ok(false);
        };
        let result = match job.provider {
            PaymentProvider::Stripe => match &self.stripe {
                Some(g) => g.refund(&job).await,
                None => Err(RefundFailure::Permanent("STRIPE_REFUND_UNAVAILABLE")),
            },
            PaymentProvider::MockBitcoin => self.mock_bitcoin.refund(&job).await,
        };
        match result {
            Ok(refund)
                if refund.status
                    == crate::domain::cancellation::ProviderRefundStatus::Processing
                    && job.attempt_count >= MAX_REFUND_ATTEMPTS =>
            {
                self.repository
                    .mark_refund_attention(
                        &job,
                        &RefundFailure::Permanent("REFUND_STILL_PROCESSING"),
                    )
                    .await?
            }
            Ok(refund) => {
                self.repository
                    .mark_refund_result(&job, &refund, now)
                    .await?
            }
            Err(f @ RefundFailure::Permanent(_)) => {
                self.repository.mark_refund_attention(&job, &f).await?
            }
            Err(f @ RefundFailure::Transient(_)) if job.attempt_count >= MAX_REFUND_ATTEMPTS => {
                self.repository.mark_refund_attention(&job, &f).await?
            }
            Err(f @ RefundFailure::Transient(_)) => {
                let seconds = [60, 300, 1800, 7200, 21600]
                    [usize::from(job.attempt_count.saturating_sub(1)).min(4)];
                self.repository
                    .mark_refund_retry(&job, now + Duration::seconds(seconds), &f)
                    .await?
            }
        }
        Ok(true)
    }
}
