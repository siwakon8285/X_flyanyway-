use async_trait::async_trait;
use chrono::Utc;
use sqlx::{FromRow, PgPool};

use crate::application::external_analytics::{
    ExternalAnalyticsFilter, ExternalAnalyticsPeriod, ExternalAnalyticsRepository,
    ExternalAnalyticsRepositoryError, ExternalAnalyticsSummary,
};

#[derive(Clone, Debug)]
pub struct SqlxExternalAnalyticsRepository {
    pool: PgPool,
}

impl SqlxExternalAnalyticsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct AggregateRow {
    total_bookings: i64,
    tickets_issued: i64,
    cancelled_bookings: i64,
    booked_seats: i64,
    sellable_seats: i64,
}

#[async_trait]
impl ExternalAnalyticsRepository for SqlxExternalAnalyticsRepository {
    async fn summary(
        &self,
        filter: &ExternalAnalyticsFilter,
    ) -> Result<ExternalAnalyticsSummary, ExternalAnalyticsRepositoryError> {
        let cabins: Vec<String> = filter
            .cabin
            .map(|cabin| vec![cabin.as_str().to_owned()])
            .unwrap_or_else(|| {
                vec![
                    crate::domain::value_objects::CabinClass::Business
                        .as_str()
                        .to_owned(),
                    crate::domain::value_objects::CabinClass::First
                        .as_str()
                        .to_owned(),
                ]
            });
        let route = filter.route.as_deref();
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(ExternalAnalyticsRepositoryError::Infrastructure)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *transaction)
            .await
            .map_err(ExternalAnalyticsRepositoryError::Infrastructure)?;

        let row: AggregateRow = sqlx::query_as(
            r#"
            WITH flight_cohort AS (
                SELECT instance.id AS flight_instance_id
                FROM flight_instances AS instance
                JOIN flight_services AS service
                  ON service.id = instance.flight_service_id
                JOIN airports AS origin
                  ON origin.code = service.origin_code
                WHERE service.departure_time IS NOT NULL
                  AND (
                      (
                          (instance.departure_date + service.departure_time)
                          AT TIME ZONE COALESCE(service.origin_time_zone, origin.time_zone)
                      ) AT TIME ZONE 'Asia/Bangkok'
                  )::date BETWEEN $1 AND $2
                  AND (
                      $3::text IS NULL
                      OR service.origin_code || '-' || service.destination_code = $3
                  )
            ), booking_aggregate AS (
                SELECT
                    COUNT(DISTINCT payment.id)::bigint AS total_bookings,
                    COUNT(DISTINCT payment.id) FILTER (
                        WHERE ticket.status = 'CANCELLED'
                    )::bigint AS cancelled_bookings,
                    COUNT(DISTINCT ticket.id) FILTER (
                        WHERE ticket.issued_at IS NOT NULL
                    )::bigint AS tickets_issued
                FROM payment_attempts AS payment
                JOIN seat_holds AS hold
                  ON hold.id = payment.seat_hold_id
                JOIN flight_cohort AS cohort
                  ON cohort.flight_instance_id = hold.flight_instance_id
                LEFT JOIN tickets AS ticket
                  ON ticket.payment_attempt_id = payment.id
                WHERE payment.status = 'SUCCEEDED'
                  AND hold.cabin = ANY($4::text[])
            ), booked_seat_aggregate AS (
                SELECT COUNT(DISTINCT finalized.flight_seat_id)::bigint AS booked_seats
                FROM payment_attempt_seats AS finalized
                JOIN payment_attempts AS payment
                  ON payment.id = finalized.payment_attempt_id
                JOIN seat_holds AS hold
                  ON hold.id = payment.seat_hold_id
                JOIN flight_cohort AS cohort
                  ON cohort.flight_instance_id = hold.flight_instance_id
                JOIN flight_seats AS seat
                  ON seat.id = finalized.flight_seat_id
                LEFT JOIN tickets AS ticket
                  ON ticket.payment_attempt_id = payment.id
                WHERE payment.status = 'SUCCEEDED'
                  AND finalized.released_at IS NULL
                  AND seat.booking_status = 'BOOKED'
                  AND ticket.status IS DISTINCT FROM 'CANCELLED'
                  AND seat.cabin = ANY($5::text[])
            ), inventory_aggregate AS (
                SELECT COUNT(*) FILTER (
                    WHERE seat.sellable = TRUE
                )::bigint AS sellable_seats
                FROM flight_seats AS seat
                JOIN flight_instances AS instance
                  ON instance.id = seat.flight_instance_id
                JOIN flight_cohort AS cohort
                  ON cohort.flight_instance_id = instance.id
                JOIN flight_services AS service
                  ON service.id = instance.flight_service_id
                WHERE service.status = 'SCHEDULED'
                  AND seat.cabin = ANY($6::text[])
            )
            SELECT
                COALESCE(booking_aggregate.total_bookings, 0)::bigint AS total_bookings,
                COALESCE(booking_aggregate.tickets_issued, 0)::bigint AS tickets_issued,
                COALESCE(booking_aggregate.cancelled_bookings, 0)::bigint AS cancelled_bookings,
                COALESCE(booked_seat_aggregate.booked_seats, 0)::bigint AS booked_seats,
                COALESCE(inventory_aggregate.sellable_seats, 0)::bigint AS sellable_seats
            FROM booking_aggregate
            CROSS JOIN booked_seat_aggregate
            CROSS JOIN inventory_aggregate
            "#,
        )
        .bind(filter.from)
        .bind(filter.to)
        .bind(route)
        .bind(&cabins)
        .bind(&cabins)
        .bind(&cabins)
        .fetch_one(&mut *transaction)
        .await
        .map_err(ExternalAnalyticsRepositoryError::Infrastructure)?;

        transaction
            .commit()
            .await
            .map_err(ExternalAnalyticsRepositoryError::Infrastructure)?;

        let occupancy_percent = if row.sellable_seats == 0 {
            0.0
        } else {
            ((row.booked_seats as f64 * 100.0 / row.sellable_seats as f64) * 100.0).round() / 100.0
        };
        Ok(ExternalAnalyticsSummary {
            period: ExternalAnalyticsPeriod {
                from: filter.from,
                to: filter.to,
            },
            generated_at: Utc::now(),
            total_bookings: row.total_bookings,
            tickets_issued: row.tickets_issued,
            cancelled_bookings: row.cancelled_bookings,
            booked_seats: row.booked_seats,
            sellable_seats: row.sellable_seats,
            occupancy_percent,
        })
    }
}
