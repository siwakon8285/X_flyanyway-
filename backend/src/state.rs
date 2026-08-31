use std::{sync::Arc, time::Duration};

use crate::{
    application::use_cases::SeatHoldApplication, domain::repositories::SeatHoldRepository,
};

#[derive(Clone)]
pub struct AppState {
    pub seat_holds: SeatHoldApplication,
    pub secure_cookies: bool,
    pub frontend_origin: String,
}

impl AppState {
    pub fn new(
        repository: Arc<dyn SeatHoldRepository>,
        hold_ttl: Duration,
        secure_cookies: bool,
        frontend_origin: String,
    ) -> Self {
        Self {
            seat_holds: SeatHoldApplication::new(repository, hold_ttl),
            secure_cookies,
            frontend_origin,
        }
    }
}
