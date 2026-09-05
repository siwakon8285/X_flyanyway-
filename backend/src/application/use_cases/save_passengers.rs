use uuid::Uuid;

use crate::domain::{
    passengers::{BookingContactInput, PassengerContext, PassengerInput},
    repositories::{PassengerRepository, PassengerRepositoryError},
};

pub async fn execute(
    repository: &dyn PassengerRepository,
    hold_id: Uuid,
    token_hash: [u8; 32],
    passengers: Vec<PassengerInput>,
    booking_contact: Option<BookingContactInput>,
) -> Result<PassengerContext, PassengerRepositoryError> {
    let mut context = repository
        .save_passengers(hold_id, token_hash, passengers)
        .await?;
    if let Some(contact) = booking_contact {
        context = repository
            .save_booking_contact(hold_id, token_hash, contact)
            .await?;
    }
    Ok(context)
}
