use async_trait::async_trait;
use chrono::{DateTime, NaiveTime, Utc};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::{
    boarding_pass::{
        derive_boarding_pass_invalid_reason, derive_check_in_state, BoardingPassDocument,
        BoardingPassInvalidReason, BoardingPassVerification, CheckInState,
    },
    flight::FlightStatus,
    repositories::{BoardingPassRepository, BoardingPassRepositoryError},
    ticket::TicketStatus,
};

use super::SqlxSeatHoldRepository;

#[async_trait]
impl BoardingPassRepository for SqlxSeatHoldRepository {
    async fn issue_boarding_pass(
        &self,
        ticket_number: &str,
        passenger_ordinal: u8,
        staff_user_id: Uuid,
    ) -> Result<BoardingPassDocument, BoardingPassRepositoryError> {
        let mut tx = self
            .pool()
            .begin()
            .await
            .map_err(BoardingPassRepositoryError::Infrastructure)?;
        let context = sqlx::query_as::<_, IssueContext>(
            "SELECT ticket.id AS ticket_id, ticket.ticket_number, ticket.booking_reference,
                    ticket.status AS ticket_status, attempt.status AS payment_status,
                    hold.id AS hold_id, hold.consumed_at,
                    instance.id AS flight_instance_id, service.status AS flight_status,
                    service.departure_time, service.origin_time_zone,
                    CASE WHEN service.departure_time IS NULL OR service.origin_time_zone IS NULL
                         THEN NULL
                         ELSE (instance.departure_date + service.departure_time)
                              AT TIME ZONE service.origin_time_zone
                    END AS departure_at
             FROM tickets AS ticket
             JOIN payment_attempts AS attempt ON attempt.id = ticket.payment_attempt_id
             JOIN seat_holds AS hold ON hold.id = attempt.seat_hold_id
             JOIN flight_instances AS instance ON instance.id = hold.flight_instance_id
             JOIN flight_services AS service ON service.id = instance.flight_service_id
             WHERE ticket.ticket_number = $1
             FOR UPDATE OF ticket, attempt, hold, instance, service",
        )
        .bind(ticket_number)
        .fetch_optional(&mut *tx)
        .await
        .map_err(BoardingPassRepositoryError::Infrastructure)?
        .ok_or(BoardingPassRepositoryError::TicketNotFound)?;

        let passenger_exists = sqlx::query_as::<_, (i16,)>(
            "SELECT ordinal
             FROM hold_passengers
             WHERE seat_hold_id = $1 AND ordinal = $2",
        )
        .bind(context.hold_id)
        .bind(i16::from(passenger_ordinal))
        .fetch_optional(&mut *tx)
        .await
        .map_err(BoardingPassRepositoryError::Infrastructure)?
        .ok_or(BoardingPassRepositoryError::PassengerNotFound)?;
        let _ = passenger_exists;

        let now: DateTime<Utc> = sqlx::query_scalar("SELECT NOW()")
            .fetch_one(&mut *tx)
            .await
            .map_err(BoardingPassRepositoryError::Infrastructure)?;
        let ticket_status = TicketStatus::parse_database(&context.ticket_status)
            .ok_or(BoardingPassRepositoryError::InconsistentState)?;
        let flight_status = FlightStatus::parse_database(&context.flight_status)
            .ok_or(BoardingPassRepositoryError::InconsistentState)?;
        let booking_valid = context.payment_status == "SUCCEEDED" && context.consumed_at.is_some();
        let seat = sqlx::query_as::<_, SeatRow>(
            "SELECT seat.seat_number, seat.cabin
             FROM payment_attempt_seats AS finalized
             JOIN flight_seats AS seat ON seat.id = finalized.flight_seat_id
             WHERE finalized.payment_attempt_id = (
                 SELECT payment_attempt_id FROM tickets WHERE id = $1
             )
               AND finalized.passenger_ordinal = $2
               AND finalized.released_at IS NULL
               AND seat.booking_status = 'BOOKED'
               AND seat.hold_id IS NULL
               AND seat.booked_at IS NOT NULL
             FOR UPDATE OF finalized, seat",
        )
        .bind(context.ticket_id)
        .bind(i16::from(passenger_ordinal))
        .fetch_optional(&mut *tx)
        .await
        .map_err(BoardingPassRepositoryError::Infrastructure)?;
        let existing: Option<Uuid> = sqlx::query_scalar(
            "SELECT id
             FROM boarding_passes
             WHERE ticket_id = $1 AND passenger_ordinal = $2",
        )
        .bind(context.ticket_id)
        .bind(i16::from(passenger_ordinal))
        .fetch_optional(&mut *tx)
        .await
        .map_err(BoardingPassRepositoryError::Infrastructure)?;

        let state = derive_check_in_state(
            now,
            context.departure_at,
            ticket_status,
            flight_status,
            booking_valid,
            seat.is_some(),
            existing.is_some(),
        );
        match state {
            CheckInState::Ready => {}
            CheckInState::CheckedIn => {
                let document = load_document(&mut tx, context.ticket_id, passenger_ordinal).await?;
                tx.commit()
                    .await
                    .map_err(BoardingPassRepositoryError::Infrastructure)?;
                return Ok(document);
            }
            CheckInState::Unavailable(reason) => {
                return Err(match reason {
                    crate::domain::boarding_pass::CheckInUnavailableReason::TooEarly => {
                        BoardingPassRepositoryError::TooEarly
                    }
                    crate::domain::boarding_pass::CheckInUnavailableReason::AlreadyDeparted => {
                        BoardingPassRepositoryError::AlreadyDeparted
                    }
                    crate::domain::boarding_pass::CheckInUnavailableReason::TicketCancelled => {
                        BoardingPassRepositoryError::TicketCancelled
                    }
                    crate::domain::boarding_pass::CheckInUnavailableReason::FlightCancelled => {
                        BoardingPassRepositoryError::FlightCancelled
                    }
                    crate::domain::boarding_pass::CheckInUnavailableReason::NoSeatAssignment => {
                        BoardingPassRepositoryError::NoSeatAssignment
                    }
                    crate::domain::boarding_pass::CheckInUnavailableReason::BookingInvalid => {
                        BoardingPassRepositoryError::BookingInvalid
                    }
                });
            }
        }

        let seat = seat.ok_or(BoardingPassRepositoryError::NoSeatAssignment)?;
        let inserted_id: Option<Uuid> = sqlx::query_scalar(
            "INSERT INTO boarding_passes (
                ticket_id, passenger_ordinal, flight_instance_id,
                seat_snapshot, cabin_snapshot, checked_in_at, issued_at,
                issued_by_staff_user_id
             ) VALUES ($1, $2, $3, $4, $5, $6, $6, $7)
             ON CONFLICT (ticket_id, passenger_ordinal) DO NOTHING
             RETURNING id",
        )
        .bind(context.ticket_id)
        .bind(i16::from(passenger_ordinal))
        .bind(context.flight_instance_id)
        .bind(&seat.seat_number)
        .bind(&seat.cabin)
        .bind(now)
        .bind(staff_user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(BoardingPassRepositoryError::Infrastructure)?;
        let (boarding_pass_id, issued_here) = match inserted_id {
            Some(id) => (id, true),
            None => {
                let id: Uuid = sqlx::query_scalar(
                    "SELECT id FROM boarding_passes
                     WHERE ticket_id = $1 AND passenger_ordinal = $2",
                )
                .bind(context.ticket_id)
                .bind(i16::from(passenger_ordinal))
                .fetch_optional(&mut *tx)
                .await
                .map_err(BoardingPassRepositoryError::Infrastructure)?
                .ok_or(BoardingPassRepositoryError::InconsistentState)?;
                (id, false)
            }
        };

        if issued_here {
            sqlx::query(
                "INSERT INTO boarding_pass_operations_audit (
                    boarding_pass_id, ticket_id, passenger_ordinal, actor_staff_user_id,
                    action, before_state, after_state
                 ) VALUES ($1, $2, $3, $4, 'BOARDING_PASS_ISSUED',
                    '{\"checkIn\":\"READY\"}'::jsonb,
                    jsonb_build_object('checkIn','CHECKED_IN','boardingPassId',$1))",
            )
            .bind(boarding_pass_id)
            .bind(context.ticket_id)
            .bind(i16::from(passenger_ordinal))
            .bind(staff_user_id)
            .execute(&mut *tx)
            .await
            .map_err(BoardingPassRepositoryError::Infrastructure)?;
        }

        let document = load_document(&mut tx, context.ticket_id, passenger_ordinal).await?;
        tx.commit()
            .await
            .map_err(BoardingPassRepositoryError::Infrastructure)?;
        Ok(document)
    }

    async fn get_boarding_pass(
        &self,
        ticket_number: &str,
        passenger_ordinal: u8,
    ) -> Result<Option<BoardingPassDocument>, BoardingPassRepositoryError> {
        let mut tx = self
            .pool()
            .begin()
            .await
            .map_err(BoardingPassRepositoryError::Infrastructure)?;
        let ticket_id: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM tickets WHERE ticket_number = $1")
                .bind(ticket_number)
                .fetch_optional(&mut *tx)
                .await
                .map_err(BoardingPassRepositoryError::Infrastructure)?;
        let Some(ticket_id) = ticket_id else {
            tx.commit()
                .await
                .map_err(BoardingPassRepositoryError::Infrastructure)?;
            return Ok(None);
        };
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM boarding_passes
                WHERE ticket_id = $1 AND passenger_ordinal = $2
             )",
        )
        .bind(ticket_id)
        .bind(i16::from(passenger_ordinal))
        .fetch_one(&mut *tx)
        .await
        .map_err(BoardingPassRepositoryError::Infrastructure)?;
        if !exists {
            tx.commit()
                .await
                .map_err(BoardingPassRepositoryError::Infrastructure)?;
            return Ok(None);
        }
        let document = load_document(&mut tx, ticket_id, passenger_ordinal).await?;
        tx.commit()
            .await
            .map_err(BoardingPassRepositoryError::Infrastructure)?;
        Ok(Some(document))
    }

    async fn verify_boarding_pass(
        &self,
        boarding_pass_id: Uuid,
    ) -> Result<Option<BoardingPassVerification>, BoardingPassRepositoryError> {
        let row = sqlx::query_as::<_, VerificationRow>(
            "SELECT boarding.id, ticket.status AS ticket_status,
                    attempt.status AS payment_status, hold.consumed_at,
                    service.status AS flight_status,
                    service.flight_number, service.origin_code, service.destination_code,
                    service.origin_time_zone,
                    CASE WHEN service.departure_time IS NULL OR service.origin_time_zone IS NULL
                         THEN NULL
                         ELSE (instance.departure_date + service.departure_time)
                              AT TIME ZONE service.origin_time_zone
                    END AS departure_at,
                    boarding.seat_snapshot, boarding.cabin_snapshot
             FROM boarding_passes AS boarding
             JOIN tickets AS ticket ON ticket.id = boarding.ticket_id
             JOIN payment_attempts AS attempt ON attempt.id = ticket.payment_attempt_id
             JOIN seat_holds AS hold ON hold.id = attempt.seat_hold_id
             JOIN flight_instances AS instance
               ON instance.id = boarding.flight_instance_id
              AND instance.id = hold.flight_instance_id
             JOIN flight_services AS service ON service.id = instance.flight_service_id
             WHERE boarding.id = $1",
        )
        .bind(boarding_pass_id)
        .fetch_optional(self.pool())
        .await
        .map_err(BoardingPassRepositoryError::Infrastructure)?;
        let Some(row) = row else {
            return Ok(None);
        };
        let now: DateTime<Utc> = sqlx::query_scalar("SELECT NOW()")
            .fetch_one(self.pool())
            .await
            .map_err(BoardingPassRepositoryError::Infrastructure)?;
        let reason = verification_reason(&row, now)?;
        Ok(Some(BoardingPassVerification {
            valid: reason.is_none(),
            boarding_pass_id: Some(row.id),
            invalid_reason: reason,
            flight_number: Some(row.flight_number),
            origin_code: Some(row.origin_code),
            destination_code: Some(row.destination_code),
            departure_at: row.departure_at,
            origin_time_zone: row.origin_time_zone,
            seat: Some(row.seat_snapshot),
            cabin: Some(row.cabin_snapshot),
        }))
    }
}

async fn load_document(
    tx: &mut Transaction<'_, Postgres>,
    ticket_id: Uuid,
    passenger_ordinal: u8,
) -> Result<BoardingPassDocument, BoardingPassRepositoryError> {
    let row = sqlx::query_as::<_, DocumentRow>(
        "SELECT boarding.id, ticket.ticket_number, ticket.booking_reference,
                boarding.passenger_ordinal, passenger.given_name, passenger.middle_name,
                passenger.family_name, service.flight_number, service.origin_code,
                service.destination_code, service.departure_time, service.origin_time_zone,
                CASE WHEN service.departure_time IS NULL OR service.origin_time_zone IS NULL
                     THEN NULL
                     ELSE (instance.departure_date + service.departure_time)
                          AT TIME ZONE service.origin_time_zone
                END AS departure_at, boarding.seat_snapshot, boarding.cabin_snapshot,
                boarding.checked_in_at, boarding.issued_at,
                ticket.status AS ticket_status, attempt.status AS payment_status,
                hold.consumed_at,
                service.status AS flight_status
         FROM boarding_passes AS boarding
         JOIN tickets AS ticket ON ticket.id = boarding.ticket_id
         JOIN payment_attempts AS attempt ON attempt.id = ticket.payment_attempt_id
         JOIN seat_holds AS hold ON hold.id = attempt.seat_hold_id
         JOIN hold_passengers AS passenger
           ON passenger.seat_hold_id = hold.id
          AND passenger.ordinal = boarding.passenger_ordinal
         JOIN flight_instances AS instance
           ON instance.id = boarding.flight_instance_id
          AND instance.id = hold.flight_instance_id
         JOIN flight_services AS service ON service.id = instance.flight_service_id
         WHERE boarding.ticket_id = $1 AND boarding.passenger_ordinal = $2",
    )
    .bind(ticket_id)
    .bind(i16::from(passenger_ordinal))
    .fetch_optional(&mut **tx)
    .await
    .map_err(BoardingPassRepositoryError::Infrastructure)?
    .ok_or(BoardingPassRepositoryError::InconsistentState)?;
    let now: DateTime<Utc> = sqlx::query_scalar("SELECT NOW()")
        .fetch_one(&mut **tx)
        .await
        .map_err(BoardingPassRepositoryError::Infrastructure)?;
    let verification_row = VerificationRow {
        id: row.id,
        ticket_status: row.ticket_status.clone(),
        payment_status: row.payment_status.clone(),
        consumed_at: row.consumed_at,
        flight_status: row.flight_status.clone(),
        flight_number: row.flight_number.clone(),
        origin_code: row.origin_code.clone(),
        destination_code: row.destination_code.clone(),
        origin_time_zone: row.origin_time_zone.clone(),
        departure_at: row.departure_at,
        seat_snapshot: row.seat_snapshot.clone(),
        cabin_snapshot: row.cabin_snapshot.clone(),
    };
    let invalid_reason = verification_reason(&verification_row, now)?;
    let passenger_name = [Some(row.given_name), row.middle_name, Some(row.family_name)]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    let departure_at = row
        .departure_at
        .ok_or(BoardingPassRepositoryError::InconsistentState)?;
    let departure_time = row
        .departure_time
        .ok_or(BoardingPassRepositoryError::InconsistentState)?
        .format("%H:%M")
        .to_string();
    let origin_time_zone = row
        .origin_time_zone
        .ok_or(BoardingPassRepositoryError::InconsistentState)?;
    Ok(BoardingPassDocument {
        boarding_pass_id: row.id,
        ticket_number: row.ticket_number,
        booking_reference: row.booking_reference,
        passenger_ordinal: u8::try_from(row.passenger_ordinal)
            .map_err(|_| BoardingPassRepositoryError::InconsistentState)?,
        passenger_name,
        flight_number: row.flight_number,
        origin_code: row.origin_code,
        destination_code: row.destination_code,
        departure_at,
        departure_time,
        origin_time_zone,
        seat: row.seat_snapshot,
        cabin: row.cabin_snapshot,
        checked_in_at: row.checked_in_at,
        issued_at: row.issued_at,
        valid_for_travel: invalid_reason.is_none(),
        invalid_reason,
    })
}

fn verification_reason(
    row: &VerificationRow,
    now: DateTime<Utc>,
) -> Result<Option<BoardingPassInvalidReason>, BoardingPassRepositoryError> {
    let ticket_status = TicketStatus::parse_database(&row.ticket_status)
        .ok_or(BoardingPassRepositoryError::InconsistentState)?;
    let flight_status = FlightStatus::parse_database(&row.flight_status)
        .ok_or(BoardingPassRepositoryError::InconsistentState)?;
    Ok(derive_boarding_pass_invalid_reason(
        now,
        row.departure_at,
        ticket_status,
        flight_status,
        row.payment_status == "SUCCEEDED" && row.consumed_at.is_some(),
    ))
}

#[derive(FromRow)]
struct IssueContext {
    ticket_id: Uuid,
    ticket_status: String,
    payment_status: String,
    hold_id: Uuid,
    consumed_at: Option<DateTime<Utc>>,
    flight_instance_id: Uuid,
    flight_status: String,
    departure_at: Option<DateTime<Utc>>,
}

#[derive(FromRow)]
struct SeatRow {
    seat_number: String,
    cabin: String,
}

#[derive(FromRow)]
struct DocumentRow {
    id: Uuid,
    ticket_number: String,
    booking_reference: String,
    passenger_ordinal: i16,
    given_name: String,
    middle_name: Option<String>,
    family_name: String,
    flight_number: String,
    origin_code: String,
    destination_code: String,
    departure_time: Option<NaiveTime>,
    origin_time_zone: Option<String>,
    departure_at: Option<DateTime<Utc>>,
    seat_snapshot: String,
    cabin_snapshot: String,
    checked_in_at: DateTime<Utc>,
    issued_at: DateTime<Utc>,
    ticket_status: String,
    payment_status: String,
    consumed_at: Option<DateTime<Utc>>,
    flight_status: String,
}

#[derive(FromRow)]
struct VerificationRow {
    id: Uuid,
    ticket_status: String,
    payment_status: String,
    consumed_at: Option<DateTime<Utc>>,
    flight_status: String,
    flight_number: String,
    origin_code: String,
    destination_code: String,
    origin_time_zone: Option<String>,
    departure_at: Option<DateTime<Utc>>,
    seat_snapshot: String,
    cabin_snapshot: String,
}
