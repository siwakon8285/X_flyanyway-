use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::value_objects::SeatNumber;
use crate::domain::{
    entities::{CreateSeatHold, FlightSelection, SeatHold, SeatMap},
    passengers::{PassengerContext, PassengerFieldError, PassengerInput},
};

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
