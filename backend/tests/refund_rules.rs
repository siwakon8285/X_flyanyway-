use async_trait::async_trait;
use chrono::Utc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use x_fly_api::{
    application::cancellation::RefundDispatcher,
    domain::{
        booking_management::BookingDetail,
        cancellation::{
            Cancellation, Clock, ProviderRefund, ProviderRefundStatus, RefundFailure, RefundJob,
            StripeRefundEvent,
        },
        extras::Money,
        payment::PaymentProvider,
        repositories::{CancellationRepository, CancellationRepositoryError, RefundGateway},
    },
    infrastructure::{
        payment::stripe::StripeAmount,
        refund::{stripe::map_status, MockBitcoinRefundGateway},
    },
};

struct RecordingRepository {
    job: Mutex<Option<RefundJob>>,
    result: Mutex<Option<String>>,
}
struct TransientGateway;
#[async_trait]
impl RefundGateway for TransientGateway {
    async fn refund(&self, _: &RefundJob) -> Result<ProviderRefund, RefundFailure> {
        Err(RefundFailure::Transient("TIMEOUT"))
    }
}
#[async_trait]
impl CancellationRepository for RecordingRepository {
    async fn cancel_booking(
        &self,
        _: Uuid,
        _: &dyn Clock,
        _: Option<&x_fly_api::domain::cancellation::StaffCancellationActor>,
    ) -> Result<Cancellation, CancellationRepositoryError> {
        unreachable!()
    }
    async fn cancel_booking_for_staff(
        &self,
        _: Uuid,
        _: &dyn Clock,
        _: &x_fly_api::domain::cancellation::StaffCancellationActor,
    ) -> Result<BookingDetail, CancellationRepositoryError> {
        unreachable!()
    }
    async fn claim_due_refund(
        &self,
        _: chrono::DateTime<Utc>,
    ) -> Result<Option<RefundJob>, sqlx::Error> {
        Ok(self.job.lock().unwrap().take())
    }
    async fn mark_refund_result(
        &self,
        _: &RefundJob,
        refund: &ProviderRefund,
        _: chrono::DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        *self.result.lock().unwrap() = Some(format!("{:?}", refund.status));
        Ok(())
    }
    async fn mark_refund_retry(
        &self,
        _: &RefundJob,
        _: chrono::DateTime<Utc>,
        _: &RefundFailure,
    ) -> Result<(), sqlx::Error> {
        *self.result.lock().unwrap() = Some("RETRY".into());
        Ok(())
    }
    async fn mark_refund_attention(
        &self,
        _: &RefundJob,
        _: &RefundFailure,
    ) -> Result<(), sqlx::Error> {
        *self.result.lock().unwrap() = Some("ATTENTION".into());
        Ok(())
    }
    async fn process_stripe_refund_event(
        &self,
        _: StripeRefundEvent,
    ) -> Result<(), CancellationRepositoryError> {
        unreachable!()
    }
}

fn job(provider: PaymentProvider) -> RefundJob {
    RefundJob {
        id: Uuid::new_v4(),
        payment_attempt_id: Uuid::new_v4(),
        provider,
        provider_payment_id: "provider-payment".into(),
        provider_refund_id: None,
        amount: Money {
            amount: 12_345,
            currency_code: "THB".into(),
        },
        attempt_count: 1,
        lease_token: Uuid::new_v4(),
    }
}

#[test]
fn every_supported_stripe_refund_status_maps_from_object_status() {
    assert_eq!(
        map_status("pending").unwrap(),
        ProviderRefundStatus::Processing
    );
    assert_eq!(
        map_status("succeeded").unwrap(),
        ProviderRefundStatus::Succeeded
    );
    for status in ["requires_action", "failed", "canceled"] {
        assert_eq!(
            map_status(status).unwrap(),
            ProviderRefundStatus::RequiresAttention
        );
    }
    assert!(map_status("invented").is_err());
}

#[test]
fn authoritative_whole_thb_is_converted_to_stripe_minor_units_once() {
    let amount = StripeAmount::from_xfly_money(&job(PaymentProvider::Stripe).amount).unwrap();
    assert_eq!(amount.amount(), 1_234_500);
}

#[tokio::test]
async fn mock_bitcoin_refund_is_deterministic_without_stripe_configuration() {
    let job = job(PaymentProvider::MockBitcoin);
    let first = MockBitcoinRefundGateway.refund(&job).await.unwrap();
    let second = MockBitcoinRefundGateway.refund(&job).await.unwrap();
    assert_eq!(first.id, second.id);
    assert_eq!(first.amount_minor, job.amount.amount);
    assert_eq!(first.status, ProviderRefundStatus::Succeeded);
}

#[tokio::test]
async fn dispatcher_routes_mock_without_stripe_and_isolates_missing_stripe_configuration() {
    for (provider, expected) in [
        (PaymentProvider::MockBitcoin, "Succeeded"),
        (PaymentProvider::Stripe, "ATTENTION"),
    ] {
        let repository = Arc::new(RecordingRepository {
            job: Mutex::new(Some(job(provider))),
            result: Mutex::new(None),
        });
        let dispatcher =
            RefundDispatcher::new(repository.clone(), None, Arc::new(MockBitcoinRefundGateway));
        assert!(dispatcher.dispatch_once(Utc::now()).await.unwrap());
        assert_eq!(repository.result.lock().unwrap().as_deref(), Some(expected));
    }
}

#[tokio::test]
async fn transient_refunds_retry_boundedly() {
    for (attempt_count, expected) in [(1, "RETRY"), (6, "ATTENTION")] {
        let mut refund_job = job(PaymentProvider::Stripe);
        refund_job.attempt_count = attempt_count;
        let repository = Arc::new(RecordingRepository {
            job: Mutex::new(Some(refund_job)),
            result: Mutex::new(None),
        });
        let dispatcher = RefundDispatcher::new(
            repository.clone(),
            Some(Arc::new(TransientGateway)),
            Arc::new(MockBitcoinRefundGateway),
        );
        dispatcher.dispatch_once(Utc::now()).await.unwrap();
        assert_eq!(repository.result.lock().unwrap().as_deref(), Some(expected));
    }
}
