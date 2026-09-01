use std::{sync::Arc, time::Duration};

use crate::{
    application::use_cases::{ExtraApplication, PassengerApplication, SeatHoldApplication},
    domain::repositories::{ExtraRepository, PassengerRepository, SeatHoldRepository},
};

#[derive(Clone)]
pub struct AppState {
    pub extras: ExtraApplication,
    pub passengers: PassengerApplication,
    pub seat_holds: SeatHoldApplication,
    pub secure_cookies: bool,
    pub frontend_origin: String,
}

impl AppState {
    pub fn new(
        repository: Arc<dyn SeatHoldRepository>,
        passenger_repository: Arc<dyn PassengerRepository>,
        extra_repository: Arc<dyn ExtraRepository>,
        hold_ttl: Duration,
        secure_cookies: bool,
        frontend_origin: String,
    ) -> Self {
        Self {
            extras: ExtraApplication::new(extra_repository),
            passengers: PassengerApplication::new(passenger_repository),
            seat_holds: SeatHoldApplication::new(repository, hold_ttl),
            secure_cookies,
            frontend_origin,
        }
    }
}
