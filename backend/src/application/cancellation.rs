use crate::domain::{
    booking_management::BookingDetail,
    cancellation::{
        Cancellation, Clock, RefundFailure, StaffCancellationActor, StripeRefundEvent,
        MAX_REFUND_ATTEMPTS,
    },
    payment::PaymentProvider,
    repositories::{CancellationRepository, CancellationRepositoryError, RefundGateway},
};
use chrono::{DateTime, Duration, Utc};
use std::{fmt, sync::Arc};
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
            .cancel_booking(ticket_id, self.clock.as_ref(), None)
            .await
    }
    pub async fn cancel_for_staff(
        &self,
        ticket_id: Uuid,
        actor: &StaffCancellationActor,
    ) -> Result<BookingDetail, CancellationRepositoryError> {
        self.repository
            .cancel_booking_for_staff(ticket_id, self.clock.as_ref(), actor)
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

pub struct RefundDispatchError {
    source: sqlx::Error,
    stage: &'static str,
    job_id: Option<Uuid>,
}

impl fmt::Debug for RefundDispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RefundDispatchError")
            .field("stage", &self.stage)
            .field("job_id", &self.job_id)
            .field("source", &"[redacted]")
            .finish()
    }
}

impl RefundDispatchError {
    fn new(stage: &'static str, job_id: Option<Uuid>, source: sqlx::Error) -> Self {
        Self {
            source,
            stage,
            job_id,
        }
    }

    pub fn source(&self) -> &sqlx::Error {
        &self.source
    }

    pub fn stage(&self) -> &'static str {
        self.stage
    }

    pub fn job_id(&self) -> Option<Uuid> {
        self.job_id
    }
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
    pub async fn dispatch_once(&self, now: DateTime<Utc>) -> Result<bool, RefundDispatchError> {
        let Some(job) = self
            .repository
            .claim_due_refund(now)
            .await
            .map_err(|source| RefundDispatchError::new("claim", None, source))?
        else {
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
                    .await
                    .map_err(|source| {
                        RefundDispatchError::new("mark_attention", Some(job.id), source)
                    })?
            }
            Ok(refund) => self
                .repository
                .mark_refund_result(&job, &refund, now)
                .await
                .map_err(|source| RefundDispatchError::new("mark_result", Some(job.id), source))?,
            Err(f @ RefundFailure::Permanent(_)) => self
                .repository
                .mark_refund_attention(&job, &f)
                .await
                .map_err(|source| {
                    RefundDispatchError::new("mark_attention", Some(job.id), source)
                })?,
            Err(f @ RefundFailure::Transient(_)) if job.attempt_count >= MAX_REFUND_ATTEMPTS => {
                self.repository
                    .mark_refund_attention(&job, &f)
                    .await
                    .map_err(|source| {
                        RefundDispatchError::new("mark_attention", Some(job.id), source)
                    })?
            }
            Err(f @ RefundFailure::Transient(_)) => {
                let seconds = [60, 300, 1800, 7200, 21600]
                    [usize::from(job.attempt_count.saturating_sub(1)).min(4)];
                self.repository
                    .mark_refund_retry(&job, now + Duration::seconds(seconds), &f)
                    .await
                    .map_err(|source| {
                        RefundDispatchError::new("mark_retry", Some(job.id), source)
                    })?
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refund_dispatch_errors_expose_only_safe_context_fields() {
        let job_id = Uuid::new_v4();
        let error = RefundDispatchError::new(
            "mark_result",
            Some(job_id),
            sqlx::Error::Protocol("private database detail".to_owned()),
        );
        assert_eq!(error.stage(), "mark_result");
        assert_eq!(error.job_id(), Some(job_id));
        assert!(matches!(error.source(), sqlx::Error::Protocol(_)));
        assert!(!format!("{error:?}").contains("private database detail"));
    }
}
