use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::extras::Money;
use crate::domain::{entities::SeatHold, review::ReviewJourney};

pub const DEMO_BTC_RATE_THB_PER_BTC: i64 = 2_000_000;
const SATOSHIS_PER_BTC: i128 = 100_000_000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    Created,
    Processing,
    AwaitingPayment,
    Succeeded,
    Failed,
    Cancelled,
}

impl PaymentStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "CREATED",
            Self::Processing => "PROCESSING",
            Self::AwaitingPayment => "AWAITING_PAYMENT",
            Self::Succeeded => "SUCCEEDED",
            Self::Failed => "FAILED",
            Self::Cancelled => "CANCELLED",
        }
    }

    pub fn parse_database(value: &str) -> Option<Self> {
        match value {
            "CREATED" => Some(Self::Created),
            "PROCESSING" => Some(Self::Processing),
            "AWAITING_PAYMENT" => Some(Self::AwaitingPayment),
            "SUCCEEDED" => Some(Self::Succeeded),
            "FAILED" => Some(Self::Failed),
            "CANCELLED" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

impl PaymentStatus {
    pub fn transition(self, next: Self) -> Result<Self, PaymentTransitionError> {
        let allowed = matches!(
            (self, next),
            (
                Self::Created,
                Self::Processing | Self::AwaitingPayment | Self::Failed
            ) | (
                Self::Processing,
                Self::AwaitingPayment | Self::Succeeded | Self::Failed | Self::Cancelled
            ) | (
                Self::AwaitingPayment,
                Self::Processing | Self::Succeeded | Self::Failed | Self::Cancelled
            )
        );
        allowed
            .then_some(next)
            .ok_or(PaymentTransitionError::InvalidTransition)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentMethod {
    Card,
    Bitcoin,
}

impl PaymentMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Card => "CARD",
            Self::Bitcoin => "BITCOIN",
        }
    }

    pub fn parse_database(value: &str) -> Option<Self> {
        match value {
            "CARD" => Some(Self::Card),
            "BITCOIN" => Some(Self::Bitcoin),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentProvider {
    Stripe,
    MockBitcoin,
}

impl PaymentProvider {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stripe => "STRIPE",
            Self::MockBitcoin => "MOCK_BITCOIN",
        }
    }

    pub fn parse_database(value: &str) -> Option<Self> {
        match value {
            "STRIPE" => Some(Self::Stripe),
            "MOCK_BITCOIN" => Some(Self::MockBitcoin),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PaymentRepositoryCommand {
    pub request_id: Uuid,
    pub request_fingerprint: [u8; 32],
    pub method: PaymentMethod,
    pub provider: PaymentProvider,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentFailure {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentAttempt {
    pub id: Uuid,
    pub provider: PaymentProvider,
    pub payment_method: PaymentMethod,
    pub status: PaymentStatus,
    pub amount: Money,
    pub provider_reference: Option<String>,
    pub failure: Option<PaymentFailure>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub succeeded_at: Option<DateTime<Utc>>,
    pub payment_finalization_deadline: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_payment_session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demo_bitcoin_invoice: Option<DemoBitcoinInvoice>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoBitcoinInvoice {
    pub amount_satoshis: i64,
    pub display_amount: String,
    pub demo_address: String,
    pub invoice_reference: String,
    pub rate_thb_per_btc: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentPricing {
    pub currency_code: String,
    pub grand_total: Money,
    pub priced_at: DateTime<Utc>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentContext {
    pub hold: SeatHold,
    pub journey: ReviewJourney,
    pub pricing: PaymentPricing,
    pub methods: Vec<PaymentMethod>,
    pub attempts: Vec<PaymentAttempt>,
    pub ready_for_payment: bool,
}

#[derive(Clone, Debug)]
pub struct CreatePaymentRequest {
    pub request_id: Uuid,
    pub method: PaymentMethod,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentSimulationOutcome {
    Received,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug)]
pub struct PaymentGatewayRequest {
    pub attempt_id: Uuid,
    pub amount: Money,
}

#[derive(Clone, Debug)]
pub enum PaymentGatewayOutcome {
    Succeeded {
        provider_reference: String,
    },
    Failed {
        provider_reference: String,
        code: &'static str,
        message: &'static str,
    },
    AwaitingPayment {
        provider_reference: String,
        client_payment_session: Option<String>,
    },
}

#[async_trait::async_trait]
pub trait PaymentGateway: Send + Sync {
    async fn initiate(&self, request: PaymentGatewayRequest) -> PaymentGatewayOutcome;
}

/// Provider-neutral reconciliation categories. Stripe's raw status strings stay
/// inside infrastructure: requires_payment_method/confirmation/action map to
/// AwaitingCustomer, processing maps to Processing, succeeded maps to
/// Succeeded, and canceled or a terminal payment error map to Cancelled/Failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaymentReconciliationStatus {
    AwaitingCustomer,
    Processing,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentProviderState {
    pub provider_reference: String,
    pub status: PaymentReconciliationStatus,
    pub amount: i64,
    pub currency: String,
    pub failure: Option<PaymentFailure>,
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum PaymentProviderError {
    #[error("Stripe payment provider is temporarily unavailable")]
    Unavailable,
    #[error("Stripe returned an invalid PaymentIntent")]
    InvalidResponse,
}

#[async_trait::async_trait]
pub trait PaymentProviderReconciler: Send + Sync {
    async fn retrieve_payment_intent(
        &self,
        provider_reference: &str,
    ) -> Result<PaymentProviderState, PaymentProviderError>;

    /// A successful request is not a local terminal state: callers must use
    /// the returned state and only close the attempt after confirmed cancelled.
    async fn cancel_payment_intent(
        &self,
        provider_reference: &str,
    ) -> Result<PaymentProviderState, PaymentProviderError>;
}

#[derive(Clone, Debug)]
pub struct PaymentAttemptTransition {
    pub status: PaymentStatus,
    pub provider_reference: Option<String>,
    pub failure: Option<PaymentFailure>,
}

impl PaymentAttemptTransition {
    pub fn processing(reference: impl Into<String>) -> Self {
        Self {
            status: PaymentStatus::Processing,
            provider_reference: Some(reference.into()),
            failure: None,
        }
    }

    pub fn awaiting_payment(reference: impl Into<String>) -> Self {
        Self {
            status: PaymentStatus::AwaitingPayment,
            provider_reference: Some(reference.into()),
            failure: None,
        }
    }

    pub fn succeeded(reference: impl Into<String>) -> Self {
        Self {
            status: PaymentStatus::Succeeded,
            provider_reference: Some(reference.into()),
            failure: None,
        }
    }

    pub fn failed(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: PaymentStatus::Failed,
            provider_reference: None,
            failure: Some(PaymentFailure {
                code: code.into(),
                message: message.into(),
            }),
        }
    }

    pub fn cancelled(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: PaymentStatus::Cancelled,
            provider_reference: None,
            failure: Some(PaymentFailure {
                code: code.into(),
                message: message.into(),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum PaymentTransitionError {
    #[error("the payment status transition is invalid")]
    InvalidTransition,
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum DemoBtcConversionError {
    #[error("the authoritative amount must be positive")]
    InvalidAmount,
    #[error("the demo BTC amount overflowed")]
    Overflow,
}

pub fn demo_btc_satoshis(amount_thb: i64) -> Result<i64, DemoBtcConversionError> {
    if amount_thb <= 0 {
        return Err(DemoBtcConversionError::InvalidAmount);
    }
    let numerator = i128::from(amount_thb)
        .checked_mul(SATOSHIS_PER_BTC)
        .ok_or(DemoBtcConversionError::Overflow)?;
    let denominator = i128::from(DEMO_BTC_RATE_THB_PER_BTC);
    let rounded_up = numerator
        .checked_add(denominator - 1)
        .ok_or(DemoBtcConversionError::Overflow)?
        / denominator;
    i64::try_from(rounded_up).map_err(|_| DemoBtcConversionError::Overflow)
}

pub fn build_demo_bitcoin_invoice(
    attempt_id: Uuid,
    amount_thb: i64,
    invoice_reference: impl Into<String>,
) -> Result<DemoBitcoinInvoice, DemoBtcConversionError> {
    let amount_satoshis = demo_btc_satoshis(amount_thb)?;
    let whole = amount_satoshis / 100_000_000;
    let fraction = amount_satoshis % 100_000_000;
    let fragment = attempt_id.simple().to_string()[..12].to_ascii_uppercase();
    Ok(DemoBitcoinInvoice {
        amount_satoshis,
        display_amount: format!("{whole}.{fraction:08}"),
        demo_address: format!("DEMO-ONLY-NOT-A-BITCOIN-ADDRESS-{fragment}"),
        invoice_reference: invoice_reference.into(),
        rate_thb_per_btc: DEMO_BTC_RATE_THB_PER_BTC,
    })
}

#[derive(Clone, Debug)]
pub struct ProcessStripeWebhookCommand {
    pub event_id: String,
    pub event_type: String,
    pub payment_intent_id: String,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    pub status: Option<String>,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StripeWebhookResult {
    Processed,
    AlreadyProcessed,
    Ignored,
}
