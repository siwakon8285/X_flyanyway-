use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rand::RngCore;
use sqlx::{FromRow, Postgres, Transaction};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::domain::{
    repositories::{TicketRepository, TicketRepositoryError},
    ticket::{Ticket, TicketJourney, TicketPassenger, TicketStatus, TicketVerification},
};

use super::SqlxSeatHoldRepository;

const REFERENCE_ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

#[async_trait]
impl TicketRepository for SqlxSeatHoldRepository {
    async fn issue_ticket(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        payment_attempt_id: Uuid,
    ) -> Result<Ticket, TicketRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(TicketRepositoryError::Infrastructure)?;
        let hold = sqlx::query_as::<_, TicketHoldRow>(
            "SELECT access_token_hash, consumed_at, adults, children
             FROM seat_holds WHERE id = $1 FOR UPDATE",
        )
        .bind(hold_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(TicketRepositoryError::Infrastructure)?
        .ok_or(TicketRepositoryError::HoldNotFound)?;
        if hold.access_token_hash.ct_eq(&token_hash).unwrap_u8() != 1 {
            return Err(TicketRepositoryError::Unauthorized);
        }
        let attempt = sqlx::query_as::<_, TicketAttemptRow>(
            "SELECT status FROM payment_attempts WHERE id = $1 AND seat_hold_id = $2 FOR UPDATE",
        )
        .bind(payment_attempt_id)
        .bind(hold_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(TicketRepositoryError::Infrastructure)?
        .ok_or(TicketRepositoryError::PaymentNotFound)?;
        if attempt.status != "SUCCEEDED" {
            return Err(TicketRepositoryError::PaymentIncomplete);
        }
        if hold.consumed_at.is_none()
            || !finalized_seats_are_consistent(
                &mut transaction,
                payment_attempt_id,
                hold.adults + hold.children,
            )
            .await?
        {
            return Err(TicketRepositoryError::FinalizationInconsistent);
        }
        if let Some(ticket) = load_ticket_by_attempt(&mut transaction, payment_attempt_id).await? {
            transaction
                .commit()
                .await
                .map_err(TicketRepositoryError::Infrastructure)?;
            return Ok(ticket);
        }
        let mut ticket_id = None;
        for _ in 0..8 {
            let booking_reference = random_identity("XF", 8);
            let ticket_number = random_identity("XFT", 12);
            match sqlx::query_scalar::<_, Uuid>(
                "INSERT INTO tickets (payment_attempt_id, booking_reference, ticket_number, status)
                 VALUES ($1, $2, $3, 'ISSUED')
                 ON CONFLICT (payment_attempt_id) DO NOTHING
                 RETURNING id",
            )
            .bind(payment_attempt_id)
            .bind(booking_reference)
            .bind(ticket_number)
            .fetch_optional(&mut *transaction)
            .await
            {
                Ok(Some(id)) => {
                    ticket_id = Some(id);
                    break;
                }
                Ok(None) => break,
                Err(error)
                    if error
                        .as_database_error()
                        .and_then(|database| database.code())
                        .is_some_and(|code| code == "23505") =>
                {
                    continue
                }
                Err(error) => return Err(TicketRepositoryError::Infrastructure(error)),
            }
        }
        if ticket_id.is_none()
            && load_ticket_by_attempt(&mut transaction, payment_attempt_id)
                .await?
                .is_none()
        {
            return Err(TicketRepositoryError::IdentityGeneration);
        }
        let ticket = load_ticket_by_attempt(&mut transaction, payment_attempt_id)
            .await?
            .ok_or(TicketRepositoryError::IdentityGeneration)?;
        transaction
            .commit()
            .await
            .map_err(TicketRepositoryError::Infrastructure)?;
        Ok(ticket)
    }

    async fn verify_ticket(
        &self,
        ticket_id: Uuid,
    ) -> Result<Option<TicketVerification>, TicketRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(TicketRepositoryError::Infrastructure)?;
        let row = sqlx::query_as::<_, VerificationRow>(
            "SELECT ticket.status, service.flight_number, service.origin_code, service.destination_code,
                instance.departure_date, service.departure_time
             FROM tickets AS ticket
             JOIN payment_attempts AS attempt ON attempt.id = ticket.payment_attempt_id
             JOIN seat_holds AS hold ON hold.id = attempt.seat_hold_id
             JOIN flight_instances AS instance ON instance.id = hold.flight_instance_id
             JOIN flight_services AS service ON service.id = instance.flight_service_id
             WHERE ticket.id = $1",
        )
        .bind(ticket_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(TicketRepositoryError::Infrastructure)?;
        let Some(row) = row else {
            transaction
                .commit()
                .await
                .map_err(TicketRepositoryError::Infrastructure)?;
            return Ok(None);
        };
        let seats: Vec<String> = sqlx::query_scalar(
            "SELECT seat.seat_number FROM payment_attempt_seats AS finalized
             JOIN flight_seats AS seat ON seat.id = finalized.flight_seat_id
             JOIN tickets AS ticket ON ticket.payment_attempt_id = finalized.payment_attempt_id
             WHERE ticket.id = $1 ORDER BY seat.seat_number",
        )
        .bind(ticket_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(TicketRepositoryError::Infrastructure)?;
        transaction
            .commit()
            .await
            .map_err(TicketRepositoryError::Infrastructure)?;
        Ok(Some(TicketVerification {
            valid: true,
            ticket_status: TicketStatus::parse_database(&row.status),
            flight_number: Some(row.flight_number),
            origin_code: Some(row.origin_code),
            destination_code: Some(row.destination_code),
            departure_date: Some(row.departure_date),
            departure_time: row
                .departure_time
                .map(|time| time.format("%H:%M").to_string()),
            seats: Some(seats),
        }))
    }
}

async fn finalized_seats_are_consistent(
    transaction: &mut Transaction<'_, Postgres>,
    attempt_id: Uuid,
    expected: i16,
) -> Result<bool, TicketRepositoryError> {
    let rows = sqlx::query_as::<_, FinalizedSeatRow>(
        "SELECT seat.booking_status, seat.hold_id, seat.booked_at
         FROM payment_attempt_seats AS finalized
         JOIN flight_seats AS seat ON seat.id = finalized.flight_seat_id
         WHERE finalized.payment_attempt_id = $1 FOR UPDATE OF seat",
    )
    .bind(attempt_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(TicketRepositoryError::Infrastructure)?;
    Ok(rows.len() == expected as usize
        && rows.iter().all(|row| {
            row.booking_status == "BOOKED" && row.hold_id.is_none() && row.booked_at.is_some()
        }))
}

pub(super) async fn load_ticket_by_attempt(
    transaction: &mut Transaction<'_, Postgres>,
    attempt_id: Uuid,
) -> Result<Option<Ticket>, TicketRepositoryError> {
    let row = sqlx::query_as::<_, TicketRow>(
        "SELECT ticket.id, ticket.booking_reference, ticket.ticket_number, ticket.status, ticket.issued_at,
            attempt.status AS payment_status, attempt.amount, attempt.currency_code, service.flight_number,
            service.origin_code, service.destination_code, hold.id AS hold_id, instance.departure_date, service.departure_time,
            service.arrival_time, service.arrival_day_offset, hold.cabin
         FROM tickets AS ticket JOIN payment_attempts AS attempt ON attempt.id = ticket.payment_attempt_id
         JOIN seat_holds AS hold ON hold.id = attempt.seat_hold_id JOIN flight_instances AS instance ON instance.id = hold.flight_instance_id
         JOIN flight_services AS service ON service.id = instance.flight_service_id WHERE attempt.id = $1",
    )
    .bind(attempt_id).fetch_optional(&mut **transaction).await.map_err(TicketRepositoryError::Infrastructure)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let passengers = sqlx::query_as::<_, PassengerRow>("SELECT given_name, middle_name, family_name FROM hold_passengers WHERE seat_hold_id = $1 ORDER BY ordinal")
        .bind(row.hold_id).fetch_all(&mut **transaction).await.map_err(TicketRepositoryError::Infrastructure)?;
    let seats: Vec<String> = sqlx::query_scalar("SELECT seat.seat_number FROM payment_attempt_seats AS finalized JOIN flight_seats AS seat ON seat.id = finalized.flight_seat_id WHERE finalized.payment_attempt_id = $1 ORDER BY seat.seat_number")
        .bind(attempt_id).fetch_all(&mut **transaction).await.map_err(TicketRepositoryError::Infrastructure)?;
    Ok(Some(Ticket {
        id: row.id,
        booking_reference: row.booking_reference,
        ticket_number: row.ticket_number,
        status: TicketStatus::parse_database(&row.status)
            .expect("ticket status constraint is valid"),
        issued_at: row.issued_at,
        payment_status: row.payment_status,
        amount: row.amount,
        currency_code: row.currency_code,
        journey: TicketJourney {
            flight_number: row.flight_number,
            origin_code: row.origin_code,
            destination_code: row.destination_code,
            departure_date: row.departure_date,
            departure_time: row
                .departure_time
                .map(|value| value.format("%H:%M").to_string()),
            arrival_time: row
                .arrival_time
                .map(|value| value.format("%H:%M").to_string()),
            arrival_day_offset: row
                .arrival_day_offset
                .and_then(|value| u8::try_from(value).ok()),
            cabin: row.cabin,
        },
        passengers: passengers
            .into_iter()
            .map(PassengerRow::into_domain)
            .collect(),
        seats,
    }))
}

fn random_identity(prefix: &str, suffix_length: usize) -> String {
    let mut bytes = vec![0_u8; suffix_length];
    rand::rng().fill_bytes(&mut bytes);
    let suffix: String = bytes
        .into_iter()
        .map(|byte| REFERENCE_ALPHABET[(byte & 31) as usize] as char)
        .collect();
    format!("{prefix}{suffix}")
}

#[derive(FromRow)]
struct TicketHoldRow {
    access_token_hash: Vec<u8>,
    consumed_at: Option<DateTime<Utc>>,
    adults: i16,
    children: i16,
}
#[derive(FromRow)]
struct TicketAttemptRow {
    status: String,
}
#[derive(FromRow)]
struct FinalizedSeatRow {
    booking_status: String,
    hold_id: Option<Uuid>,
    booked_at: Option<DateTime<Utc>>,
}
#[derive(FromRow)]
struct VerificationRow {
    status: String,
    flight_number: String,
    origin_code: String,
    destination_code: String,
    departure_date: NaiveDate,
    departure_time: Option<NaiveTime>,
}
#[derive(FromRow)]
struct PassengerRow {
    given_name: String,
    middle_name: Option<String>,
    family_name: String,
}
impl PassengerRow {
    fn into_domain(self) -> TicketPassenger {
        TicketPassenger {
            display_name: [
                Some(self.given_name),
                self.middle_name,
                Some(self.family_name),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" "),
        }
    }
}
#[derive(FromRow)]
struct TicketRow {
    id: Uuid,
    booking_reference: String,
    ticket_number: String,
    status: String,
    issued_at: DateTime<Utc>,
    payment_status: String,
    amount: i64,
    currency_code: String,
    flight_number: String,
    origin_code: String,
    destination_code: String,
    hold_id: Uuid,
    departure_date: NaiveDate,
    departure_time: Option<NaiveTime>,
    arrival_time: Option<NaiveTime>,
    arrival_day_offset: Option<i16>,
    cabin: String,
}
