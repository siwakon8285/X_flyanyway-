use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::FromRow;

use crate::domain::{
    manage_booking::BookingStatus,
    passengers::{Gender, PassengerType},
    payment::PaymentStatus,
    repositories::{TicketOperationsRepository, TicketOperationsRepositoryError},
    ticket::TicketStatus,
    ticket_operations::{
        TicketOperationsDetail, TicketOperationsFilter, TicketOperationsJourney,
        TicketOperationsListItem, TicketOperationsPage, TicketOperationsPassenger,
        TicketOperationsRecord,
    },
};

use super::SqlxSeatHoldRepository;

fn checked_pagination(
    limit: i64,
    offset: i64,
) -> Result<(usize, i64, i64), TicketOperationsRepositoryError> {
    if !(1..=50).contains(&limit) || offset < 0 {
        return Err(TicketOperationsRepositoryError::InvalidPagination);
    }
    let next_offset = offset
        .checked_add(limit)
        .ok_or(TicketOperationsRepositoryError::InvalidPagination)?;
    let fetch_limit = limit
        .checked_add(1)
        .ok_or(TicketOperationsRepositoryError::InvalidPagination)?;
    let page_size =
        usize::try_from(limit).map_err(|_| TicketOperationsRepositoryError::InvalidPagination)?;
    Ok((page_size, fetch_limit, next_offset))
}

#[async_trait]
impl TicketOperationsRepository for SqlxSeatHoldRepository {
    async fn list_tickets(
        &self,
        filter: &TicketOperationsFilter,
    ) -> Result<TicketOperationsPage, TicketOperationsRepositoryError> {
        let (page_size, fetch_limit, next_offset) =
            checked_pagination(filter.limit, filter.offset)?;
        let status = filter.ticket_status.map(|status| match status {
            TicketStatus::Issued => "ISSUED",
            TicketStatus::Cancelled => "CANCELLED",
        });
        let rows = sqlx::query_as::<_, ListRow>(
            "SELECT ticket.ticket_number,ticket.booking_reference,
                    ARRAY(SELECT concat_ws(' ',p.given_name,p.middle_name,p.family_name)
                          FROM hold_passengers p WHERE p.seat_hold_id=hold.id ORDER BY p.ordinal) passenger_names,
                    ARRAY(SELECT seat.seat_number FROM payment_attempt_seats finalized
                          JOIN flight_seats seat ON seat.id=finalized.flight_seat_id
                          WHERE finalized.payment_attempt_id=attempt.id
                          ORDER BY finalized.passenger_ordinal) seats,
                    service.flight_number,service.origin_code,service.destination_code,
                    instance.departure_date travel_date,hold.cabin,ticket.status ticket_status
             FROM tickets ticket
             JOIN payment_attempts attempt ON attempt.id=ticket.payment_attempt_id
             JOIN seat_holds hold ON hold.id=attempt.seat_hold_id
             JOIN flight_instances instance ON instance.id=hold.flight_instance_id
             JOIN flight_services service ON service.id=instance.flight_service_id
             WHERE attempt.status='SUCCEEDED'
               AND ($1::TEXT IS NULL OR ticket.ticket_number=$1)
               AND ($2::TEXT IS NULL OR ticket.booking_reference=$2)
               AND ($3::TEXT IS NULL OR EXISTS (
                    SELECT 1 FROM hold_passengers passenger_search
                    WHERE passenger_search.seat_hold_id=hold.id
                      AND lower(passenger_search.given_name || ' ' ||
                          COALESCE(passenger_search.middle_name || ' ','') ||
                          passenger_search.family_name) LIKE $3 ESCAPE '\\'))
               AND ($4::TEXT IS NULL OR service.flight_number=$4)
               AND ($5::TEXT IS NULL OR service.origin_code=$5)
               AND ($6::TEXT IS NULL OR service.destination_code=$6)
               AND ($7::DATE IS NULL OR instance.departure_date=$7)
               AND ($8::TEXT IS NULL OR ticket.status=$8)
               AND ($9::TEXT IS NULL OR hold.cabin=$9)
             ORDER BY ticket.created_at DESC,ticket.ticket_number
             LIMIT $10 OFFSET $11",
        )
        .bind(filter.ticket_number.as_deref())
        .bind(filter.booking_reference.as_deref())
        .bind(filter.passenger_name.as_deref())
        .bind(filter.flight_number.as_deref())
        .bind(filter.origin.as_deref())
        .bind(filter.destination.as_deref())
        .bind(filter.travel_date)
        .bind(status)
        .bind(filter.cabin.as_deref())
        .bind(fetch_limit)
        .bind(filter.offset)
        .fetch_all(self.pool())
        .await
        .map_err(TicketOperationsRepositoryError::Infrastructure)?;
        let has_more = rows.len() > page_size;
        let items = rows
            .into_iter()
            .take(page_size)
            .map(ListRow::domain)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(TicketOperationsPage {
            items,
            next_offset: has_more.then_some(next_offset),
        })
    }

    async fn get_ticket_operations(
        &self,
        ticket_number: &str,
    ) -> Result<Option<TicketOperationsRecord>, TicketOperationsRepositoryError> {
        let mut tx = self
            .pool()
            .begin()
            .await
            .map_err(TicketOperationsRepositoryError::Infrastructure)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(TicketOperationsRepositoryError::Infrastructure)?;
        let row = sqlx::query_as::<_, DetailRow>(
            "SELECT ticket.id ticket_id,ticket.ticket_number,ticket.booking_reference,
                    ticket.status ticket_status,ticket.issued_at,ticket.cancelled_at,
                    attempt.id payment_attempt_id,attempt.status payment_status,hold.id hold_id,
                    hold.cabin,service.flight_number,service.origin_code,service.destination_code,
                    service.status flight_status,instance.departure_date travel_date,
                    service.departure_time,service.origin_time_zone,
                    CASE WHEN service.departure_time IS NULL OR service.origin_time_zone IS NULL
                         THEN NULL ELSE (instance.departure_date+service.departure_time)
                              AT TIME ZONE service.origin_time_zone END departure_at,
                    cancellation.refund_status
             FROM tickets ticket
             JOIN payment_attempts attempt ON attempt.id=ticket.payment_attempt_id
             JOIN seat_holds hold ON hold.id=attempt.seat_hold_id
             JOIN flight_instances instance ON instance.id=hold.flight_instance_id
             JOIN flight_services service ON service.id=instance.flight_service_id
             LEFT JOIN booking_cancellations cancellation ON cancellation.ticket_id=ticket.id
             WHERE ticket.ticket_number=$1 AND attempt.status='SUCCEEDED'",
        )
        .bind(ticket_number)
        .fetch_optional(&mut *tx)
        .await
        .map_err(TicketOperationsRepositoryError::Infrastructure)?;
        let Some(row) = row else {
            tx.commit()
                .await
                .map_err(TicketOperationsRepositoryError::Infrastructure)?;
            return Ok(None);
        };
        let passengers = sqlx::query_as::<_, PassengerRow>(
            "SELECT passenger.ordinal,passenger.passenger_type,passenger.given_name,
                    passenger.middle_name,passenger.family_name,passenger.gender,seat.seat_number
             FROM hold_passengers passenger
             LEFT JOIN payment_attempt_seats finalized
               ON finalized.payment_attempt_id=$2 AND finalized.passenger_ordinal=passenger.ordinal
             LEFT JOIN flight_seats seat ON seat.id=finalized.flight_seat_id
             WHERE passenger.seat_hold_id=$1 ORDER BY passenger.ordinal",
        )
        .bind(row.hold_id)
        .bind(row.payment_attempt_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(TicketOperationsRepositoryError::Infrastructure)?;
        tx.commit()
            .await
            .map_err(TicketOperationsRepositoryError::Infrastructure)?;
        let ticket_status = parse_ticket_status(&row.ticket_status)?;
        Ok(Some(TicketOperationsRecord {
            ticket_id: row.ticket_id,
            detail: TicketOperationsDetail {
                ticket_number: row.ticket_number,
                ticket_status,
                issued_at: row.issued_at,
                cancelled_at: row.cancelled_at,
                booking_reference: row.booking_reference,
                booking_status: if ticket_status == TicketStatus::Cancelled {
                    BookingStatus::Cancelled
                } else {
                    BookingStatus::Confirmed
                },
                payment_status: PaymentStatus::parse_database(&row.payment_status)
                    .ok_or(TicketOperationsRepositoryError::InconsistentState)?,
                refund_status: row
                    .refund_status
                    .as_deref()
                    .and_then(crate::domain::cancellation::RefundStatus::parse_database),
                journey: TicketOperationsJourney {
                    flight_number: row.flight_number,
                    origin_code: row.origin_code,
                    destination_code: row.destination_code,
                    travel_date: row.travel_date,
                    departure_at: row.departure_at,
                    departure_time: row
                        .departure_time
                        .map(|time| time.format("%H:%M").to_string()),
                    origin_time_zone: row.origin_time_zone,
                    cabin: row.cabin,
                    flight_status: row.flight_status,
                },
                passengers: passengers
                    .into_iter()
                    .map(PassengerRow::domain)
                    .collect::<Result<_, _>>()?,
            },
        }))
    }
}

fn parse_ticket_status(value: &str) -> Result<TicketStatus, TicketOperationsRepositoryError> {
    TicketStatus::parse_database(value).ok_or(TicketOperationsRepositoryError::InconsistentState)
}

#[derive(FromRow)]
struct ListRow {
    ticket_number: String,
    booking_reference: String,
    passenger_names: Vec<String>,
    seats: Vec<String>,
    flight_number: String,
    origin_code: String,
    destination_code: String,
    travel_date: NaiveDate,
    cabin: String,
    ticket_status: String,
}
impl ListRow {
    fn domain(self) -> Result<TicketOperationsListItem, TicketOperationsRepositoryError> {
        Ok(TicketOperationsListItem {
            ticket_number: self.ticket_number,
            booking_reference: self.booking_reference,
            passenger_names: self.passenger_names,
            flight_number: self.flight_number,
            origin_code: self.origin_code,
            destination_code: self.destination_code,
            travel_date: self.travel_date,
            cabin: self.cabin,
            seats: self.seats,
            ticket_status: parse_ticket_status(&self.ticket_status)?,
        })
    }
}

#[derive(FromRow)]
struct DetailRow {
    ticket_id: uuid::Uuid,
    ticket_number: String,
    booking_reference: String,
    ticket_status: String,
    issued_at: DateTime<Utc>,
    cancelled_at: Option<DateTime<Utc>>,
    payment_attempt_id: uuid::Uuid,
    payment_status: String,
    hold_id: uuid::Uuid,
    cabin: String,
    flight_number: String,
    origin_code: String,
    destination_code: String,
    flight_status: String,
    travel_date: NaiveDate,
    departure_time: Option<NaiveTime>,
    origin_time_zone: Option<String>,
    departure_at: Option<DateTime<Utc>>,
    refund_status: Option<String>,
}

#[derive(FromRow)]
struct PassengerRow {
    ordinal: i16,
    passenger_type: String,
    given_name: String,
    middle_name: Option<String>,
    family_name: String,
    gender: String,
    seat_number: Option<String>,
}
impl PassengerRow {
    fn domain(self) -> Result<TicketOperationsPassenger, TicketOperationsRepositoryError> {
        Ok(TicketOperationsPassenger {
            ordinal: u8::try_from(self.ordinal)
                .map_err(|_| TicketOperationsRepositoryError::InconsistentState)?,
            display_name: [
                Some(self.given_name),
                self.middle_name,
                Some(self.family_name),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" "),
            passenger_type: PassengerType::parse_database(&self.passenger_type)
                .ok_or(TicketOperationsRepositoryError::InconsistentState)?,
            gender: Gender::parse_database(&self.gender)
                .ok_or(TicketOperationsRepositoryError::InconsistentState)?,
            seat: self.seat_number,
        })
    }
}
