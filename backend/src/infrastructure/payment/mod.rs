use crate::domain::payment::{PaymentGateway, PaymentGatewayOutcome, PaymentGatewayRequest};

pub mod stripe;

#[derive(Clone, Debug, Default)]
pub struct UnavailableCardPaymentGateway;

#[async_trait::async_trait]
impl PaymentGateway for UnavailableCardPaymentGateway {
    async fn initiate(&self, _request: PaymentGatewayRequest) -> PaymentGatewayOutcome {
        PaymentGatewayOutcome::Failed {
            provider_reference: String::new(),
            code: "PAYMENT_CONFIGURATION_MISSING",
            message: "Card payment is unavailable.",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MockBitcoinPaymentGateway;

#[async_trait::async_trait]
impl PaymentGateway for MockBitcoinPaymentGateway {
    async fn initiate(&self, request: PaymentGatewayRequest) -> PaymentGatewayOutcome {
        PaymentGatewayOutcome::AwaitingPayment {
            provider_reference: format!("XFBTC-{}", request.attempt_id.simple()),
            client_payment_session: None,
        }
    }
}
