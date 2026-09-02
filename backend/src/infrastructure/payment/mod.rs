use crate::domain::payment::{
    CardScenario, PaymentGateway, PaymentGatewayOutcome, PaymentGatewayRequest,
};

#[derive(Clone, Debug, Default)]
pub struct MockCardPaymentGateway;

#[async_trait::async_trait]
impl PaymentGateway for MockCardPaymentGateway {
    async fn initiate(&self, request: PaymentGatewayRequest) -> PaymentGatewayOutcome {
        let reference = format!("XFCARD-{}", request.attempt_id.simple());
        match request.scenario.unwrap_or(CardScenario::ProcessingError) {
            CardScenario::Success => PaymentGatewayOutcome::Succeeded {
                provider_reference: reference,
            },
            CardScenario::Declined => PaymentGatewayOutcome::Failed {
                provider_reference: reference,
                code: "MOCK_CARD_DECLINED",
                message: "The demo card was declined.",
            },
            CardScenario::ProcessingError => PaymentGatewayOutcome::Failed {
                provider_reference: reference,
                code: "MOCK_CARD_PROCESSING_ERROR",
                message: "The demo card processor returned an error.",
            },
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
        }
    }
}
