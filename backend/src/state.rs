use std::{sync::Arc, time::Duration};

use crate::{
    application::use_cases::{PassengerApplication, SeatHoldApplication},
    domain::repositories::{PassengerRepository, SeatHoldRepository},
};

#[derive(Clone)]
pub struct AppState {
    pub passengers: PassengerApplication,
    pub seat_holds: SeatHoldApplication,
    pub secure_cookies: bool,
    pub frontend_origin: String,
}

impl AppState {
    pub fn new(
        repository: Arc<dyn SeatHoldRepository>,
        passenger_repository: Arc<dyn PassengerRepository>,
        hold_ttl: Duration,
        secure_cookies: bool,
        frontend_origin: String,
    ) -> Self {
        Self {
            passengers: PassengerApplication::new(passenger_repository),
            seat_holds: SeatHoldApplication::new(repository, hold_ttl),
            secure_cookies,
            frontend_origin,
        }
    }
}
