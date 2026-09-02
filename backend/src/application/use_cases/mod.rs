use std::{sync::Arc, time::Duration};

use uuid::Uuid;

use crate::domain::{
    entities::{CreateSeatHold, FlightSelection, SeatHold, SeatMap},
    extras::{ExtraContext, ExtraSelectionInput},
    passengers::{PassengerContext, PassengerInput},
    payment::{
        build_demo_bitcoin_invoice, CreatePaymentRequest, PaymentAttempt, PaymentContext,
        PaymentGateway, PaymentMethod, PaymentSimulationOutcome,
    },
    repositories::{
        ExtraRepository, ExtraRepositoryError, PassengerRepository, PassengerRepositoryError,
        PaymentRepository, PaymentRepositoryError, ReviewRepository, ReviewRepositoryError,
        SeatHoldRepository, SeatHoldRepositoryError,
    },
    review::ReviewContext,
    value_objects::SeatNumber,
};

mod create_payment_attempt;
mod create_seat_hold;
mod get_extras;
mod get_passengers;
mod get_payment;
mod get_review;
mod get_seat_hold;
mod get_seat_map;
mod release_seat_hold;
mod replace_seat_hold_seats;
mod save_extras;
mod save_passengers;
mod simulate_payment_attempt;
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

#[derive(Clone)]
pub struct ExtraApplication {
    repository: Arc<dyn ExtraRepository>,
}

#[derive(Clone)]
pub struct ReviewApplication {
    repository: Arc<dyn ReviewRepository>,
}

#[derive(Clone)]
pub struct PaymentApplication {
    repository: Arc<dyn PaymentRepository>,
    card_gateway: Arc<dyn PaymentGateway>,
    bitcoin_gateway: Arc<dyn PaymentGateway>,
}

impl PaymentApplication {
    pub fn new(
        repository: Arc<dyn PaymentRepository>,
        card_gateway: Arc<dyn PaymentGateway>,
        bitcoin_gateway: Arc<dyn PaymentGateway>,
    ) -> Self {
        Self {
            repository,
            card_gateway,
            bitcoin_gateway,
        }
    }

    pub async fn get_payment(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<PaymentContext, PaymentRepositoryError> {
        let mut context =
            get_payment::execute(self.repository.as_ref(), hold_id, token_hash).await?;
        context
            .attempts
            .iter_mut()
            .for_each(decorate_bitcoin_attempt);
        Ok(context)
    }

    pub async fn create_attempt(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        request: CreatePaymentRequest,
    ) -> Result<PaymentAttempt, PaymentRepositoryError> {
        let mut attempt = create_payment_attempt::execute(
            self.repository.as_ref(),
            self.card_gateway.as_ref(),
            self.bitcoin_gateway.as_ref(),
            hold_id,
            token_hash,
            request,
        )
        .await?;
        decorate_bitcoin_attempt(&mut attempt);
        Ok(attempt)
    }

    pub async fn simulate(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        attempt_id: Uuid,
        outcome: PaymentSimulationOutcome,
    ) -> Result<PaymentAttempt, PaymentRepositoryError> {
        let mut attempt = simulate_payment_attempt::execute(
            self.repository.as_ref(),
            hold_id,
            token_hash,
            attempt_id,
            outcome,
        )
        .await?;
        decorate_bitcoin_attempt(&mut attempt);
        Ok(attempt)
    }
}

fn decorate_bitcoin_attempt(attempt: &mut PaymentAttempt) {
    if attempt.payment_method != PaymentMethod::Bitcoin {
        return;
    }
    let Some(reference) = attempt.provider_reference.as_deref() else {
        return;
    };
    attempt.demo_bitcoin_invoice =
        build_demo_bitcoin_invoice(attempt.id, attempt.amount.amount, reference).ok();
}

impl ReviewApplication {
    pub fn new(repository: Arc<dyn ReviewRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_review(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<ReviewContext, ReviewRepositoryError> {
        get_review::execute(self.repository.as_ref(), hold_id, token_hash).await
    }
}

impl ExtraApplication {
    pub fn new(repository: Arc<dyn ExtraRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_extras(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<ExtraContext, ExtraRepositoryError> {
        get_extras::execute(self.repository.as_ref(), hold_id, token_hash).await
    }

    pub async fn save_extras(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        selections: Vec<ExtraSelectionInput>,
    ) -> Result<ExtraContext, ExtraRepositoryError> {
        save_extras::execute(self.repository.as_ref(), hold_id, token_hash, selections).await
    }
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
