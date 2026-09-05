pub mod resend;

use crate::domain::{
    booking_confirmation::{DeliveryFailure, RenderedBookingConfirmationEmail},
    repositories::EmailDeliveryGateway,
};
use async_trait::async_trait;
use serde::Serialize;

#[derive(Clone)]
pub struct DisabledEmailGateway;

#[async_trait]
impl EmailDeliveryGateway for DisabledEmailGateway {
    async fn send(
        &self,
        _recipient: &str,
        _idempotency_key: &str,
        _email: &RenderedBookingConfirmationEmail,
    ) -> Result<String, DeliveryFailure> {
        Err(DeliveryFailure::ProviderStatus(503))
    }
}

#[derive(Serialize)]
pub(crate) struct ResendPayload<'a> {
    pub from: &'a str,
    pub to: [&'a str; 1],
    pub subject: &'a str,
    pub html: &'a str,
    pub text: &'a str,
}
