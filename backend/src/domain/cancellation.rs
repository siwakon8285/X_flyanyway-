use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{extras::Money, payment::PaymentProvider};

pub const MAX_REFUND_ATTEMPTS: u8 = 6;

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RefundStatus {
    Pending,
    InFlight,
    Processing,
    Succeeded,
    RequiresAttention,
}

impl RefundStatus {
    pub fn parse_database(value: &str) -> Option<Self> {
        match value {
            "PENDING" => Some(Self::Pending),
            "IN_FLIGHT" => Some(Self::InFlight),
            "PROCESSING" => Some(Self::Processing),
            "SUCCEEDED" => Some(Self::Succeeded),
            "REQUIRES_ATTENTION" => Some(Self::RequiresAttention),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cancellation {
    #[serde(skip)]
    pub id: Uuid,
    pub refund_status: RefundStatus,
    pub refund_amount: Money,
    pub cancelled_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct StaffCancellationActor {
    pub staff_user_id: Uuid,
    pub email: String,
}

#[derive(Clone, Debug)]
pub struct RefundJob {
    pub id: Uuid,
    pub payment_attempt_id: Uuid,
    pub provider: PaymentProvider,
    pub provider_payment_id: String,
    pub provider_refund_id: Option<String>,
    pub amount: Money,
    pub attempt_count: u8,
    pub lease_token: Uuid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderRefundStatus {
    Processing,
    Succeeded,
    RequiresAttention,
}

#[derive(Clone, Debug)]
pub struct ProviderRefund {
    pub id: String,
    pub payment_id: String,
    pub cancellation_id: Uuid,
    pub amount_minor: i64,
    pub currency: String,
    pub status: ProviderRefundStatus,
}

#[derive(Clone, Debug)]
pub struct StripeRefundEvent {
    pub event_id: String,
    pub event_type: String,
    pub refund: ProviderRefund,
}

#[derive(Clone, Debug)]
pub enum RefundFailure {
    Transient(&'static str),
    Permanent(&'static str),
}
