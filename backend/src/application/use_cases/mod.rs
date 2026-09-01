use std::{sync::Arc, time::Duration};

use uuid::Uuid;

use crate::domain::{
    entities::{CreateSeatHold, FlightSelection, SeatHold, SeatMap},
    passengers::{PassengerContext, PassengerInput},
    repositories::{
        PassengerRepository, PassengerRepositoryError, SeatHoldRepository, SeatHoldRepositoryError,
    },
    value_objects::SeatNumber,
};

mod create_seat_hold;
mod get_passengers;
mod get_seat_hold;
mod get_seat_map;
mod release_seat_hold;
mod replace_seat_hold_seats;
mod save_passengers;
mod validate_seat_hold;

#[derive(Clone)]
pub struct SeatHoldApplication {
    repository: Arc<dyn SeatHoldRepository>,
    hold_ttl: Duration,
}

#[derive(Clone)]
pub struct PassengerApplication {
    repository: Arc<dyn PassengerRepository>,
}

impl PassengerApplication {
    pub fn new(repository: Arc<dyn PassengerRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_passengers(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<PassengerContext, PassengerRepositoryError> {
        get_passengers::execute(self.repository.as_ref(), hold_id, token_hash).await
    }

    pub async fn save_passengers(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        passengers: Vec<PassengerInput>,
    ) -> Result<PassengerContext, PassengerRepositoryError> {
        save_passengers::execute(self.repository.as_ref(), hold_id, token_hash, passengers).await
    }
}

impl SeatHoldApplication {
    pub fn new(repository: Arc<dyn SeatHoldRepository>, hold_ttl: Duration) -> Self {
        Self {
            repository,
            hold_ttl,
        }
    }

    pub async fn seat_map(
        &self,
        selection: &FlightSelection,
        owner: Option<(Uuid, [u8; 32])>,
    ) -> Result<SeatMap, SeatHoldRepositoryError> {
        get_seat_map::execute(self.repository.as_ref(), selection, owner).await
    }

    pub async fn create_hold(
        &self,
        command: CreateSeatHold,
    ) -> Result<SeatHold, SeatHoldRepositoryError> {
        create_seat_hold::execute(self.repository.as_ref(), command, self.hold_ttl).await
    }

    pub async fn replace_seats(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        seats: Vec<SeatNumber>,
    ) -> Result<SeatHold, SeatHoldRepositoryError> {
        replace_seat_hold_seats::execute(self.repository.as_ref(), hold_id, token_hash, seats).await
    }

    pub async fn get_hold(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<SeatHold, SeatHoldRepositoryError> {
        get_seat_hold::execute(self.repository.as_ref(), hold_id, token_hash).await
    }

    pub async fn validate_hold_for_continue(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<SeatHold, SeatHoldRepositoryError> {
        validate_seat_hold::execute(self.repository.as_ref(), hold_id, token_hash).await
    }

    pub async fn release_hold(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<(), SeatHoldRepositoryError> {
        release_seat_hold::execute(self.repository.as_ref(), hold_id, token_hash).await
    }
}
