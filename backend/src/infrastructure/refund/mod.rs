use crate::domain::{
    cancellation::{ProviderRefund, ProviderRefundStatus, RefundFailure, RefundJob},
    repositories::RefundGateway,
};
use async_trait::async_trait;

pub mod stripe;

#[derive(Clone, Default)]
pub struct MockBitcoinRefundGateway;
#[async_trait]
impl RefundGateway for MockBitcoinRefundGateway {
    async fn refund(&self, job: &RefundJob) -> Result<ProviderRefund, RefundFailure> {
        Ok(ProviderRefund {
            id: format!("mock-refund-{}", job.id),
            payment_id: job.provider_payment_id.clone(),
            cancellation_id: job.id,
            amount_minor: job.amount.amount,
            currency: job.amount.currency_code.to_ascii_lowercase(),
            status: ProviderRefundStatus::Succeeded,
        })
    }
}
