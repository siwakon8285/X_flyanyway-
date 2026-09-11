use crate::{
    domain::{
        cancellation::{ProviderRefund, ProviderRefundStatus, RefundFailure, RefundJob},
        repositories::RefundGateway,
    },
    infrastructure::payment::stripe::StripeAmount,
};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Clone)]
pub struct StripeRefundGateway {
    client: reqwest::Client,
    key: String,
}
impl StripeRefundGateway {
    pub fn new(key: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .expect("static Stripe refund client configuration is valid"),
            key,
        }
    }
}
#[derive(Deserialize)]
struct StripeRefund {
    id: String,
    payment_intent: String,
    amount: i64,
    currency: String,
    status: String,
    metadata: std::collections::HashMap<String, String>,
}
#[derive(Deserialize)]
struct StripeRefundList {
    data: Vec<StripeRefund>,
}

pub fn parse_refund_object(value: &serde_json::Value) -> Result<ProviderRefund, RefundFailure> {
    let refund: StripeRefund = serde_json::from_value(value.clone())
        .map_err(|_| RefundFailure::Permanent("STRIPE_MALFORMED_REFUND"))?;
    let cancellation_id = refund
        .metadata
        .get("x_fly_cancellation_id")
        .and_then(|value| uuid::Uuid::parse_str(value).ok())
        .ok_or(RefundFailure::Permanent("STRIPE_REFUND_METADATA_INVALID"))?;
    Ok(ProviderRefund {
        id: refund.id,
        payment_id: refund.payment_intent,
        cancellation_id,
        amount_minor: refund.amount,
        currency: refund.currency,
        status: map_status(&refund.status)?,
    })
}
#[async_trait]
impl RefundGateway for StripeRefundGateway {
    async fn refund(&self, job: &RefundJob) -> Result<ProviderRefund, RefundFailure> {
        let minor = StripeAmount::from_xfly_money(&job.amount)
            .map_err(|_| RefundFailure::Permanent("INVALID_REFUND_AMOUNT"))?
            .amount();
        if let Some(refund_id) = &job.provider_refund_id {
            let response = self
                .client
                .get(format!("https://api.stripe.com/v1/refunds/{refund_id}"))
                .bearer_auth(&self.key)
                .send()
                .await
                .map_err(|_| RefundFailure::Transient("STRIPE_CONNECTIVITY"))?;
            return self.read_and_validate(response, job, minor).await;
        }
        let list = self
            .client
            .get("https://api.stripe.com/v1/refunds")
            .bearer_auth(&self.key)
            .query(&[
                ("payment_intent", job.provider_payment_id.as_str()),
                ("limit", "100"),
            ])
            .send()
            .await
            .map_err(|_| RefundFailure::Transient("STRIPE_CONNECTIVITY"))?;
        if list.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
            || list.status().is_server_error()
        {
            return Err(RefundFailure::Transient("STRIPE_RETRYABLE"));
        }
        if !list.status().is_success() {
            return Err(RefundFailure::Permanent("STRIPE_REJECTED"));
        }
        let existing: StripeRefundList = list
            .json()
            .await
            .map_err(|_| RefundFailure::Permanent("STRIPE_MALFORMED_RESPONSE"))?;
        if let Some(found) = existing.data.into_iter().find(|refund| {
            refund.metadata.get("x_fly_cancellation_id") == Some(&job.id.to_string())
        }) {
            return validate_refund(found, job, minor);
        }
        let amount = minor.to_string();
        let cancellation_id = job.id.to_string();
        let response = self
            .client
            .post("https://api.stripe.com/v1/refunds")
            .bearer_auth(&self.key)
            .header("Idempotency-Key", refund_idempotency_key(job))
            .form(&[
                ("payment_intent", job.provider_payment_id.as_str()),
                ("amount", amount.as_str()),
                ("metadata[x_fly_cancellation_id]", cancellation_id.as_str()),
            ])
            .send()
            .await
            .map_err(|_| RefundFailure::Transient("STRIPE_CONNECTIVITY"))?;
        self.read_and_validate(response, job, minor).await
    }
}

impl StripeRefundGateway {
    async fn read_and_validate(
        &self,
        response: reqwest::Response,
        job: &RefundJob,
        minor: i64,
    ) -> Result<ProviderRefund, RefundFailure> {
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
            || response.status().is_server_error()
        {
            return Err(RefundFailure::Transient("STRIPE_RETRYABLE"));
        }
        if !response.status().is_success() {
            return Err(RefundFailure::Permanent("STRIPE_REJECTED"));
        }
        let refund = response
            .json()
            .await
            .map_err(|_| RefundFailure::Permanent("STRIPE_MALFORMED_RESPONSE"))?;
        validate_refund(refund, job, minor)
    }
}
fn validate_refund(
    r: StripeRefund,
    job: &RefundJob,
    minor: i64,
) -> Result<ProviderRefund, RefundFailure> {
    if r.payment_intent != job.provider_payment_id
        || r.amount != minor
        || r.currency != job.amount.currency_code.to_ascii_lowercase()
        || r.metadata.get("x_fly_cancellation_id") != Some(&job.id.to_string())
    {
        return Err(RefundFailure::Permanent("STRIPE_REFUND_MISMATCH"));
    }
    Ok(ProviderRefund {
        id: r.id,
        payment_id: r.payment_intent,
        cancellation_id: job.id,
        amount_minor: r.amount,
        currency: r.currency,
        status: map_status(&r.status)?,
    })
}

fn refund_idempotency_key(job: &RefundJob) -> String {
    format!("x-fly-refund/{}", job.id)
}

pub fn map_status(status: &str) -> Result<ProviderRefundStatus, RefundFailure> {
    match status {
        "pending" => Ok(ProviderRefundStatus::Processing),
        "requires_action" | "failed" | "canceled" => Ok(ProviderRefundStatus::RequiresAttention),
        "succeeded" => Ok(ProviderRefundStatus::Succeeded),
        _ => Err(RefundFailure::Permanent("STRIPE_UNKNOWN_REFUND_STATUS")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{extras::Money, payment::PaymentProvider};

    #[test]
    fn refund_idempotency_key_is_stable_for_a_job() {
        let job = RefundJob {
            id: uuid::Uuid::new_v4(),
            payment_attempt_id: uuid::Uuid::new_v4(),
            provider: PaymentProvider::Stripe,
            provider_payment_id: "pi_test".to_owned(),
            provider_refund_id: None,
            amount: Money {
                amount: 100,
                currency_code: "THB".to_owned(),
            },
            attempt_count: 1,
            lease_token: uuid::Uuid::new_v4(),
        };
        assert_eq!(
            refund_idempotency_key(&job),
            format!("x-fly-refund/{}", job.id)
        );
        assert_eq!(refund_idempotency_key(&job), refund_idempotency_key(&job));
    }
}
