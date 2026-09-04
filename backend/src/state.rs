use std::{sync::Arc, time::Duration};

use crate::{
    application::use_cases::{
        ExtraApplication, PassengerApplication, PaymentApplication, ReviewApplication,
        SeatHoldApplication, TicketApplication,
    },
    domain::repositories::{
        ExtraRepository, PassengerRepository, ReviewRepository, SeatHoldRepository,
        TicketRepository,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub extras: ExtraApplication,
    pub passengers: PassengerApplication,
    pub payments: Option<PaymentApplication>,
    pub reviews: ReviewApplication,
    pub seat_holds: SeatHoldApplication,
    pub tickets: Option<TicketApplication>,
    pub secure_cookies: bool,
    pub frontend_origin: String,
    pub stripe_webhook_secret: Option<String>,
}

impl AppState {
    pub fn new(
        repository: Arc<dyn SeatHoldRepository>,
        passenger_repository: Arc<dyn PassengerRepository>,
        extra_repository: Arc<dyn ExtraRepository>,
        review_repository: Arc<dyn ReviewRepository>,
        hold_ttl: Duration,
        secure_cookies: bool,
        frontend_origin: String,
    ) -> Self {
        Self {
            extras: ExtraApplication::new(extra_repository),
            passengers: PassengerApplication::new(passenger_repository),
            payments: None,
            reviews: ReviewApplication::new(review_repository),
            seat_holds: SeatHoldApplication::new(repository, hold_ttl),
            tickets: None,
            secure_cookies,
            frontend_origin,
            stripe_webhook_secret: None,
        }
    }

    pub fn with_payments(mut self, payments: PaymentApplication) -> Self {
        self.payments = Some(payments);
        self
    }

    pub fn with_stripe_webhook_secret(mut self, secret: Option<String>) -> Self {
        self.stripe_webhook_secret = secret;
        self
    }

    pub fn with_tickets(
        mut self,
        repository: Arc<dyn TicketRepository>,
        qr_signing_secret: String,
    ) -> Self {
        self.tickets = Some(TicketApplication::new(repository, qr_signing_secret));
        self
    }
}
