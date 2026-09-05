use async_trait::async_trait;
use chrono::{DateTime, Days, NaiveDate, NaiveTime, Utc};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::{
    extras::Money,
    manage_booking::{
        derive_travel_eligibility, BookingStatus, ManageBooking, ManageBookingCancellation,
        ManageBookingExtra, ManageBookingJourney, ManageBookingLookup, ManageBookingPassenger,
        ManageBookingPayment, ManageBookingRecord, ManageBookingTicket, TravelDocumentStatus,
    },
    payment::PaymentStatus,
    repositories::{ManageBookingRepository, ManageBookingRepositoryError, TicketRepositoryError},
    ticket::{Ticket, TicketStatus},
};

use super::{extras::load_selections, ticket::load_ticket_by_attempt, SqlxSeatHoldRepository};

#[async_trait]
impl ManageBookingRepository for SqlxSeatHoldRepository {
    async fn lookup_manage_booking(
        &self,
        lookup: &ManageBookingLookup,
        now: DateTime<Utc>,
    ) -> Result<Option<ManageBookingRecord>, ManageBookingRepositoryError> {
        let ticket_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT ticket.id
             FROM tickets AS ticket
             JOIN payment_attempts AS attempt ON attempt.id = ticket.payment_attempt_id
             WHERE ticket.booking_reference = $1
               AND EXISTS (
                    SELECT 1 FROM hold_passengers AS passenger
                    WHERE passenger.seat_hold_id = attempt.seat_hold_id
                      AND lower(passenger.family_name) = $2
               )",
        )
        .bind(lookup.booking_reference())
        .bind(lookup.family_name())
        .fetch_optional(self.pool())
        .await
        .map_err(ManageBookingRepositoryError::Infrastructure)?;
        match ticket_id {
            Some(ticket_id) => self.get_manage_booking(ticket_id, now).await,
            None => Ok(None),
        }
    }

    async fn get_manage_booking(
        &self,
        ticket_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Option<ManageBookingRecord>, ManageBookingRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(ManageBookingRepositoryError::Infrastructure)?;
        let record = load_manage_booking(&mut transaction, ticket_id, now).await?;
        transaction
            .commit()
            .await
            .map_err(ManageBookingRepositoryError::Infrastructure)?;
        Ok(record)
    }

    async fn get_manage_booking_ticket(
        &self,
        ticket_id: Uuid,
    ) -> Result<Option<Ticket>, ManageBookingRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(ManageBookingRepositoryError::Infrastructure)?;
        let attempt_id =
            sqlx::query_scalar::<_, Uuid>("SELECT payment_attempt_id FROM tickets WHERE id = $1")
                .bind(ticket_id)
                .fetch_optional(&mut *transaction)
                .await
                .map_err(ManageBookingRepositoryError::Infrastructure)?;
        let ticket = match attempt_id {
            Some(attempt_id) => load_ticket_by_attempt(&mut transaction, attempt_id)
                .await
                .map_err(map_ticket_error)?,
            None => None,
        };
        transaction
            .commit()
            .await
            .map_err(ManageBookingRepositoryError::Infrastructure)?;
        Ok(ticket)
    }
}

async fn load_manage_booking(
    transaction: &mut Transaction<'_, Postgres>,
    ticket_id: Uuid,
    now: DateTime<Utc>,
) -> Result<Option<ManageBookingRecord>, ManageBookingRepositoryError> {
    let row = sqlx::query_as::<_, BookingRow>(
        "SELECT ticket.id AS ticket_id, ticket.booking_reference, ticket.ticket_number,
                ticket.status AS ticket_status, ticket.issued_at,
                attempt.status AS payment_status, attempt.amount, attempt.currency_code,
                attempt.id AS payment_attempt_id, hold.id AS hold_id, hold.cabin,
                hold.adults, hold.children, hold.consumed_at,
                service.flight_number, service.origin_code, service.destination_code,
                instance.departure_date, service.departure_time, service.origin_time_zone,
                CASE WHEN service.departure_time IS NULL OR service.origin_time_zone IS NULL
                     THEN NULL
                     ELSE (instance.departure_date + service.departure_time)
                          AT TIME ZONE service.origin_time_zone
                END AS departure_at,
                service.arrival_time, service.arrival_day_offset
         FROM tickets AS ticket
         JOIN payment_attempts AS attempt ON attempt.id = ticket.payment_attempt_id
         JOIN seat_holds AS hold ON hold.id = attempt.seat_hold_id
         JOIN flight_instances AS instance ON instance.id = hold.flight_instance_id
         JOIN flight_services AS service ON service.id = instance.flight_service_id
         WHERE ticket.id = $1 AND attempt.status = 'SUCCEEDED'",
    )
    .bind(ticket_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(ManageBookingRepositoryError::Infrastructure)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let ticket_status = TicketStatus::parse_database(&row.ticket_status)
        .expect("ticket status constraint is valid");
    let payment_status = PaymentStatus::parse_database(&row.payment_status)
        .expect("payment status constraint is valid");
    let passengers = sqlx::query_as::<_, PassengerRow>(
        "SELECT ordinal, given_name, middle_name, family_name,
                passport_number <> '' AND passport_issuing_country_code <> '' AS document_complete
         FROM hold_passengers WHERE seat_hold_id = $1 ORDER BY ordinal",
    )
    .bind(row.hold_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(ManageBookingRepositoryError::Infrastructure)?;
    let seats = sqlx::query_scalar::<_, String>(
        "SELECT seat.seat_number FROM payment_attempt_seats AS finalized
         JOIN flight_seats AS seat ON seat.id = finalized.flight_seat_id
         WHERE finalized.payment_attempt_id = $1
           AND seat.booking_status = 'BOOKED' AND seat.hold_id IS NULL
           AND seat.booked_at IS NOT NULL
         ORDER BY seat.seat_number",
    )
    .bind(row.payment_attempt_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(ManageBookingRepositoryError::Infrastructure)?;
    let expected_seat_count = usize::try_from(row.adults + row.children)
        .map_err(|_| ManageBookingRepositoryError::InconsistentState)?;
    if row.consumed_at.is_none() || seats.len() != expected_seat_count {
        return Err(ManageBookingRepositoryError::InconsistentState);
    }
    let extras = load_selections(transaction, row.hold_id)
        .await
        .map_err(|error| match error {
            crate::domain::repositories::ExtraRepositoryError::Infrastructure(error) => {
                ManageBookingRepositoryError::Infrastructure(error)
            }
            _ => unreachable!("loading persisted extras has no state errors"),
        })?;
    let eligibility = derive_travel_eligibility(row.departure_at, ticket_status, now);
    let arrival_date = row.arrival_day_offset.and_then(|offset| {
        u64::try_from(offset)
            .ok()
            .and_then(|days| row.departure_date.checked_add_days(Days::new(days)))
    });
    Ok(Some(ManageBookingRecord {
        ticket_id: row.ticket_id,
        booking: ManageBooking {
            booking_reference: row.booking_reference,
            status: match ticket_status {
                TicketStatus::Issued => BookingStatus::Confirmed,
                TicketStatus::Cancelled => BookingStatus::Cancelled,
            },
            journey: ManageBookingJourney {
                flight_number: row.flight_number,
                origin_code: row.origin_code,
                destination_code: row.destination_code,
                departure_date: row.departure_date,
                departure_time: format_time(row.departure_time),
                departure_time_zone: row.origin_time_zone,
                departure_at: row.departure_at,
                arrival_date,
                arrival_time: format_time(row.arrival_time),
                cabin: row.cabin,
            },
            passengers: passengers
                .into_iter()
                .map(PassengerRow::into_domain)
                .collect(),
            seats,
            extras: extras
                .into_iter()
                .map(|extra| ManageBookingExtra {
                    passenger_ordinal: extra.passenger_ordinal,
                    product_code: extra.product_code,
                    category: extra.category,
                    quantity: extra.quantity,
                })
                .collect(),
            payment: ManageBookingPayment {
                status: payment_status,
                amount: Money {
                    amount: row.amount,
                    currency_code: row.currency_code,
                },
            },
            ticket: ManageBookingTicket {
                ticket_number: row.ticket_number,
                status: ticket_status,
                issued_at: row.issued_at,
            },
            cancellation: ManageBookingCancellation {
                eligibility: eligibility.cancellation,
                cutoff_at: eligibility.cancellation_cutoff_at,
            },
        },
    }))
}

fn format_time(value: Option<NaiveTime>) -> Option<String> {
    value.map(|time| time.format("%H:%M").to_string())
}

fn map_ticket_error(error: TicketRepositoryError) -> ManageBookingRepositoryError {
    match error {
        TicketRepositoryError::Infrastructure(error) => {
            ManageBookingRepositoryError::Infrastructure(error)
        }
        _ => unreachable!("loading an existing ticket has no issuance state errors"),
    }
}

#[derive(FromRow)]
struct BookingRow {
    ticket_id: Uuid,
    booking_reference: String,
    ticket_number: String,
    ticket_status: String,
    issued_at: DateTime<Utc>,
    payment_status: String,
    amount: i64,
    currency_code: String,
    payment_attempt_id: Uuid,
    hold_id: Uuid,
    cabin: String,
    adults: i16,
    children: i16,
    consumed_at: Option<DateTime<Utc>>,
    flight_number: String,
    origin_code: String,
    destination_code: String,
    departure_date: NaiveDate,
    departure_time: Option<NaiveTime>,
    origin_time_zone: Option<String>,
    departure_at: Option<DateTime<Utc>>,
    arrival_time: Option<NaiveTime>,
    arrival_day_offset: Option<i16>,
}

#[derive(FromRow)]
struct PassengerRow {
    ordinal: i16,
    given_name: String,
    middle_name: Option<String>,
    family_name: String,
    document_complete: bool,
}

impl PassengerRow {
    fn into_domain(self) -> ManageBookingPassenger {
        ManageBookingPassenger {
            ordinal: self.ordinal as u8,
            display_name: [
                Some(self.given_name),
                self.middle_name,
                Some(self.family_name),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" "),
            travel_document_status: if self.document_complete {
                TravelDocumentStatus::Complete
            } else {
                TravelDocumentStatus::Incomplete
            },
        }
    }
}
