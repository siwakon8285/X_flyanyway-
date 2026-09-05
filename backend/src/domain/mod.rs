pub mod entities;
pub mod extras;
pub mod manage_booking;
pub mod passengers;
pub mod payment;
pub mod pricing;
pub mod repositories;
pub mod review;
pub mod ticket;
pub mod value_objects;

use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("at least one adult is required")]
    AdultRequired,
    #[error("lap infants cannot outnumber adults")]
    TooManyInfants,
    #[error("the passenger count exceeds the supported booking size")]
    PartyTooLarge,
    #[error("invalid seat number")]
    InvalidSeatNumber,
    #[error("invalid cabin")]
    InvalidCabin,
}
