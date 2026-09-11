use async_trait::async_trait;
use chrono::{DateTime, Days, NaiveDate, NaiveTime, Utc};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::{
    booking_management::{
        BookingAuditEntry, BookingCancellationSummary, BookingContactSummary, BookingDetail,
        BookingJourney, BookingListFilter, BookingListItem, BookingListPage, BookingPassenger,
        BookingPaymentSummary, BookingTicketSummary,
    },
    extras::Money,
    manage_booking::{derive_travel_eligibility, BookingStatus},
    passengers::{Gender, PassengerType},
    payment::{PaymentMethod, PaymentProvider, PaymentStatus},
    repositories::{BookingManagementRepository, BookingManagementRepositoryError},
    ticket::TicketStatus,
};

use super::SqlxSeatHoldRepository;

fn checked_pagination(
    limit: i64,
    offset: i64,
) -> Result<(usize, i64, i64), BookingManagementRepositoryError> {
    if !(1..=50).contains(&limit) || offset < 0 {
        return Err(BookingManagementRepositoryError::InvalidPagination);
    }
    let next_offset = offset
        .checked_add(limit)
        .ok_or(BookingManagementRepositoryError::InvalidPagination)?;
    let fetch_limit = limit
        .checked_add(1)
        .ok_or(BookingManagementRepositoryError::InvalidPagination)?;
    let page_size =
        usize::try_from(limit).map_err(|_| BookingManagementRepositoryError::InvalidPagination)?;
    Ok((page_size, fetch_limit, next_offset))
}

#[async_trait]
impl BookingManagementRepository for SqlxSeatHoldRepository {
    async fn list_bookings(
        &self,
        filter: &BookingListFilter,
    ) -> Result<BookingListPage, BookingManagementRepositoryError> {
        let (page_size, fetch_limit, next_offset) =
            checked_pagination(filter.limit, filter.offset)?;
        let booking_status = filter.booking_status.map(|status| match status {
            BookingStatus::Confirmed => "CONFIRMED",
            BookingStatus::Cancelled => "CANCELLED",
        });
        let rows = sqlx::query_as::<_, BookingListRow>(
            "SELECT ticket.booking_reference,
                    concat_ws(' ',lead.given_name,lead.middle_name,lead.family_name) lead_passenger_name,
                    (SELECT COUNT(*) FROM hold_passengers passenger_count
                     WHERE passenger_count.seat_hold_id=hold.id) passenger_count,
                    service.flight_number,service.origin_code,service.destination_code,
                    instance.departure_date travel_date,
                    CASE WHEN service.departure_time IS NULL OR service.origin_time_zone IS NULL
                         THEN NULL ELSE (instance.departure_date+service.departure_time)
                              AT TIME ZONE service.origin_time_zone END departure_at,
                    hold.cabin,ticket.status ticket_status,service.status flight_status,
                    attempt.status payment_status,cancellation.refund_status,ticket.created_at booked_at
             FROM tickets ticket
             JOIN payment_attempts attempt ON attempt.id=ticket.payment_attempt_id
             JOIN seat_holds hold ON hold.id=attempt.seat_hold_id
             JOIN flight_instances instance ON instance.id=hold.flight_instance_id
             JOIN flight_services service ON service.id=instance.flight_service_id
             JOIN LATERAL (
                 SELECT given_name,middle_name,family_name FROM hold_passengers
                 WHERE seat_hold_id=hold.id ORDER BY ordinal LIMIT 1
             ) lead ON TRUE
             LEFT JOIN booking_cancellations cancellation ON cancellation.ticket_id=ticket.id
             WHERE attempt.status='SUCCEEDED'
               AND ($1::TEXT IS NULL OR ticket.booking_reference=$1)
               AND ($2::TEXT IS NULL OR EXISTS (
                    SELECT 1 FROM hold_passengers passenger_search
                    WHERE passenger_search.seat_hold_id=hold.id
                      AND lower(passenger_search.given_name || ' ' ||
                                COALESCE(passenger_search.middle_name || ' ','') ||
                                passenger_search.family_name)
                          LIKE $2 ESCAPE '\\'))
               AND ($3::TEXT IS NULL OR service.flight_number=$3)
               AND ($4::DATE IS NULL OR instance.departure_date=$4)
               AND ($5::TEXT IS NULL OR
                    CASE WHEN ticket.status='CANCELLED' THEN 'CANCELLED' ELSE 'CONFIRMED' END=$5)
               AND ($6::TEXT IS NULL OR hold.cabin=$6)
               AND ($7::TEXT IS NULL OR service.origin_code=$7)
               AND ($8::TEXT IS NULL OR service.destination_code=$8)
             ORDER BY ticket.created_at DESC,ticket.booking_reference
             LIMIT $9 OFFSET $10",
        )
        .bind(filter.booking_reference.as_deref())
        .bind(filter.passenger_name.as_deref())
        .bind(filter.flight_number.as_deref())
        .bind(filter.travel_date)
        .bind(booking_status)
        .bind(filter.cabin.as_deref())
        .bind(filter.origin.as_deref())
        .bind(filter.destination.as_deref())
        .bind(fetch_limit)
        .bind(filter.offset)
        .fetch_all(self.pool())
        .await
        .map_err(BookingManagementRepositoryError::Infrastructure)?;

        let has_more = rows.len() > page_size;
        let items = rows
            .into_iter()
            .take(page_size)
            .map(BookingListRow::domain)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(BookingListPage {
            next_offset: has_more.then_some(next_offset),
            items,
        })
    }

    async fn get_booking_detail(
        &self,
        booking_reference: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<BookingDetail>, BookingManagementRepositoryError> {
        let mut tx = self
            .pool()
            .begin()
            .await
            .map_err(BookingManagementRepositoryError::Infrastructure)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(BookingManagementRepositoryError::Infrastructure)?;
        let detail = load_booking_detail(&mut tx, booking_reference, now).await?;
        tx.commit()
            .await
            .map_err(BookingManagementRepositoryError::Infrastructure)?;
        Ok(detail)
    }

    async fn ticket_id_for_booking_reference(
        &self,
        booking_reference: &str,
    ) -> Result<Option<Uuid>, BookingManagementRepositoryError> {
        sqlx::query_scalar("SELECT id FROM tickets WHERE booking_reference=$1")
            .bind(booking_reference)
            .fetch_optional(self.pool())
            .await
            .map_err(BookingManagementRepositoryError::Infrastructure)
    }
}

pub(super) async fn load_booking_detail(
    tx: &mut Transaction<'_, Postgres>,
    booking_reference: &str,
    now: DateTime<Utc>,
) -> Result<Option<BookingDetail>, BookingManagementRepositoryError> {
    let row = sqlx::query_as::<_, BookingDetailRow>(
        "SELECT ticket.id ticket_id,ticket.booking_reference,ticket.ticket_number,
                    ticket.status ticket_status,
                    ticket.issued_at,ticket.cancelled_at ticket_cancelled_at,
                    ticket.created_at booking_created_at,
                    attempt.id payment_attempt_id,attempt.provider,attempt.payment_method,
                    attempt.status payment_status,attempt.amount,attempt.currency_code,
                    attempt.succeeded_at,hold.id hold_id,hold.cabin,
                    service.flight_number,service.origin_code,service.destination_code,
                    service.aircraft_code,service.status flight_status,
                    instance.departure_date travel_date,service.departure_time,
                    service.arrival_time,service.arrival_day_offset,service.origin_time_zone,
                    CASE WHEN service.departure_time IS NULL OR service.origin_time_zone IS NULL
                         THEN NULL ELSE (instance.departure_date+service.departure_time)
                              AT TIME ZONE service.origin_time_zone END departure_at,
                    COALESCE(contact.phone_country_code,lead.phone_country_code) phone_country_code,
                    COALESCE(contact.phone_number,lead.phone_number) phone_number,
                    cancellation.refund_status,cancellation.refund_amount,
                    cancellation.cancelled_at,cancellation.refunded_at
             FROM tickets ticket
             JOIN payment_attempts attempt ON attempt.id=ticket.payment_attempt_id
             JOIN seat_holds hold ON hold.id=attempt.seat_hold_id
             JOIN flight_instances instance ON instance.id=hold.flight_instance_id
             JOIN flight_services service ON service.id=instance.flight_service_id
             JOIN LATERAL (
                 SELECT phone_country_code,phone_number FROM hold_passengers
                 WHERE seat_hold_id=hold.id ORDER BY ordinal LIMIT 1
             ) lead ON TRUE
             LEFT JOIN booking_contacts contact ON contact.seat_hold_id=hold.id
             LEFT JOIN booking_cancellations cancellation ON cancellation.ticket_id=ticket.id
             WHERE ticket.booking_reference=$1 AND attempt.status='SUCCEEDED'",
    )
    .bind(booking_reference)
    .fetch_optional(&mut **tx)
    .await
    .map_err(BookingManagementRepositoryError::Infrastructure)?;
    let Some(row) = row else {
        return Ok(None);
    };

    let passenger_rows = sqlx::query_as::<_, PassengerRow>(
        "SELECT ordinal,passenger_type,given_name,middle_name,family_name,gender
             FROM hold_passengers WHERE seat_hold_id=$1 ORDER BY ordinal",
    )
    .bind(row.hold_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(BookingManagementRepositoryError::Infrastructure)?;
    let seats = sqlx::query_scalar::<_, String>(
        "SELECT seat.seat_number FROM payment_attempt_seats finalized
             JOIN flight_seats seat ON seat.id=finalized.flight_seat_id
             WHERE finalized.payment_attempt_id=$1 ORDER BY seat.row_number,seat.column_code",
    )
    .bind(row.payment_attempt_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(BookingManagementRepositoryError::Infrastructure)?;
    let audit = sqlx::query_as::<_, AuditRow>(
        "SELECT action,actor_email,created_at FROM booking_operations_audit
             WHERE ticket_id=$1 ORDER BY created_at DESC,id DESC",
    )
    .bind(row.ticket_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(BookingManagementRepositoryError::Infrastructure)?;

    let ticket_status = parse_ticket_status(&row.ticket_status)?;
    let payment_status = PaymentStatus::parse_database(&row.payment_status)
        .ok_or(BookingManagementRepositoryError::InconsistentState)?;
    let eligibility = derive_travel_eligibility(row.departure_at, ticket_status, now);
    let arrival_date = row.arrival_day_offset.and_then(|offset| {
        u64::try_from(offset)
            .ok()
            .and_then(|days| row.travel_date.checked_add_days(Days::new(days)))
    });
    let currency_code = row.currency_code.clone();
    Ok(Some(BookingDetail {
        booking_reference: row.booking_reference,
        booking_status: booking_status(ticket_status),
        created_at: row.booking_created_at,
        journey: BookingJourney {
            flight_number: row.flight_number,
            origin_code: row.origin_code,
            destination_code: row.destination_code,
            travel_date: row.travel_date,
            departure_at: row.departure_at,
            departure_time: format_time(row.departure_time),
            arrival_date,
            arrival_time: format_time(row.arrival_time),
            origin_time_zone: row.origin_time_zone,
            aircraft_code: row.aircraft_code,
            cabin: row.cabin,
            flight_status: row.flight_status,
        },
        passengers: passenger_rows
            .into_iter()
            .map(PassengerRow::domain)
            .collect::<Result<Vec<_>, _>>()?,
        seats,
        contact: row.phone_country_code.zip(row.phone_number).map(
            |(phone_country_code, phone_number)| BookingContactSummary {
                phone_country_code,
                phone_number,
            },
        ),
        payment: BookingPaymentSummary {
            method: PaymentMethod::parse_database(&row.payment_method)
                .ok_or(BookingManagementRepositoryError::InconsistentState)?,
            provider: PaymentProvider::parse_database(&row.provider)
                .ok_or(BookingManagementRepositoryError::InconsistentState)?,
            status: payment_status,
            amount: Money {
                amount: row.amount,
                currency_code: currency_code.clone(),
            },
            succeeded_at: row.succeeded_at,
        },
        ticket: BookingTicketSummary {
            ticket_number: row.ticket_number,
            status: ticket_status,
            issued_at: row.issued_at,
            cancelled_at: row.ticket_cancelled_at,
        },
        cancellation: BookingCancellationSummary {
            eligibility: eligibility.cancellation,
            cutoff_at: eligibility.cancellation_cutoff_at,
            cancelled_at: row.cancelled_at,
            refund_status: row
                .refund_status
                .as_deref()
                .and_then(crate::domain::cancellation::RefundStatus::parse_database),
            refund_amount: row.refund_amount.map(|amount| Money {
                amount,
                currency_code,
            }),
            refunded_at: row.refunded_at,
        },
        audit: audit.into_iter().map(AuditRow::domain).collect(),
    }))
}

#[derive(FromRow)]
struct BookingListRow {
    booking_reference: String,
    lead_passenger_name: String,
    passenger_count: i64,
    flight_number: String,
    origin_code: String,
    destination_code: String,
    travel_date: NaiveDate,
    departure_at: Option<DateTime<Utc>>,
    cabin: String,
    ticket_status: String,
    flight_status: String,
    payment_status: String,
    refund_status: Option<String>,
    booked_at: DateTime<Utc>,
}

impl BookingListRow {
    fn domain(self) -> Result<BookingListItem, BookingManagementRepositoryError> {
        let ticket_status = parse_ticket_status(&self.ticket_status)?;
        Ok(BookingListItem {
            booking_reference: self.booking_reference,
            lead_passenger_name: self.lead_passenger_name,
            passenger_count: self.passenger_count,
            flight_number: self.flight_number,
            origin_code: self.origin_code,
            destination_code: self.destination_code,
            travel_date: self.travel_date,
            departure_at: self.departure_at,
            cabin: self.cabin,
            booking_status: booking_status(ticket_status),
            flight_status: self.flight_status,
            payment_status: PaymentStatus::parse_database(&self.payment_status)
                .ok_or(BookingManagementRepositoryError::InconsistentState)?,
            ticket_status,
            refund_status: self
                .refund_status
                .as_deref()
                .and_then(crate::domain::cancellation::RefundStatus::parse_database),
            booked_at: self.booked_at,
        })
    }
}

#[derive(FromRow)]
struct BookingDetailRow {
    ticket_id: Uuid,
    booking_reference: String,
    ticket_number: String,
    ticket_status: String,
    issued_at: DateTime<Utc>,
    ticket_cancelled_at: Option<DateTime<Utc>>,
    booking_created_at: DateTime<Utc>,
    payment_attempt_id: Uuid,
    provider: String,
    payment_method: String,
    payment_status: String,
    amount: i64,
    currency_code: String,
    succeeded_at: Option<DateTime<Utc>>,
    hold_id: Uuid,
    cabin: String,
    flight_number: String,
    origin_code: String,
    destination_code: String,
    aircraft_code: String,
    flight_status: String,
    travel_date: NaiveDate,
    departure_time: Option<NaiveTime>,
    arrival_time: Option<NaiveTime>,
    arrival_day_offset: Option<i16>,
    origin_time_zone: Option<String>,
    departure_at: Option<DateTime<Utc>>,
    phone_country_code: Option<String>,
    phone_number: Option<String>,
    refund_status: Option<String>,
    refund_amount: Option<i64>,
    cancelled_at: Option<DateTime<Utc>>,
    refunded_at: Option<DateTime<Utc>>,
}

#[derive(FromRow)]
struct PassengerRow {
    ordinal: i16,
    passenger_type: String,
    given_name: String,
    middle_name: Option<String>,
    family_name: String,
    gender: String,
}

impl PassengerRow {
    fn domain(self) -> Result<BookingPassenger, BookingManagementRepositoryError> {
        Ok(BookingPassenger {
            ordinal: u8::try_from(self.ordinal)
                .map_err(|_| BookingManagementRepositoryError::InconsistentState)?,
            passenger_type: PassengerType::parse_database(&self.passenger_type)
                .ok_or(BookingManagementRepositoryError::InconsistentState)?,
            display_name: [
                Some(self.given_name),
                self.middle_name,
                Some(self.family_name),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" "),
            gender: Gender::parse_database(&self.gender)
                .ok_or(BookingManagementRepositoryError::InconsistentState)?,
        })
    }
}

#[derive(FromRow)]
struct AuditRow {
    action: String,
    actor_email: String,
    created_at: DateTime<Utc>,
}

impl AuditRow {
    fn domain(self) -> BookingAuditEntry {
        BookingAuditEntry {
            action: self.action,
            actor_email: self.actor_email,
            created_at: self.created_at,
        }
    }
}

fn parse_ticket_status(value: &str) -> Result<TicketStatus, BookingManagementRepositoryError> {
    TicketStatus::parse_database(value).ok_or(BookingManagementRepositoryError::InconsistentState)
}

fn booking_status(ticket_status: TicketStatus) -> BookingStatus {
    match ticket_status {
        TicketStatus::Issued => BookingStatus::Confirmed,
        TicketStatus::Cancelled => BookingStatus::Cancelled,
    }
}

fn format_time(value: Option<NaiveTime>) -> Option<String> {
    value.map(|time| time.format("%H:%M").to_string())
}
