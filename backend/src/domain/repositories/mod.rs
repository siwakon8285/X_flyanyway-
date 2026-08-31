use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::entities::{CreateSeatHold, FlightSelection, SeatHold, SeatMap};
use crate::domain::value_objects::SeatNumber;

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
