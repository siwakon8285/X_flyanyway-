use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::{
    passengers::{
        expected_passenger_slots, BookingContact, BookingContactInput, EmergencyContact, Gender,
        Passenger, PassengerContext, PassengerDraft, PassengerDraftError, PassengerInput,
        PassengerType, Title,
    },
    repositories::{PassengerRepository, PassengerRepositoryError, SeatHoldRepositoryError},
    value_objects::PassengerCounts,
};

use super::SqlxSeatHoldRepository;

#[async_trait]
impl PassengerRepository for SqlxSeatHoldRepository {
    async fn get_passengers(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<PassengerContext, PassengerRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(PassengerRepositoryError::Infrastructure)?;
        let hold_row = Self::locked_hold(&mut transaction, hold_id, token_hash)
            .await
            .map_err(map_hold_error)?;
        let hold = Self::hold_entity(&mut transaction, hold_row)
            .await
            .map_err(map_hold_error)?;
        let passengers = load_passengers(&mut transaction, hold_id).await?;
        let booking_contact = load_booking_contact(&mut transaction, hold_id).await?;
        let expected_passengers = expected_passenger_slots(hold.passengers);
        let ready_to_continue = hold.seats.len() == hold.passengers.required_seats()
            && passengers.len() == expected_passengers.len()
            && passengers
                .iter()
                .zip(&expected_passengers)
                .all(|(passenger, expected)| {
                    passenger.ordinal == expected.ordinal
                        && passenger.passenger_type == expected.passenger_type
                });
        transaction
            .commit()
            .await
            .map_err(PassengerRepositoryError::Infrastructure)?;
        Ok(PassengerContext {
            hold,
            expected_passengers,
            passengers,
            booking_contact,
            ready_to_continue,
        })
    }

    async fn save_passengers(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        inputs: Vec<PassengerInput>,
    ) -> Result<PassengerContext, PassengerRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(PassengerRepositoryError::Infrastructure)?;
        let hold_row = Self::locked_hold(&mut transaction, hold_id, token_hash)
            .await
            .map_err(map_hold_error)?;
        Self::ensure_no_protected_payment(&mut transaction, hold_id)
            .await
            .map_err(map_hold_error)?;
        let counts = PassengerCounts::new(
            hold_row.adults as u8,
            hold_row.children as u8,
            hold_row.infants as u8,
        )
        .expect("database passenger constraints are valid");
        let server_date = Self::server_time(&mut transaction)
            .await
            .map_err(map_hold_error)?
            .date_naive();
        let draft = PassengerDraft::validate(inputs, counts, hold_row.departure_date, server_date)
            .map_err(map_draft_error)?;
        let hold = Self::hold_entity(&mut transaction, hold_row)
            .await
            .map_err(map_hold_error)?;
        if hold.seats.len() != counts.required_seats() {
            return Err(PassengerRepositoryError::SeatCountMismatch);
        }

        sqlx::query("DELETE FROM hold_passengers WHERE seat_hold_id = $1")
            .bind(hold_id)
            .execute(&mut *transaction)
            .await
            .map_err(PassengerRepositoryError::Infrastructure)?;
        let passengers = draft.into_passengers();
        for passenger in &passengers {
            let emergency = passenger.emergency_contact.as_ref();
            sqlx::query(
                "INSERT INTO hold_passengers (
                    seat_hold_id, ordinal, passenger_type, title, given_name, middle_name,
                    family_name, date_of_birth, gender, nationality_code, passport_number,
                    passport_issuing_country_code, passport_issue_date, passport_expiry_date,
                    email, phone_country_code, phone_number, emergency_contact_name,
                    emergency_contact_relationship, emergency_contact_phone_country_code,
                    emergency_contact_phone_number
                 ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                    $15, $16, $17, $18, $19, $20, $21
                 )",
            )
            .bind(hold_id)
            .bind(i16::from(passenger.ordinal))
            .bind(passenger.passenger_type.as_str())
            .bind(passenger.title.as_str())
            .bind(&passenger.given_name)
            .bind(&passenger.middle_name)
            .bind(&passenger.family_name)
            .bind(passenger.date_of_birth)
            .bind(passenger.gender.as_str())
            .bind(&passenger.nationality_code)
            .bind(&passenger.passport_number)
            .bind(&passenger.passport_issuing_country_code)
            .bind(Option::<NaiveDate>::None)
            .bind(Option::<NaiveDate>::None)
            .bind(&passenger.email)
            .bind(&passenger.phone_country_code)
            .bind(&passenger.phone_number)
            .bind(emergency.map(|contact| &contact.name))
            .bind(emergency.map(|contact| &contact.relationship))
            .bind(emergency.map(|contact| &contact.phone_country_code))
            .bind(emergency.map(|contact| &contact.phone_number))
            .execute(&mut *transaction)
            .await
            .map_err(PassengerRepositoryError::Infrastructure)?;
        }

        sqlx::query("DELETE FROM hold_review_pricing WHERE seat_hold_id = $1")
            .bind(hold_id)
            .execute(&mut *transaction)
            .await
            .map_err(PassengerRepositoryError::Infrastructure)?;

        let expected_passengers = expected_passenger_slots(counts);
        let booking_contact = load_booking_contact(&mut transaction, hold_id).await?;
        transaction
            .commit()
            .await
            .map_err(PassengerRepositoryError::Infrastructure)?;
        Ok(PassengerContext {
            hold,
            expected_passengers,
            passengers,
            booking_contact,
            ready_to_continue: true,
        })
    }

    async fn save_booking_contact(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        contact: BookingContactInput,
    ) -> Result<PassengerContext, PassengerRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(PassengerRepositoryError::Infrastructure)?;
        let hold_row = Self::locked_hold(&mut transaction, hold_id, token_hash)
            .await
            .map_err(map_hold_error)?;
        Self::ensure_no_protected_payment(&mut transaction, hold_id)
            .await
            .map_err(map_hold_error)?;
        let email = contact.email.trim();
        let locale = contact.preferred_locale.trim().to_ascii_uppercase();
        if email.is_empty() || !email.contains('@') || !matches!(locale.as_str(), "EN" | "TH") {
            return Err(PassengerRepositoryError::Validation(Vec::new()));
        }
        sqlx::query(
            "INSERT INTO booking_contacts (seat_hold_id, email, preferred_locale)
             VALUES ($1, $2, $3)
             ON CONFLICT (seat_hold_id) DO UPDATE SET email = EXCLUDED.email,
                preferred_locale = EXCLUDED.preferred_locale, updated_at = NOW()",
        )
        .bind(hold_id)
        .bind(email)
        .bind(locale)
        .execute(&mut *transaction)
        .await
        .map_err(PassengerRepositoryError::Infrastructure)?;
        let hold = Self::hold_entity(&mut transaction, hold_row)
            .await
            .map_err(map_hold_error)?;
        let passengers = load_passengers(&mut transaction, hold_id).await?;
        let context = PassengerContext {
            expected_passengers: expected_passenger_slots(hold.passengers),
            hold,
            passengers,
            booking_contact: load_booking_contact(&mut transaction, hold_id).await?,
            ready_to_continue: true,
        };
        transaction
            .commit()
            .await
            .map_err(PassengerRepositoryError::Infrastructure)?;
        Ok(context)
    }
}

async fn load_booking_contact(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
) -> Result<Option<BookingContact>, PassengerRepositoryError> {
    let row = sqlx::query_as::<_, BookingContactRow>(
        "SELECT email, preferred_locale FROM booking_contacts WHERE seat_hold_id = $1",
    )
    .bind(hold_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(PassengerRepositoryError::Infrastructure)?;
    Ok(row.map(|value| BookingContact {
        email: value.email,
        preferred_locale: value.preferred_locale,
    }))
}

pub(super) async fn load_passengers(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
) -> Result<Vec<Passenger>, PassengerRepositoryError> {
    let rows = sqlx::query_as::<_, PassengerRow>(
        "SELECT
            ordinal, passenger_type, title, given_name, middle_name, family_name,
            date_of_birth, gender, nationality_code, passport_number,
            passport_issuing_country_code, email, phone_country_code, phone_number,
            emergency_contact_name,
            emergency_contact_relationship, emergency_contact_phone_country_code,
            emergency_contact_phone_number
         FROM hold_passengers
         WHERE seat_hold_id = $1
         ORDER BY ordinal",
    )
    .bind(hold_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(PassengerRepositoryError::Infrastructure)?;

    Ok(rows
        .into_iter()
        .map(|row| Passenger {
            ordinal: row.ordinal as u8,
            passenger_type: PassengerType::parse_database(&row.passenger_type)
                .expect("database passenger type constraint is valid"),
            title: Title::parse_database(&row.title).expect("database title constraint is valid"),
            given_name: row.given_name,
            middle_name: row.middle_name,
            family_name: row.family_name,
            date_of_birth: row.date_of_birth,
            gender: Gender::parse_database(&row.gender)
                .expect("database gender constraint is valid"),
            nationality_code: row.nationality_code,
            passport_number: row.passport_number,
            passport_issuing_country_code: row.passport_issuing_country_code,
            email: row.email,
            phone_country_code: row.phone_country_code,
            phone_number: row.phone_number,
            emergency_contact: row.emergency_contact_name.map(|name| EmergencyContact {
                name,
                relationship: row
                    .emergency_contact_relationship
                    .expect("database emergency contact constraint is valid"),
                phone_country_code: row
                    .emergency_contact_phone_country_code
                    .expect("database emergency contact constraint is valid"),
                phone_number: row
                    .emergency_contact_phone_number
                    .expect("database emergency contact constraint is valid"),
            }),
        })
        .collect())
}

fn map_hold_error(error: SeatHoldRepositoryError) -> PassengerRepositoryError {
    match error {
        SeatHoldRepositoryError::HoldNotFound => PassengerRepositoryError::HoldNotFound,
        SeatHoldRepositoryError::Unauthorized => PassengerRepositoryError::Unauthorized,
        SeatHoldRepositoryError::HoldExpired => PassengerRepositoryError::HoldExpired,
        SeatHoldRepositoryError::HoldReleased => PassengerRepositoryError::HoldReleased,
        SeatHoldRepositoryError::HoldConsumed => PassengerRepositoryError::HoldConsumed,
        SeatHoldRepositoryError::PaymentFinalizationInProgress => {
            PassengerRepositoryError::PaymentFinalizationInProgress
        }
        SeatHoldRepositoryError::SeatCountMismatch => PassengerRepositoryError::SeatCountMismatch,
        SeatHoldRepositoryError::Infrastructure(error) => {
            PassengerRepositoryError::Infrastructure(error)
        }
        SeatHoldRepositoryError::FlightNotFound
        | SeatHoldRepositoryError::CabinUnavailable
        | SeatHoldRepositoryError::SeatNotFound(_)
        | SeatHoldRepositoryError::SeatConflict(_) => PassengerRepositoryError::SeatCountMismatch,
    }
}

fn map_draft_error(error: PassengerDraftError) -> PassengerRepositoryError {
    match error {
        PassengerDraftError::CountMismatch => PassengerRepositoryError::CountMismatch,
        PassengerDraftError::TypeMismatch => PassengerRepositoryError::TypeMismatch,
        PassengerDraftError::Fields(errors) => PassengerRepositoryError::Validation(errors),
    }
}

#[derive(FromRow)]
struct PassengerRow {
    ordinal: i16,
    passenger_type: String,
    title: String,
    given_name: String,
    middle_name: Option<String>,
    family_name: String,
    date_of_birth: NaiveDate,
    gender: String,
    nationality_code: String,
    passport_number: String,
    passport_issuing_country_code: String,
    email: String,
    phone_country_code: String,
    phone_number: String,
    emergency_contact_name: Option<String>,
    emergency_contact_relationship: Option<String>,
    emergency_contact_phone_country_code: Option<String>,
    emergency_contact_phone_number: Option<String>,
}

#[derive(FromRow)]
struct BookingContactRow {
    email: String,
    preferred_locale: String,
}
