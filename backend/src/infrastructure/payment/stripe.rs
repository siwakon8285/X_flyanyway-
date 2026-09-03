use crate::domain::extras::Money;
use crate::domain::payment::{
    PaymentFailure, PaymentGateway, PaymentGatewayOutcome, PaymentGatewayRequest,
    PaymentProviderError, PaymentProviderReconciler, PaymentProviderState,
    PaymentReconciliationStatus,
};
use serde::Deserialize;

const STRIPE_MAX_AMOUNT: i64 = 99_999_999;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StripeAmount {
    amount: i64,
}

impl StripeAmount {
    pub fn from_xfly_money(money: &Money) -> Result<Self, StripeAmountError> {
        if money.currency_code != "THB" || money.amount <= 0 {
            return Err(StripeAmountError);
        }
        let amount = money.amount.checked_mul(100).ok_or(StripeAmountError)?;
        if amount > STRIPE_MAX_AMOUNT {
            return Err(StripeAmountError);
        }
        Ok(Self { amount })
    }

    pub const fn amount(&self) -> i64 {
        self.amount
    }

    pub const fn currency(&self) -> &'static str {
        "thb"
    }

    pub fn to_xfly_money(&self) -> Result<Money, StripeAmountError> {
        if self.amount <= 0 || self.amount % 100 != 0 {
            return Err(StripeAmountError);
        }
        Ok(Money {
            amount: self.amount / 100,
            currency_code: "THB".to_owned(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StripeAmountError;

#[derive(Clone)]
pub struct StripePaymentGateway {
    client: reqwest::Client,
    secret_key: String,
}

impl StripePaymentGateway {
    pub fn new(secret_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            secret_key,
        }
    }
}

#[derive(Deserialize)]
struct PaymentIntentResponse {
    id: String,
    client_secret: Option<String>,
}

#[derive(Deserialize)]
struct PaymentIntentReconciliationResponse {
    id: String,
    status: String,
    amount: i64,
    currency: String,
    last_payment_error: Option<StripePaymentError>,
}

#[async_trait::async_trait]
impl PaymentGateway for StripePaymentGateway {
    async fn initiate(&self, request: PaymentGatewayRequest) -> PaymentGatewayOutcome {
        let amount = match StripeAmount::from_xfly_money(&request.amount) {
            Ok(value) => value,
            Err(_) => {
                return PaymentGatewayOutcome::Failed {
                    provider_reference: String::new(),
                    code: "PROCESSING_ERROR",
                    message: "The payment could not be prepared.",
                }
            }
        };
        let key = format!("x-fly-payment-intent-{}", request.attempt_id);
        let response = self
            .client
            .post("https://api.stripe.com/v1/payment_intents")
            .basic_auth(&self.secret_key, Some(""))
            .header("Idempotency-Key", key)
            .form(&[
                ("amount", amount.amount().to_string()),
                ("currency", amount.currency().to_owned()),
                ("payment_method_types[]", "card".to_owned()),
                ("metadata[x_fly_attempt_id]", request.attempt_id.to_string()),
            ])
            .send()
            .await;
        match response {
            Ok(response) if response.status().is_success() => {
                match response.json::<PaymentIntentResponse>().await {
                    Ok(intent) => PaymentGatewayOutcome::AwaitingPayment {
                        provider_reference: intent.id,
                        client_payment_session: intent.client_secret,
                    },
                    Err(_) => PaymentGatewayOutcome::Failed {
                        provider_reference: String::new(),
                        code: "PROCESSING_ERROR",
                        message: "The payment provider returned an invalid response.",
                    },
                }
            }
            _ => PaymentGatewayOutcome::Failed {
                provider_reference: String::new(),
                code: "PROCESSING_ERROR",
                message: "The payment provider is temporarily unavailable.",
            },
        }
    }
}

#[async_trait::async_trait]
impl PaymentProviderReconciler for StripePaymentGateway {
    async fn retrieve_payment_intent(
        &self,
        provider_reference: &str,
    ) -> Result<PaymentProviderState, PaymentProviderError> {
        let response = self
            .client
            .get(format!(
                "https://api.stripe.com/v1/payment_intents/{provider_reference}"
            ))
            .basic_auth(&self.secret_key, Some(""))
            .send()
            .await
            .map_err(|_| PaymentProviderError::Unavailable)?;
        if !response.status().is_success() {
            return Err(PaymentProviderError::Unavailable);
        }
        let intent = response
            .json::<PaymentIntentReconciliationResponse>()
            .await
            .map_err(|_| PaymentProviderError::InvalidResponse)?;
        reconciliation_state(intent)
    }

    async fn cancel_payment_intent(
        &self,
        provider_reference: &str,
    ) -> Result<PaymentProviderState, PaymentProviderError> {
        let response = self
            .client
            .post(format!(
                "https://api.stripe.com/v1/payment_intents/{provider_reference}/cancel"
            ))
            .basic_auth(&self.secret_key, Some(""))
            .send()
            .await
            .map_err(|_| PaymentProviderError::Unavailable)?;
        if !response.status().is_success() {
            return Err(PaymentProviderError::Unavailable);
        }
        let intent = response
            .json::<PaymentIntentReconciliationResponse>()
            .await
            .map_err(|_| PaymentProviderError::InvalidResponse)?;
        reconciliation_state(intent)
    }
}

fn reconciliation_state(
    intent: PaymentIntentReconciliationResponse,
) -> Result<PaymentProviderState, PaymentProviderError> {
    let status = match intent.status.as_str() {
        // Exact Stripe -> neutral mapping deliberately lives only here.
        "requires_payment_method" | "requires_confirmation" | "requires_action" => {
            PaymentReconciliationStatus::AwaitingCustomer
        }
        "processing" => PaymentReconciliationStatus::Processing,
        "succeeded" => PaymentReconciliationStatus::Succeeded,
        "canceled" => PaymentReconciliationStatus::Cancelled,
        // Stripe exposes a terminal error through last_payment_error. Unknown
        // statuses are not treated as terminal or releasable.
        _ if intent.last_payment_error.is_some() => PaymentReconciliationStatus::Failed,
        _ => return Err(PaymentProviderError::InvalidResponse),
    };
    let failure = matches!(status, PaymentReconciliationStatus::Failed).then(|| {
        let (code, message) = map_stripe_failure(intent.last_payment_error.as_ref());
        PaymentFailure { code, message }
    });
    Ok(PaymentProviderState {
        provider_reference: intent.id,
        status,
        amount: intent.amount,
        currency: intent.currency,
        failure,
    })
}

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

pub const STRIPE_SIGNATURE_TOLERANCE_SECONDS: i64 = 300;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum WebhookVerificationError {
    #[error("missing Stripe-Signature header")]
    MissingHeader,
    #[error("malformed Stripe-Signature header")]
    MalformedHeader,
    #[error("signature timestamp is outside tolerance")]
    TimestampOutOfTolerance,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("invalid webhook secret")]
    InvalidSecret,
}

pub fn compute_stripe_signature(
    timestamp: i64,
    raw_payload: &[u8],
    secret: &str,
) -> Result<String, WebhookVerificationError> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| WebhookVerificationError::InvalidSecret)?;
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(raw_payload);
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub fn verify_stripe_signature(
    header_value: &str,
    raw_payload: &[u8],
    secret: &str,
    current_timestamp: i64,
    tolerance_seconds: i64,
) -> Result<(), WebhookVerificationError> {
    if header_value.trim().is_empty() {
        return Err(WebhookVerificationError::MissingHeader);
    }

    let mut timestamp: Option<i64> = None;
    let mut signatures: Vec<&str> = Vec::new();

    for item in header_value.split(',') {
        let trimmed = item.trim();
        if let Some((key, val)) = trimmed.split_once('=') {
            match key.trim() {
                "t" => {
                    let ts = val
                        .trim()
                        .parse::<i64>()
                        .map_err(|_| WebhookVerificationError::MalformedHeader)?;
                    timestamp = Some(ts);
                }
                "v1" => {
                    let sig = val.trim();
                    if !sig.is_empty() {
                        signatures.push(sig);
                    }
                }
                _ => {}
            }
        }
    }

    let timestamp = timestamp.ok_or(WebhookVerificationError::MalformedHeader)?;
    if signatures.is_empty() {
        return Err(WebhookVerificationError::MalformedHeader);
    }

    if (current_timestamp - timestamp).abs() > tolerance_seconds {
        return Err(WebhookVerificationError::TimestampOutOfTolerance);
    }

    let expected_hex = compute_stripe_signature(timestamp, raw_payload, secret)?;
    let expected_bytes = expected_hex.as_bytes();

    let valid = signatures
        .iter()
        .any(|sig| sig.as_bytes().ct_eq(expected_bytes).into());

    if !valid {
        return Err(WebhookVerificationError::InvalidSignature);
    }

    Ok(())
}

#[derive(Clone, Debug, Deserialize)]
pub struct StripeWebhookEnvelope {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: StripeEventData,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StripeEventData {
    pub object: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StripePaymentIntentObject {
    pub id: String,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    pub status: Option<String>,
    pub last_payment_error: Option<StripePaymentError>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StripePaymentError {
    pub code: Option<String>,
    pub message: Option<String>,
    pub decline_code: Option<String>,
}

pub fn parse_stripe_webhook(
    raw_payload: &[u8],
) -> Result<StripeWebhookEnvelope, serde_json::Error> {
    serde_json::from_slice(raw_payload)
}

pub fn parse_payment_intent_object(
    value: &serde_json::Value,
) -> Result<StripePaymentIntentObject, serde_json::Error> {
    serde_json::from_value(value.clone())
}

pub fn map_stripe_failure(error: Option<&StripePaymentError>) -> (String, String) {
    let Some(error) = error else {
        return (
            "PROCESSING_ERROR".to_owned(),
            "The payment could not be processed.".to_owned(),
        );
    };

    let code = error.code.as_deref().unwrap_or_default();
    let decline_code = error.decline_code.as_deref().unwrap_or_default();

    if code == "card_declined" || !decline_code.is_empty() {
        (
            "CARD_DECLINED".to_owned(),
            "The payment card was declined.".to_owned(),
        )
    } else if code == "payment_intent_authentication_failure" {
        (
            "AUTHENTICATION_FAILED".to_owned(),
            "Card authentication failed.".to_owned(),
        )
    } else {
        (
            "PROCESSING_ERROR".to_owned(),
            "The payment could not be processed.".to_owned(),
        )
    }
}
