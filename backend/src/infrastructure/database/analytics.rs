use async_trait::async_trait;
use chrono::Utc;
use sqlx::{FromRow, PgPool};

use crate::application::analytics::{
    AnalyticsFilter, AnalyticsRepository, AnalyticsRepositoryError, DashboardCabin,
    DashboardFlight, DashboardInventory, DashboardInventoryFlight, DashboardReport, DashboardRoute,
    DashboardSummary, DashboardTrend, DASHBOARD_ACTIVE_CABINS, DASHBOARD_CURRENCY,
    DASHBOARD_TIME_ZONE,
};

#[derive(Clone, Debug)]
pub struct SqlxAnalyticsRepository {
    pool: PgPool,
}

impl SqlxAnalyticsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct SummaryRow {
    gross_revenue: i64,
    total_bookings: i64,
    tickets_issued: i64,
    cancelled_bookings: i64,
    refund_count: i64,
    refund_value: i64,
    pending_refund_count: i64,
    pending_refund_value: i64,
    attention_refund_count: i64,
}

#[derive(FromRow)]
struct InventorySummaryRow {
    booked_seats: i64,
    sellable_seats: i64,
}

const COHORT: &str = r#"
    FROM payment_attempts AS payment
    JOIN seat_holds AS hold ON hold.id = payment.seat_hold_id
    JOIN flight_instances AS instance ON instance.id = hold.flight_instance_id
    JOIN flight_services AS service ON service.id = instance.flight_service_id
    WHERE payment.status = 'SUCCEEDED'
      AND payment.currency_code = 'THB'
      AND payment.provider = $3
      AND (payment.succeeded_at AT TIME ZONE 'Asia/Bangkok')::date BETWEEN $1 AND $2
      AND ($4::text IS NULL OR service.origin_code || '-' || service.destination_code = $4)
      AND hold.cabin IN ('business', 'first')
      AND ($5::text IS NULL OR hold.cabin = $5)
"#;

fn round_percent(numerator: i64, denominator: i64) -> Option<f64> {
    (denominator > 0).then(|| ((numerator as f64 * 10_000.0 / denominator as f64).round()) / 100.0)
}

fn active_cohort_reconciles(
    summary: &SummaryRow,
    trends: &[DashboardTrend],
    cabins: &[DashboardCabin],
) -> bool {
    let cabins_are_active = cabins.iter().all(|row| {
        DASHBOARD_ACTIVE_CABINS.contains(&row.cabin.as_str())
            && cabins
                .iter()
                .filter(|candidate| candidate.cabin == row.cabin)
                .count()
                == 1
    });
    let cabin_bookings: i64 = cabins.iter().map(|row| row.bookings).sum();
    let cabin_revenue: i64 = cabins.iter().map(|row| row.revenue).sum();
    let trend_bookings: i64 = trends.iter().map(|row| row.bookings).sum();
    let trend_revenue: i64 = trends.iter().map(|row| row.revenue).sum();

    cabins_are_active
        && cabin_bookings == summary.total_bookings
        && cabin_revenue == summary.gross_revenue
        && trend_bookings == summary.total_bookings
        && trend_revenue == summary.gross_revenue
}

#[async_trait]
impl AnalyticsRepository for SqlxAnalyticsRepository {
    async fn dashboard(
        &self,
        filter: &AnalyticsFilter,
    ) -> Result<DashboardReport, AnalyticsRepositoryError> {
        let route = filter.route.as_deref();
        let cabin = filter.cabin.map(|value| value.as_str());
        let provider = filter.provider.as_str();
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(AnalyticsRepositoryError::Infrastructure)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *transaction)
            .await
            .map_err(AnalyticsRepositoryError::Infrastructure)?;

        let summary_sql = format!(
            "WITH cohort AS (SELECT payment.id, payment.amount {COHORT})
             SELECT COALESCE(SUM(cohort.amount), 0)::bigint AS gross_revenue,
                    COUNT(cohort.id)::bigint AS total_bookings,
                    COUNT(ticket.id)::bigint AS tickets_issued,
                    COUNT(ticket.id) FILTER (WHERE ticket.status = 'CANCELLED')::bigint AS cancelled_bookings,
                    COUNT(cancellation.id) FILTER (WHERE cancellation.refund_status = 'SUCCEEDED')::bigint AS refund_count,
                    COALESCE(SUM(cancellation.refund_amount) FILTER (WHERE cancellation.refund_status = 'SUCCEEDED' AND cancellation.currency_code = 'THB'), 0)::bigint AS refund_value,
                    COUNT(cancellation.id) FILTER (WHERE cancellation.refund_status IN ('PENDING','IN_FLIGHT','PROCESSING'))::bigint AS pending_refund_count,
                    COALESCE(SUM(cancellation.refund_amount) FILTER (WHERE cancellation.refund_status IN ('PENDING','IN_FLIGHT','PROCESSING') AND cancellation.currency_code = 'THB'), 0)::bigint AS pending_refund_value,
                    COUNT(cancellation.id) FILTER (WHERE cancellation.refund_status = 'REQUIRES_ATTENTION')::bigint AS attention_refund_count
             FROM cohort
             LEFT JOIN tickets AS ticket ON ticket.payment_attempt_id = cohort.id
             LEFT JOIN booking_cancellations AS cancellation ON cancellation.payment_attempt_id = cohort.id"
        );
        let summary: SummaryRow = sqlx::query_as(&summary_sql)
            .bind(filter.from)
            .bind(filter.to)
            .bind(provider)
            .bind(route)
            .bind(cabin)
            .fetch_one(&mut *transaction)
            .await
            .map_err(AnalyticsRepositoryError::Infrastructure)?;

        let trend_sql = format!(
            "WITH days AS (SELECT generate_series($1::date, $2::date, INTERVAL '1 day')::date AS date),
                   cohort AS (SELECT (payment.succeeded_at AT TIME ZONE 'Asia/Bangkok')::date AS date, payment.amount {COHORT}),
                   totals AS (SELECT date, COUNT(*)::bigint AS bookings, SUM(amount)::bigint AS revenue FROM cohort GROUP BY date)
             SELECT days.date, COALESCE(totals.bookings, 0)::bigint AS bookings, COALESCE(totals.revenue, 0)::bigint AS revenue
             FROM days LEFT JOIN totals USING (date) ORDER BY days.date"
        );
        let trends = sqlx::query_as::<_, DashboardTrend>(&trend_sql)
            .bind(filter.from)
            .bind(filter.to)
            .bind(provider)
            .bind(route)
            .bind(cabin)
            .fetch_all(&mut *transaction)
            .await
            .map_err(AnalyticsRepositoryError::Infrastructure)?;

        let route_sql = format!(
            "SELECT service.origin_code || '-' || service.destination_code AS route, COUNT(*)::bigint AS bookings, SUM(payment.amount)::bigint AS revenue
             {COHORT} GROUP BY service.origin_code, service.destination_code
             ORDER BY revenue DESC, bookings DESC, route LIMIT 10"
        );
        let routes = sqlx::query_as::<_, DashboardRoute>(&route_sql)
            .bind(filter.from)
            .bind(filter.to)
            .bind(provider)
            .bind(route)
            .bind(cabin)
            .fetch_all(&mut *transaction)
            .await
            .map_err(AnalyticsRepositoryError::Infrastructure)?;

        let cabin_sql = format!(
            "SELECT hold.cabin, COUNT(*)::bigint AS bookings, SUM(payment.amount)::bigint AS revenue
             {COHORT} GROUP BY hold.cabin ORDER BY revenue DESC, hold.cabin"
        );
        let cabins = sqlx::query_as::<_, DashboardCabin>(&cabin_sql)
            .bind(filter.from)
            .bind(filter.to)
            .bind(provider)
            .bind(route)
            .bind(cabin)
            .fetch_all(&mut *transaction)
            .await
            .map_err(AnalyticsRepositoryError::Infrastructure)?;

        let flight_base = format!(
            "SELECT service.flight_number, service.origin_code || '-' || service.destination_code AS route,
                    instance.departure_date, COUNT(*)::bigint AS bookings, SUM(payment.amount)::bigint AS revenue
             {COHORT} GROUP BY service.flight_number, service.origin_code, service.destination_code, instance.departure_date"
        );
        let flights = sqlx::query_as::<_, DashboardFlight>(&format!(
            "{flight_base} ORDER BY bookings DESC, revenue DESC, flight_number, departure_date LIMIT 10"
        )).bind(filter.from).bind(filter.to).bind(provider).bind(route).bind(cabin)
          .fetch_all(&mut *transaction).await.map_err(AnalyticsRepositoryError::Infrastructure)?;
        let revenue_flights = sqlx::query_as::<_, DashboardFlight>(&format!(
            "{flight_base} ORDER BY revenue DESC, bookings DESC, flight_number, departure_date LIMIT 10"
        )).bind(filter.from).bind(filter.to).bind(provider).bind(route).bind(cabin)
          .fetch_all(&mut *transaction).await.map_err(AnalyticsRepositoryError::Infrastructure)?;

        let inventory_where = r#"
            FROM flight_seats AS seat
            JOIN flight_instances AS instance ON instance.id = seat.flight_instance_id
            JOIN flight_services AS service ON service.id = instance.flight_service_id
            WHERE instance.departure_date BETWEEN $1 AND $2
              AND ($3::text IS NULL OR service.origin_code || '-' || service.destination_code = $3)
              AND seat.cabin IN ('business', 'first')
              AND ($4::text IS NULL OR seat.cabin = $4)
        "#;
        let inventory_summary: InventorySummaryRow = sqlx::query_as(&format!(
            "SELECT COUNT(*) FILTER (WHERE seat.sellable AND seat.booking_status = 'BOOKED')::bigint AS booked_seats,
                    COUNT(*) FILTER (WHERE seat.sellable)::bigint AS sellable_seats {inventory_where}"
        )).bind(filter.from).bind(filter.to).bind(route).bind(cabin)
          .fetch_one(&mut *transaction).await.map_err(AnalyticsRepositoryError::Infrastructure)?;
        let inventory_flights = sqlx::query_as::<_, DashboardInventoryFlight>(&format!(
            "SELECT service.flight_number, service.origin_code || '-' || service.destination_code AS route, instance.departure_date,
                    COUNT(*) FILTER (WHERE seat.sellable AND seat.booking_status = 'BOOKED')::bigint AS booked_seats,
                    COUNT(*) FILTER (WHERE seat.sellable)::bigint AS sellable_seats,
                    ROUND(100.0 * COUNT(*) FILTER (WHERE seat.sellable AND seat.booking_status = 'BOOKED') / NULLIF(COUNT(*) FILTER (WHERE seat.sellable), 0), 2)::float8 AS occupancy_percent
             {inventory_where}
             GROUP BY service.flight_number, service.origin_code, service.destination_code, instance.departure_date
             HAVING COUNT(*) FILTER (WHERE seat.sellable) > 0
             ORDER BY occupancy_percent, sellable_seats DESC, flight_number, departure_date LIMIT 10"
        )).bind(filter.from).bind(filter.to).bind(route).bind(cabin)
          .fetch_all(&mut *transaction).await.map_err(AnalyticsRepositoryError::Infrastructure)?;
        let available_routes: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT origin_code || '-' || destination_code FROM flight_services ORDER BY 1",
        ).fetch_all(&mut *transaction).await.map_err(AnalyticsRepositoryError::Infrastructure)?;

        if !active_cohort_reconciles(&summary, &trends, &cabins) {
            return Err(AnalyticsRepositoryError::InconsistentActiveCabinCohort);
        }

        transaction
            .commit()
            .await
            .map_err(AnalyticsRepositoryError::Infrastructure)?;
        let cancellation_rate_percent =
            round_percent(summary.cancelled_bookings, summary.total_bookings);
        let average_booking_value = (summary.total_bookings > 0)
            .then(|| summary.gross_revenue as f64 / summary.total_bookings as f64);
        let inventory_percent = round_percent(
            inventory_summary.booked_seats,
            inventory_summary.sellable_seats,
        );
        Ok(DashboardReport {
            from: filter.from,
            to: filter.to,
            time_zone: DASHBOARD_TIME_ZONE,
            currency: DASHBOARD_CURRENCY,
            provider: filter.provider,
            generated_at: Utc::now(),
            active_cabins: DASHBOARD_ACTIVE_CABINS,
            summary: DashboardSummary {
                gross_revenue: summary.gross_revenue,
                total_bookings: summary.total_bookings,
                tickets_issued: summary.tickets_issued,
                cancelled_bookings: summary.cancelled_bookings,
                cancellation_rate_percent,
                refund_count: summary.refund_count,
                refund_value: summary.refund_value,
                pending_refund_count: summary.pending_refund_count,
                pending_refund_value: summary.pending_refund_value,
                attention_refund_count: summary.attention_refund_count,
                average_booking_value,
            },
            trends,
            routes,
            cabins,
            flights,
            revenue_flights,
            inventory: DashboardInventory {
                booked_seats: inventory_summary.booked_seats,
                sellable_seats: inventory_summary.sellable_seats,
                occupancy_percent: inventory_percent,
                flights: inventory_flights,
            },
            available_routes,
        })
    }
}
