use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::value_objects::SeatNumber;
use crate::domain::{
    entities::{CreateSeatHold, FlightSelection, SeatHold, SeatMap},
    extras::{ExtraContext, ExtraSelectionInput, ExtraValidationError},
    manage_booking::{ManageBookingLookup, ManageBookingRecord},
    passengers::{PassengerContext, PassengerFieldError, PassengerInput},
    payment::{PaymentAttempt, PaymentAttemptTransition, PaymentContext, PaymentRepositoryCommand},
    review::ReviewContext,
    ticket::{Ticket, TicketVerification},
};

#[derive(Debug, Error)]
pub enum ManageBookingRepositoryError {
    #[error("database operation failed")]
    Infrastructure(#[source] sqlx::Error),
    #[error("authoritative booking state is inconsistent")]
    InconsistentState,
}

#[async_trait]
pub trait ManageBookingRepository: Send + Sync {
    async fn lookup_manage_booking(
        &self,
        lookup: &ManageBookingLookup,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<ManageBookingRecord>, ManageBookingRepositoryError>;

    async fn get_manage_booking(
        &self,
        ticket_id: Uuid,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<ManageBookingRecord>, ManageBookingRepositoryError>;

    async fn get_manage_booking_ticket(
        &self,
        ticket_id: Uuid,
    ) -> Result<Option<Ticket>, ManageBookingRepositoryError>;
}

#[derive(Debug, Error)]
pub enum TicketRepositoryError {
    #[error("seat hold not found")]
    HoldNotFound,
    #[error("seat hold authorization failed")]
    Unauthorized,
    #[error("payment attempt not found")]
    PaymentNotFound,
    #[error("payment has not succeeded")]
    PaymentIncomplete,
    #[error("payment finalization state is inconsistent")]
    FinalizationInconsistent,
    #[error("ticket identity generation failed")]
    IdentityGeneration,
    #[error("database operation failed")]
    Infrastructure(#[source] sqlx::Error),
}

#[async_trait]
pub trait TicketRepository: Send + Sync {
    async fn issue_ticket(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        payment_attempt_id: Uuid,
    ) -> Result<Ticket, TicketRepositoryError>;

    async fn verify_ticket(
        &self,
        ticket_id: Uuid,
    ) -> Result<Option<TicketVerification>, TicketRepositoryError>;
}

#[derive(Debug, Error)]
pub enum PaymentRepositoryError {
    #[error("seat hold not found")]
    HoldNotFound,
    #[error("seat hold authorization failed")]
    Unauthorized,
    #[error("seat hold has expired")]
    HoldExpired,
    #[error("seat hold has been released")]
    HoldReleased,
    #[error("seat hold has already been consumed")]
    HoldConsumed,
    #[error("payment finalization is in progress")]
    PaymentFinalizationInProgress,
    #[error("held seats are not ready for payment")]
    SeatsNotReady,
    #[error("passenger information is not ready for payment")]
    PassengersNotReady,
    #[error("travel extras are not ready for payment")]
    ExtrasNotReady,
    #[error("review snapshot is not ready for payment")]
    ReviewNotReady,
    #[error("a payment has already succeeded")]
    AlreadySucceeded,
    #[error("another payment attempt is in progress")]
    AttemptInProgress,
    #[error("payment attempt not found")]
    AttemptNotFound,
    #[error("payment transition is invalid")]
    InvalidTransition,
    #[error("idempotency key was reused with a different request")]
    IdempotencyKeyReused,
    #[error("payment request is invalid")]
    InvalidRequest,
    #[error("payment amount or currency mismatch")]
    AmountMismatch,
    #[error("database operation failed")]
    Infrastructure(#[source] sqlx::Error),
}

#[async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn get_payment(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<PaymentContext, PaymentRepositoryError>;

    async fn get_payment_attempt(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        attempt_id: Uuid,
    ) -> Result<PaymentAttempt, PaymentRepositoryError>;

    async fn create_payment_attempt(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        command: PaymentRepositoryCommand,
    ) -> Result<PaymentAttempt, PaymentRepositoryError>;

    async fn transition_payment_attempt(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        attempt_id: Uuid,
        transition: PaymentAttemptTransition,
    ) -> Result<PaymentAttempt, PaymentRepositoryError>;

    async fn process_stripe_webhook(
        &self,
        command: crate::domain::payment::ProcessStripeWebhookCommand,
    ) -> Result<crate::domain::payment::StripeWebhookResult, PaymentRepositoryError>;
}

#[derive(Debug, Error)]
pub enum ExtraRepositoryError {
    #[error("seat hold not found")]
    HoldNotFound,
    #[error("seat hold authorization failed")]
    Unauthorized,
    #[error("seat hold has expired")]
    HoldExpired,
    #[error("seat hold has been released")]
    HoldReleased,
    #[error("seat hold has already been consumed")]
    HoldConsumed,
    #[error("payment finalization is in progress")]
    PaymentFinalizationInProgress,
    #[error("held seat count no longer matches the passenger party")]
    SeatCountMismatch,
    #[error("passenger information must be completed before extras")]
    PassengersNotReady,
    #[error("extra product code is unknown")]
    UnknownProduct,
    #[error("extra quantity is invalid")]
    InvalidQuantity,
    #[error("passenger ordinal does not belong to the active hold")]
    InvalidPassenger,
    #[error("passenger is not eligible for the selected extra")]
    PassengerIneligible,
    #[error("passenger has conflicting selections in one category")]
    CategoryConflict,
    #[error("database operation failed")]
    Infrastructure(#[source] sqlx::Error),
}

impl From<ExtraValidationError> for ExtraRepositoryError {
    fn from(error: ExtraValidationError) -> Self {
        match error {
            ExtraValidationError::UnknownProduct => Self::UnknownProduct,
            ExtraValidationError::InvalidQuantity => Self::InvalidQuantity,
            ExtraValidationError::InvalidPassenger => Self::InvalidPassenger,
            ExtraValidationError::PassengerIneligible => Self::PassengerIneligible,
            ExtraValidationError::CategoryConflict => Self::CategoryConflict,
        }
    }
}

#[async_trait]
pub trait ExtraRepository: Send + Sync {
    async fn get_extras(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<ExtraContext, ExtraRepositoryError>;

    async fn save_extras(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        selections: Vec<ExtraSelectionInput>,
    ) -> Result<ExtraContext, ExtraRepositoryError>;
}

#[derive(Debug, Error)]
pub enum SeatHoldRepositoryError {
    #[error("flight not found")]
    FlightNotFound,
    #[error("cabin is not available for this flight")]
    CabinUnavailable,
    #[error("one or more seats do not exist in the selected inventory")]
    SeatNotFound(Vec<String>),
    #[error("the number of requested seats does not match the passenger party")]
    SeatCountMismatch,
    #[error("one or more seats are no longer available")]
    SeatConflict(Vec<String>),
    #[error("seat hold not found")]
    HoldNotFound,
    #[error("seat hold authorization failed")]
    Unauthorized,
    #[error("seat hold has expired")]
    HoldExpired,
    #[error("seat hold has been released")]
    HoldReleased,
    #[error("seat hold has already been consumed")]
    HoldConsumed,
    #[error("payment finalization is in progress")]
    PaymentFinalizationInProgress,
    #[error("database operation failed")]
    Infrastructure(#[source] sqlx::Error),
}

#[async_trait]
pub trait SeatHoldRepository: Send + Sync {
    async fn seat_map(
        &self,
        selection: &FlightSelection,
        owner: Option<(Uuid, [u8; 32])>,
    ) -> Result<SeatMap, SeatHoldRepositoryError>;

    async fn create_hold(
        &self,
        command: CreateSeatHold,
        ttl: Duration,
    ) -> Result<SeatHold, SeatHoldRepositoryError>;

    async fn replace_seats(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        seats: Vec<SeatNumber>,
    ) -> Result<SeatHold, SeatHoldRepositoryError>;

    async fn get_hold(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<SeatHold, SeatHoldRepositoryError>;

    async fn release_hold(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<(), SeatHoldRepositoryError>;
}

#[derive(Debug, Error)]
pub enum PassengerRepositoryError {
    #[error("seat hold not found")]
    HoldNotFound,
    #[error("seat hold authorization failed")]
    Unauthorized,
    #[error("seat hold has expired")]
    HoldExpired,
    #[error("seat hold has been released")]
    HoldReleased,
    #[error("seat hold has already been consumed")]
    HoldConsumed,
    #[error("payment finalization is in progress")]
    PaymentFinalizationInProgress,
    #[error("held seat count no longer matches the passenger party")]
    SeatCountMismatch,
    #[error("passenger count does not match the active hold")]
    CountMismatch,
    #[error("passenger types do not match the active hold")]
    TypeMismatch,
    #[error("passenger fields are invalid")]
    Validation(Vec<PassengerFieldError>),
    #[error("database operation failed")]
    Infrastructure(#[source] sqlx::Error),
}

#[async_trait]
pub trait PassengerRepository: Send + Sync {
    async fn get_passengers(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<PassengerContext, PassengerRepositoryError>;

    async fn save_passengers(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        passengers: Vec<PassengerInput>,
    ) -> Result<PassengerContext, PassengerRepositoryError>;
}

#[derive(Debug, Error)]
pub enum ReviewRepositoryError {
    #[error("seat hold not found")]
    HoldNotFound,
    #[error("seat hold authorization failed")]
    Unauthorized,
    #[error("seat hold has expired")]
    HoldExpired,
    #[error("seat hold has been released")]
    HoldReleased,
    #[error("seat hold has already been consumed")]
    HoldConsumed,
    #[error("held seats are not ready for review")]
    SeatsNotReady,
    #[error("passenger information is not ready for review")]
    PassengersNotReady,
    #[error("travel extras have not been explicitly saved")]
    ExtrasNotReady,
    #[error("authoritative review pricing is unavailable")]
    PricingUnavailable,
    #[error("database operation failed")]
    Infrastructure(#[source] sqlx::Error),
}

#[async_trait]
pub trait ReviewRepository: Send + Sync {
    async fn get_review(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<ReviewContext, ReviewRepositoryError>;
}
