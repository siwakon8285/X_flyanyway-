use super::ResendPayload;
use crate::domain::{
    booking_confirmation::{DeliveryFailure, RenderedBookingConfirmationEmail},
    repositories::EmailDeliveryGateway,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

#[derive(Clone)]
pub struct ResendEmailDeliveryGateway {
    client: Client,
    api_key: String,
    from: String,
}

impl ResendEmailDeliveryGateway {
    pub fn new(api_key: String, from: String) -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()?,
            api_key,
            from,
        })
    }
}

#[async_trait]
impl EmailDeliveryGateway for ResendEmailDeliveryGateway {
    async fn send(
        &self,
        recipient: &str,
        idempotency_key: &str,
        email: &RenderedBookingConfirmationEmail,
    ) -> Result<String, DeliveryFailure> {
        let response = self
            .client
            .post("https://api.resend.com/emails")
            .bearer_auth(&self.api_key)
            .header("Idempotency-Key", idempotency_key)
            .json(&ResendPayload {
                from: &self.from,
                to: [recipient],
                subject: &email.subject,
                html: &email.html,
                text: &email.text,
            })
            .send()
            .await
            .map_err(|error| {
                if error.is_timeout() {
                    DeliveryFailure::Timeout
                } else {
                    DeliveryFailure::Connectivity
                }
            })?;
        let status = response.status();
        if !status.is_success() {
            return Err(DeliveryFailure::ProviderStatus(status.as_u16()));
        }
        let body: ResendResponse = response
            .json()
            .await
            .map_err(|_| DeliveryFailure::Connectivity)?;
        Ok(body.id)
    }
}

#[derive(Deserialize)]
struct ResendResponse {
    id: String,
}
