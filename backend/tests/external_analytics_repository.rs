mod common;

use std::{
    future::Future,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

use x_fly_api::{
    application::external_analytics::{ExternalAnalyticsFilter, ExternalAnalyticsService},
    domain::value_objects::CabinClass,
    infrastructure::database::{
        migrate_database, verify_database_ready, SqlxExternalAnalyticsRepository,
    },
};

const BOUNDARY_NOW: &str = "2040-01-15T00:00:00Z";
const FIXTURE_NAMESPACE_LOCK: i64 = 0x5846_4c59_5445_5354;
const FIXTURE_BLOCK_DAYS: i64 = 128;
const FIXTURE_ANCHOR_OFFSET_DAYS: i64 = 16;

fn fixture_reference_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2040, 1, 1).expect("fixture reference date")
}

fn fixture_window() -> (NaiveDate, NaiveDate) {
    (
        NaiveDate::from_ymd_opt(2300, 1, 1).expect("fixture window start"),
        NaiveDate::from_ymd_opt(2400, 1, 1).expect("fixture window end"),
    )
}

async fn pools() -> (PgPool, PgPool) {
    let setup = PgPoolOptions::new()
        .max_connections(8)
        .connect(&common::test_database_url())
        .await
        .expect("connect to TEST setup database");
    migrate_database(&setup)
        .await
        .expect("TEST migration chain is current");
    let runtime = PgPoolOptions::new()
        .max_connections(8)
        .connect(&common::test_runtime_database_url())
        .await
        .expect("connect to TEST runtime database");
    verify_database_ready(&runtime)
        .await
        .expect("runtime sees ready TEST schema");
    (setup, runtime)
}

#[derive(Clone)]
struct Fixture {
    reservation_service: Uuid,
    anchor_date: NaiveDate,
    services: Arc<Mutex<Vec<Uuid>>>,
    instances: Arc<Mutex<Vec<Uuid>>>,
    holds: Arc<Mutex<Vec<Uuid>>>,
    attempts: Arc<Mutex<Vec<Uuid>>>,
    tickets: Arc<Mutex<Vec<Uuid>>>,
    seats: Arc<Mutex<Vec<Uuid>>>,
}

impl Fixture {
    async fn new(pool: &PgPool) -> Self {
        let (window_start, window_end) = fixture_window();
        let mut transaction = pool
            .begin()
            .await
            .expect("begin analytics fixture namespace allocation");
        sqlx::query("SELECT pg_advisory_xact_lock($1)")
            .bind(FIXTURE_NAMESPACE_LOCK)
            .execute(&mut *transaction)
            .await
            .expect("lock analytics fixture namespace allocation");
        let candidate: NaiveDate = sqlx::query_scalar(
            "SELECT candidate::date
             FROM generate_series(
                 $1::date,
                 $2::date - ($3::bigint * INTERVAL '1 day'),
                 INTERVAL '1 day'
             ) AS candidate
             WHERE NOT EXISTS (
                 SELECT 1
                 FROM flight_instances AS instance
                 WHERE instance.departure_date >= candidate::date
                   AND instance.departure_date < candidate::date + ($3::bigint * INTERVAL '1 day')
             )
             ORDER BY candidate
             LIMIT 1",
        )
        .bind(window_start)
        .bind(window_end)
        .bind(FIXTURE_BLOCK_DAYS)
        .fetch_optional(&mut *transaction)
        .await
        .expect("inspect analytics fixture date blocks")
        .expect("analytics fixture date window exhausted");

        let reservation_service: Uuid = sqlx::query_scalar(
            "INSERT INTO flight_services (
                 public_id,flight_number,origin_code,destination_code,aircraft_code,
                 origin_time_zone,departure_time,arrival_time,arrival_day_offset,
                 duration_minutes,stops,status,operating_date
             ) VALUES ($1,$2,'BKK','DXB','A320','Asia/Bangkok','00:00','04:00',0,240,
                 'DIRECT','SCHEDULED',$3) RETURNING id",
        )
        .bind(format!(
            "external-analytics-reservation-{}",
            Uuid::new_v4().simple()
        ))
        .bind(format!("XR{}", Uuid::new_v4().simple()))
        .bind(candidate)
        .fetch_one(&mut *transaction)
        .await
        .expect("insert analytics fixture reservation service");
        sqlx::query(
            "INSERT INTO flight_instances (flight_service_id,departure_date)
             SELECT $1, candidate::date
             FROM generate_series(
                 $2::date,
                 $2::date + (($3::bigint - 1) * INTERVAL '1 day'),
                 INTERVAL '1 day'
             ) AS candidate",
        )
        .bind(reservation_service)
        .bind(candidate)
        .bind(FIXTURE_BLOCK_DAYS)
        .execute(&mut *transaction)
        .await
        .expect("reserve analytics fixture date block");
        transaction
            .commit()
            .await
            .expect("commit analytics fixture date block");

        Self {
            reservation_service,
            anchor_date: candidate + Duration::days(FIXTURE_ANCHOR_OFFSET_DAYS),
            services: Arc::new(Mutex::new(Vec::new())),
            instances: Arc::new(Mutex::new(Vec::new())),
            holds: Arc::new(Mutex::new(Vec::new())),
            attempts: Arc::new(Mutex::new(Vec::new())),
            tickets: Arc::new(Mutex::new(Vec::new())),
            seats: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn date(&self, date: NaiveDate) -> NaiveDate {
        self.anchor_date + date.signed_duration_since(fixture_reference_date())
    }
}

async fn insert_service(
    pool: &PgPool,
    fixture: &mut Fixture,
    origin: &str,
    destination: &str,
    origin_time_zone: &str,
    departure: NaiveDate,
    departure_time: NaiveTime,
) -> (Uuid, Uuid) {
    let departure = fixture.date(departure);
    let service_id: Uuid = sqlx::query_scalar(
        "INSERT INTO flight_services (
             public_id,flight_number,origin_code,destination_code,aircraft_code,
             origin_time_zone,departure_time,arrival_time,arrival_day_offset,
             duration_minutes,stops,status,operating_date
         ) VALUES ($1,$2,$3,$4,'A320',$5,$6,'12:00',0,120,'DIRECT','SCHEDULED',$7)
         RETURNING id",
    )
    .bind(format!("external-analytics-{}", Uuid::new_v4().simple()))
    .bind(format!("XA{}", Uuid::new_v4().simple()))
    .bind(origin)
    .bind(destination)
    .bind(origin_time_zone)
    .bind(departure_time)
    .bind(departure)
    .fetch_one(pool)
    .await
    .expect("insert analytics flight service");
    // Record the parent immediately so a later child-insert failure can still
    // be removed by the scoped finalizer (ON DELETE CASCADE handles instances).
    fixture
        .services
        .lock()
        .expect("service tracker lock")
        .push(service_id);
    let instance_id: Uuid = sqlx::query_scalar(
        "INSERT INTO flight_instances (flight_service_id,departure_date)
         VALUES ($1,$2) RETURNING id",
    )
    .bind(service_id)
    .bind(departure)
    .fetch_one(pool)
    .await
    .expect("insert analytics flight instance");
    fixture
        .instances
        .lock()
        .expect("instance tracker lock")
        .push(instance_id);
    (service_id, instance_id)
}

async fn insert_inventory_seat(
    pool: &PgPool,
    fixture: &mut Fixture,
    instance_id: Uuid,
    cabin: &str,
    seat_number: &str,
    sellable: bool,
    booking_status: &str,
) -> Uuid {
    let seat_id: Uuid = sqlx::query_scalar(
        "INSERT INTO flight_seats (
             flight_instance_id,seat_number,row_number,column_code,cabin,position,
             sellable,booking_status,booked_at
         ) VALUES ($1,$2,$3,'A',$4,'window',$5,$6,
             CASE WHEN $6='BOOKED' THEN TIMESTAMPTZ '2040-01-01 00:00:00+00' ELSE NULL END)
         RETURNING id",
    )
    .bind(instance_id)
    .bind(seat_number)
    .bind(
        seat_number
            .trim_end_matches('A')
            .parse::<i16>()
            .unwrap_or(1),
    )
    .bind(cabin)
    .bind(sellable)
    .bind(booking_status)
    .fetch_one(pool)
    .await
    .expect("insert analytics inventory seat");
    fixture
        .seats
        .lock()
        .expect("seat tracker lock")
        .push(seat_id);
    seat_id
}

async fn insert_booking(
    pool: &PgPool,
    fixture: &mut Fixture,
    instance_id: Uuid,
    cabin: &str,
    provider: &str,
    ticket_status: Option<&str>,
    payment_time: &str,
) -> (Uuid, Option<Uuid>) {
    let hold_id: Uuid = sqlx::query_scalar(
        "INSERT INTO seat_holds (
             flight_instance_id,cabin,adults,children,infants,access_token_hash,
             expires_at,consumed_at
         ) VALUES ($1,$2,1,0,0,$3,TIMESTAMPTZ '2040-02-01 00:00:00+00',
             TIMESTAMPTZ '2040-01-01 00:00:00+00') RETURNING id",
    )
    .bind(instance_id)
    .bind(cabin)
    .bind([7_u8; 32].as_slice())
    .fetch_one(pool)
    .await
    .expect("insert analytics seat hold");
    fixture
        .holds
        .lock()
        .expect("hold tracker lock")
        .push(hold_id);
    let attempt_id: Uuid = sqlx::query_scalar(
        "INSERT INTO payment_attempts (
             seat_hold_id,request_id,request_fingerprint,provider,payment_method,status,
             amount,currency_code,review_priced_at,provider_reference,succeeded_at,
             payment_finalization_deadline
         ) VALUES ($1,$2,$3,$4,CASE WHEN $4='MOCK_BITCOIN' THEN 'BITCOIN' ELSE 'CARD' END,
             'SUCCEEDED',1000,'THB',$5::timestamptz,
             CASE WHEN $4='STRIPE' THEN $6 ELSE NULL END,
             $5::timestamptz,CASE WHEN $4='STRIPE' THEN TIMESTAMPTZ '2040-02-01 00:00:00+00' ELSE NULL END)
         RETURNING id",
    )
    .bind(hold_id)
    .bind(Uuid::new_v4())
    .bind([8_u8; 32].as_slice())
    .bind(provider)
    .bind(payment_time)
    .bind(format!("pi_{}", Uuid::new_v4().simple()))
    .fetch_one(pool)
        .await
        .expect("insert analytics successful payment");
    fixture
        .attempts
        .lock()
        .expect("attempt tracker lock")
        .push(attempt_id);

    let ticket_id = if let Some(status) = ticket_status {
        const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let booking_suffix: String = Uuid::new_v4()
            .as_bytes()
            .iter()
            .take(8)
            .map(|byte| ALPHABET[(*byte & 31) as usize] as char)
            .collect();
        let ticket_suffix: String = Uuid::new_v4()
            .as_bytes()
            .iter()
            .cycle()
            .take(12)
            .map(|byte| ALPHABET[(*byte & 31) as usize] as char)
            .collect();
        let ticket_id: Uuid = sqlx::query_scalar(
            "INSERT INTO tickets (
                 payment_attempt_id,booking_reference,ticket_number,status,issued_at,cancelled_at
             ) VALUES ($1,$2,$3,$4,TIMESTAMPTZ '2040-01-02 00:00:00+00',
                 CASE WHEN $4='CANCELLED' THEN TIMESTAMPTZ '2040-01-03 00:00:00+00' ELSE NULL END)
             RETURNING id",
        )
        .bind(attempt_id)
        .bind(format!("XF{booking_suffix}"))
        .bind(format!("XFT{ticket_suffix}"))
        .bind(status)
        .fetch_one(pool)
        .await
        .expect("insert analytics ticket");
        fixture
            .tickets
            .lock()
            .expect("ticket tracker lock")
            .push(ticket_id);
        Some(ticket_id)
    } else {
        None
    };
    (attempt_id, ticket_id)
}

async fn link_booked_seat(
    pool: &PgPool,
    attempt_id: Uuid,
    seat_id: Uuid,
    released_at: Option<&str>,
) {
    link_booked_seat_for_passenger(pool, attempt_id, seat_id, 1, released_at).await;
}

async fn link_booked_seat_for_passenger(
    pool: &PgPool,
    attempt_id: Uuid,
    seat_id: Uuid,
    passenger_ordinal: i16,
    released_at: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO payment_attempt_seats
             (payment_attempt_id,flight_seat_id,passenger_ordinal,released_at)
         VALUES ($1,$2,$3,$4::timestamptz)",
    )
    .bind(attempt_id)
    .bind(seat_id)
    .bind(passenger_ordinal)
    .bind(released_at)
    .execute(pool)
    .await
    .expect("link analytics finalized seat");
}

async fn add_passengers_for_attempt(pool: &PgPool, attempt_id: Uuid, count: i16) {
    let hold_id: Uuid = sqlx::query_scalar("SELECT seat_hold_id FROM payment_attempts WHERE id=$1")
        .bind(attempt_id)
        .fetch_one(pool)
        .await
        .expect("find analytics multi-passenger hold");
    sqlx::query("UPDATE seat_holds SET adults=$2 WHERE id=$1")
        .bind(hold_id)
        .bind(count)
        .execute(pool)
        .await
        .expect("set analytics multi-passenger hold");

    for ordinal in 1..=count {
        let passport_suffix = Uuid::new_v4().simple().to_string();
        sqlx::query(
            "INSERT INTO hold_passengers (
                 seat_hold_id,ordinal,passenger_type,title,given_name,family_name,
                 date_of_birth,gender,nationality_code,passport_number,
                 passport_issuing_country_code,passport_expiry_date,email,
                 phone_country_code,phone_number
             ) VALUES ($1,$2,'ADULT','MR',$3,'Fanout','1990-01-01','UNSPECIFIED',
                 'TH',$4,'TH','2099-01-01',NULL,NULL,NULL)",
        )
        .bind(hold_id)
        .bind(ordinal)
        .bind(format!("Passenger{ordinal}"))
        .bind(format!("P{}", &passport_suffix[..12]).to_ascii_uppercase())
        .execute(pool)
        .await
        .expect("insert analytics passenger");
    }
}

async fn insert_failed_attempt_for_hold(
    pool: &PgPool,
    fixture: &mut Fixture,
    successful_attempt_id: Uuid,
) {
    let hold_id: Uuid = sqlx::query_scalar("SELECT seat_hold_id FROM payment_attempts WHERE id=$1")
        .bind(successful_attempt_id)
        .fetch_one(pool)
        .await
        .expect("find analytics hold for failed retry");
    let attempt_id: Uuid = sqlx::query_scalar(
        "INSERT INTO payment_attempts (
             seat_hold_id,request_id,request_fingerprint,provider,payment_method,status,
             amount,currency_code,review_priced_at,provider_reference,failure_code,
             failure_message,succeeded_at,payment_finalization_deadline
         ) VALUES ($1,$2,$3,'MOCK_BITCOIN','BITCOIN','FAILED',1000,'THB',
             TIMESTAMPTZ '2040-01-01 00:00:00+00',NULL,'TEST_FAILED',
             'fixture payment did not succeed',NULL,NULL)
         RETURNING id",
    )
    .bind(hold_id)
    .bind(Uuid::new_v4())
    .bind([9_u8; 32].as_slice())
    .fetch_one(pool)
    .await
    .expect("insert analytics failed retry");
    fixture
        .attempts
        .lock()
        .expect("attempt tracker lock")
        .push(attempt_id);
}

async fn mark_payment_failed(pool: &PgPool, attempt_id: Uuid) {
    sqlx::query(
        "UPDATE payment_attempts
         SET status='FAILED', succeeded_at=NULL, failure_code='TEST_FAILED',
             failure_message='fixture payment did not succeed'
         WHERE id=$1",
    )
    .bind(attempt_id)
    .execute(pool)
    .await
    .expect("mark analytics payment failed");
}

fn tracked_ids(tracker: &Arc<Mutex<Vec<Uuid>>>, label: &str) -> Result<Vec<Uuid>, sqlx::Error> {
    tracker
        .lock()
        .map_err(|_| sqlx::Error::Protocol(format!("{label} fixture tracker poisoned")))
        .map(|ids| ids.clone())
}

async fn cleanup(pool: &PgPool, fixture: &Fixture) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let tickets = tracked_ids(&fixture.tickets, "ticket")?;
    let attempts = tracked_ids(&fixture.attempts, "attempt")?;
    let holds = tracked_ids(&fixture.holds, "hold")?;
    let seats = tracked_ids(&fixture.seats, "seat")?;
    let instances = tracked_ids(&fixture.instances, "instance")?;
    let services = tracked_ids(&fixture.services, "service")?;
    for ticket_id in &tickets {
        sqlx::query("DELETE FROM tickets WHERE id=$1")
            .bind(ticket_id)
            .execute(&mut *tx)
            .await?;
    }
    for attempt_id in &attempts {
        sqlx::query("DELETE FROM payment_attempt_seats WHERE payment_attempt_id=$1")
            .bind(attempt_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM payment_attempts WHERE id=$1")
            .bind(attempt_id)
            .execute(&mut *tx)
            .await?;
    }
    for hold_id in &holds {
        sqlx::query("DELETE FROM seat_holds WHERE id=$1")
            .bind(hold_id)
            .execute(&mut *tx)
            .await?;
    }
    for seat_id in &seats {
        sqlx::query("DELETE FROM flight_seats WHERE id=$1")
            .bind(seat_id)
            .execute(&mut *tx)
            .await?;
    }
    for instance_id in &instances {
        sqlx::query("DELETE FROM flight_instances WHERE id=$1")
            .bind(instance_id)
            .execute(&mut *tx)
            .await?;
    }
    for service_id in &services {
        sqlx::query("DELETE FROM flight_service_cabins WHERE flight_service_id=$1")
            .bind(service_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM flight_services WHERE id=$1")
            .bind(service_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM flight_instances WHERE flight_service_id=$1")
        .bind(fixture.reservation_service)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM flight_services WHERE id=$1")
        .bind(fixture.reservation_service)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

async fn run_fixture_body<F, Fut>(setup: PgPool, runtime: PgPool, fixture: Fixture, body: F)
where
    F: FnOnce(PgPool, PgPool, Fixture) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let cleanup_setup = setup.clone();
    let cleanup_fixture = fixture.clone();
    common::run_fixture_body_with_cleanup(
        move || body(setup, runtime, fixture),
        move || async move { cleanup(&cleanup_setup, &cleanup_fixture).await },
    )
    .await;
}

#[tokio::test]
async fn dynamically_added_service_is_cleaned_when_body_mutates_fixture() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, _runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    let body_fixture = fixture.clone();
    let cleanup_fixture = fixture.clone();
    let body_setup = setup.clone();
    let cleanup_setup = setup.clone();
    let created_service = Arc::new(Mutex::new(None));
    let created_service_for_body = Arc::clone(&created_service);

    common::run_fixture_body_with_cleanup(
        move || async move {
            let mut fixture = body_fixture;
            let (service_id, _) = insert_service(
                &body_setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 1, 10).expect("fixture date"),
                NaiveTime::from_hms_opt(12, 0, 0).expect("fixture time"),
            )
            .await;
            *created_service_for_body
                .lock()
                .expect("service tracker lock") = Some(service_id);
        },
        move || async move { cleanup(&cleanup_setup, &cleanup_fixture).await },
    )
    .await;

    let service_id = created_service
        .lock()
        .expect("service tracker lock")
        .expect("body created a service");
    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM flight_services WHERE id=$1")
        .bind(service_id)
        .fetch_one(&setup)
        .await
        .expect("inspect dynamically-created analytics service");
    assert_eq!(remaining, 0, "body-created service must be finalized");
}

fn filter(from: &str, to: &str) -> ExternalAnalyticsFilter {
    ExternalAnalyticsFilter::parse(
        Some(from),
        Some(to),
        None,
        None,
        DateTime::parse_from_rfc3339(BOUNDARY_NOW)
            .unwrap()
            .with_timezone(&Utc),
    )
    .expect("valid analytics filter")
}

async fn summary(
    setup: &PgPool,
    runtime: &PgPool,
    fixture: &Fixture,
    filter: ExternalAnalyticsFilter,
) -> Value {
    let filter = ExternalAnalyticsFilter {
        from: fixture.date(filter.from),
        to: fixture.date(filter.to),
        route: filter.route,
        cabin: filter.cabin,
    };
    let repository = Arc::new(SqlxExternalAnalyticsRepository::new(runtime.clone()));
    let service = ExternalAnalyticsService::new(repository);
    let result = service
        .summary(
            filter,
            DateTime::parse_from_rfc3339(BOUNDARY_NOW)
                .unwrap()
                .with_timezone(&Utc),
        )
        .await
        .expect("analytics summary");
    let value = serde_json::to_value(result).expect("serialize analytics summary");
    let _ = setup;
    value
}

#[tokio::test]
async fn summary_returns_only_nonfinancial_aggregate_keys() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    run_fixture_body(
        setup.clone(),
        runtime.clone(),
        fixture,
        move |setup, runtime, mut fixture| async move {
            let (_, _) = insert_service(
                &setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 1, 10).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            let payload = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2040-01-10", "2040-01-10"),
            )
            .await;
            let keys = payload
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            assert_eq!(
                keys,
                vec![
                    "bookedSeats",
                    "cancelledBookings",
                    "generatedAt",
                    "occupancyPercent",
                    "period",
                    "sellableSeats",
                    "ticketsIssued",
                    "totalBookings",
                ]
            );
            let expected_date = fixture.date(NaiveDate::from_ymd_opt(2040, 1, 10).unwrap());
            assert_eq!(
                payload["period"],
                serde_json::json!({
                    "from": expected_date.to_string(),
                    "to": expected_date.to_string()
                })
            );
            for forbidden in [
                "revenue",
                "grossRevenue",
                "bookingReference",
                "ticketNumber",
                "providerReference",
                "passenger",
                "payment",
                "refund",
                "internalId",
                "uuid",
            ] {
                assert!(
                    !payload.to_string().contains(forbidden),
                    "forbidden field {forbidden}"
                );
            }
        },
    )
    .await;
}

#[tokio::test]
async fn summary_uses_flight_departure_bangkok_cohort_not_payment_date() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    run_fixture_body(
        setup.clone(),
        runtime.clone(),
        fixture,
        move |setup, runtime, mut fixture| async move {
            let (_, instance) = insert_service(
                &setup,
                &mut fixture,
                "HND",
                "BKK",
                "Asia/Tokyo",
                NaiveDate::from_ymd_opt(2040, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(0, 30, 0).unwrap(),
            )
            .await;
            insert_booking(
                &setup,
                &mut fixture,
                instance,
                "business",
                "STRIPE",
                Some("ISSUED"),
                "2040-01-01T12:00:00Z",
            )
            .await;
            let local_previous_day = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2039-12-31", "2039-12-31"),
            )
            .await;
            let utc_departure_day = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2040-01-01", "2040-01-01"),
            )
            .await;
            assert_eq!(local_previous_day["totalBookings"], 1);
            assert_eq!(utc_departure_day["totalBookings"], 0);
        },
    )
    .await;
}

#[tokio::test]
async fn summary_uses_inclusive_bangkok_dates() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    run_fixture_body(
        setup.clone(),
        runtime.clone(),
        fixture,
        move |setup, runtime, mut fixture| async move {
            for (date, time, expected_cohort) in [
                ("2040-01-01", "00:30:00", "2039-12-31"),
                ("2040-01-01", "02:00:00", "2040-01-01"),
                ("2040-01-01", "02:01:00", "2040-01-01"),
            ] {
                let (_, instance) = insert_service(
                    &setup,
                    &mut fixture,
                    "HND",
                    "BKK",
                    "Asia/Tokyo",
                    NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
                    NaiveTime::parse_from_str(time, "%H:%M:%S").unwrap(),
                )
                .await;
                insert_booking(
                    &setup,
                    &mut fixture,
                    instance,
                    "business",
                    "MOCK_BITCOIN",
                    None,
                    "2040-01-01T00:00:00Z",
                )
                .await;
                assert!(!expected_cohort.is_empty());
            }
            let both_days = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2039-12-31", "2040-01-01"),
            )
            .await;
            let boundary_day = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2040-01-01", "2040-01-01"),
            )
            .await;
            assert_eq!(both_days["totalBookings"], 3);
            assert_eq!(boundary_day["totalBookings"], 2);
        },
    )
    .await;
}

#[tokio::test]
async fn summary_includes_all_supported_successful_providers() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    run_fixture_body(
        setup.clone(),
        runtime.clone(),
        fixture,
        move |setup, runtime, mut fixture| async move {
            let (_, instance) = insert_service(
                &setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 2, 1).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            insert_booking(
                &setup,
                &mut fixture,
                instance,
                "business",
                "STRIPE",
                None,
                "2040-01-01T00:00:00Z",
            )
            .await;
            insert_booking(
                &setup,
                &mut fixture,
                instance,
                "first",
                "MOCK_BITCOIN",
                None,
                "2040-01-01T00:00:00Z",
            )
            .await;
            let payload = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2040-02-01", "2040-02-01"),
            )
            .await;
            assert_eq!(payload["totalBookings"], 2);
        },
    )
    .await;
}

#[tokio::test]
async fn summary_counts_all_cohort_bookings_including_cancelled() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    run_fixture_body(
        setup.clone(),
        runtime.clone(),
        fixture,
        move |setup, runtime, mut fixture| async move {
            let (_, instance) = insert_service(
                &setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 1, 10).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            insert_booking(
                &setup,
                &mut fixture,
                instance,
                "business",
                "STRIPE",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await;
            insert_booking(
                &setup,
                &mut fixture,
                instance,
                "business",
                "MOCK_BITCOIN",
                Some("CANCELLED"),
                "2040-01-01T00:00:00Z",
            )
            .await;
            let payload = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2040-01-10", "2040-01-10"),
            )
            .await;
            assert_eq!(payload["totalBookings"], 2);
            assert_eq!(payload["ticketsIssued"], 2);
            assert_eq!(payload["cancelledBookings"], 1);
        },
    )
    .await;
}

#[tokio::test]
async fn summary_counts_issued_tickets_even_when_booking_cancelled() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    run_fixture_body(
        setup.clone(),
        runtime.clone(),
        fixture,
        move |setup, runtime, mut fixture| async move {
            let (_, instance) = insert_service(
                &setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 1, 10).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            insert_booking(
                &setup,
                &mut fixture,
                instance,
                "business",
                "STRIPE",
                Some("CANCELLED"),
                "2040-01-01T00:00:00Z",
            )
            .await;
            let payload = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2040-01-10", "2040-01-10"),
            )
            .await;
            assert_eq!(payload["ticketsIssued"], 1);
        },
    )
    .await;
}

#[tokio::test]
async fn summary_counts_cancelled_booking_subset() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    run_fixture_body(
        setup.clone(),
        runtime.clone(),
        fixture,
        move |setup, runtime, mut fixture| async move {
            let (_, instance) = insert_service(
                &setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 1, 10).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            insert_booking(
                &setup,
                &mut fixture,
                instance,
                "business",
                "STRIPE",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await;
            insert_booking(
                &setup,
                &mut fixture,
                instance,
                "first",
                "STRIPE",
                Some("CANCELLED"),
                "2040-01-01T00:00:00Z",
            )
            .await;
            let payload = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2040-01-10", "2040-01-10"),
            )
            .await;
            assert_eq!(payload["totalBookings"], 2);
            assert_eq!(payload["cancelledBookings"], 1);
        },
    )
    .await;
}

#[tokio::test]
async fn summary_excludes_cancelled_bookings_from_booked_seats() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    run_fixture_body(
        setup.clone(),
        runtime.clone(),
        fixture,
        move |setup, runtime, mut fixture| async move {
            let (_, instance) = insert_service(
                &setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 1, 10).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            let active_attempt = insert_booking(
                &setup,
                &mut fixture,
                instance,
                "business",
                "STRIPE",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            let cancelled_attempt = insert_booking(
                &setup,
                &mut fixture,
                instance,
                "business",
                "MOCK_BITCOIN",
                Some("CANCELLED"),
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            let active_seat = insert_inventory_seat(
                &setup,
                &mut fixture,
                instance,
                "business",
                "1A",
                true,
                "BOOKED",
            )
            .await;
            let cancelled_seat = insert_inventory_seat(
                &setup,
                &mut fixture,
                instance,
                "business",
                "2A",
                true,
                "BOOKED",
            )
            .await;
            link_booked_seat(&setup, active_attempt, active_seat, None).await;
            link_booked_seat(
                &setup,
                cancelled_attempt,
                cancelled_seat,
                Some("2040-01-04T00:00:00Z"),
            )
            .await;
            let payload = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2040-01-10", "2040-01-10"),
            )
            .await;
            assert_eq!(payload["bookedSeats"], 1);
        },
    )
    .await;
}

#[tokio::test]
async fn summary_excludes_unsuccessful_payment_states() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    run_fixture_body(
        setup.clone(),
        runtime.clone(),
        fixture,
        move |setup, runtime, mut fixture| async move {
            let (_, instance) = insert_service(
                &setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 1, 10).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            let failed_attempt = insert_booking(
                &setup,
                &mut fixture,
                instance,
                "business",
                "STRIPE",
                None,
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            mark_payment_failed(&setup, failed_attempt).await;
            let payload = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2040-01-10", "2040-01-10"),
            )
            .await;
            assert_eq!(payload["totalBookings"], 0);
            assert_eq!(payload["ticketsIssued"], 0);
            assert_eq!(payload["cancelledBookings"], 0);
        },
    )
    .await;
}

#[tokio::test]
async fn summary_computes_occupancy_and_zero_denominator_is_zero_point_zero() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    run_fixture_body(
        setup.clone(),
        runtime.clone(),
        fixture,
        move |setup, runtime, mut fixture| async move {
            let (_, instance) = insert_service(
                &setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 1, 10).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            let attempt = insert_booking(
                &setup,
                &mut fixture,
                instance,
                "business",
                "STRIPE",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            let booked = insert_inventory_seat(
                &setup,
                &mut fixture,
                instance,
                "business",
                "1A",
                true,
                "BOOKED",
            )
            .await;
            link_booked_seat(&setup, attempt, booked, None).await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                instance,
                "business",
                "2A",
                true,
                "AVAILABLE",
            )
            .await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                instance,
                "economy",
                "4A",
                true,
                "AVAILABLE",
            )
            .await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                instance,
                "premium-economy",
                "5A",
                true,
                "AVAILABLE",
            )
            .await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                instance,
                "business",
                "3A",
                true,
                "AVAILABLE",
            )
            .await;
            let payload = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2040-01-10", "2040-01-10"),
            )
            .await;
            assert_eq!(payload["bookedSeats"], 1);
            assert_eq!(payload["sellableSeats"], 3);
            assert_eq!(payload["occupancyPercent"], 33.33);

            let (_, zero_instance) = insert_service(
                &setup,
                &mut fixture,
                "HND",
                "JFK",
                "Asia/Tokyo",
                NaiveDate::from_ymd_opt(2040, 2, 1).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                zero_instance,
                "business",
                "1A",
                false,
                "AVAILABLE",
            )
            .await;
            let zero = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2040-01-10", "2040-01-10"),
            )
            .await;
            assert_eq!(zero["sellableSeats"], 3);
            assert_eq!(zero["occupancyPercent"], 33.33);
            let empty = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2040-02-01", "2040-02-01"),
            )
            .await;
            assert_eq!(empty["sellableSeats"], 0);
            assert_eq!(empty["occupancyPercent"], 0.0);
        },
    )
    .await;
}

#[tokio::test]
async fn route_and_cabin_filters_bound_both_numerators_and_denominator() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    run_fixture_body(
        setup.clone(),
        runtime.clone(),
        fixture,
        move |setup, runtime, mut fixture| async move {
            let (_, bkk_instance) = insert_service(
                &setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 1, 10).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            let bkk_business = insert_booking(
                &setup,
                &mut fixture,
                bkk_instance,
                "business",
                "STRIPE",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            let bkk_first = insert_booking(
                &setup,
                &mut fixture,
                bkk_instance,
                "first",
                "MOCK_BITCOIN",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            let bkk_b = insert_inventory_seat(
                &setup,
                &mut fixture,
                bkk_instance,
                "business",
                "1A",
                true,
                "BOOKED",
            )
            .await;
            let bkk_f = insert_inventory_seat(
                &setup,
                &mut fixture,
                bkk_instance,
                "first",
                "1A",
                true,
                "BOOKED",
            )
            .await;
            link_booked_seat(&setup, bkk_business, bkk_b, None).await;
            link_booked_seat(&setup, bkk_first, bkk_f, None).await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                bkk_instance,
                "business",
                "2A",
                true,
                "AVAILABLE",
            )
            .await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                bkk_instance,
                "first",
                "2A",
                true,
                "AVAILABLE",
            )
            .await;

            let (_, other_instance) = insert_service(
                &setup,
                &mut fixture,
                "HND",
                "JFK",
                "Asia/Tokyo",
                NaiveDate::from_ymd_opt(2040, 1, 10).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            insert_booking(
                &setup,
                &mut fixture,
                other_instance,
                "business",
                "STRIPE",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                other_instance,
                "business",
                "1A",
                true,
                "AVAILABLE",
            )
            .await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                other_instance,
                "business",
                "2A",
                true,
                "AVAILABLE",
            )
            .await;

            let filtered = ExternalAnalyticsFilter::parse(
                Some("2040-01-10"),
                Some("2040-01-10"),
                Some("BKK-DXB"),
                Some("business"),
                DateTime::parse_from_rfc3339(BOUNDARY_NOW)
                    .unwrap()
                    .with_timezone(&Utc),
            )
            .unwrap();
            let payload = summary(&setup, &runtime, &fixture, filtered).await;
            assert_eq!(payload["totalBookings"], 1);
            assert_eq!(payload["ticketsIssued"], 1);
            assert_eq!(payload["cancelledBookings"], 0);
            assert_eq!(payload["bookedSeats"], 1);
            assert_eq!(payload["sellableSeats"], 2);
            assert_eq!(payload["occupancyPercent"], 50.0);
        },
    )
    .await;
}

#[tokio::test]
async fn mixed_cross_metric_fixture_applies_every_filter_and_metric_rule() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    run_fixture_body(
        setup.clone(),
        runtime.clone(),
        fixture,
        move |setup, runtime, mut fixture| async move {
            let (_, primary_day_one) = insert_service(
                &setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 1, 10).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            let active_day_one = insert_booking(
                &setup,
                &mut fixture,
                primary_day_one,
                "business",
                "STRIPE",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            let active_day_one_seat = insert_inventory_seat(
                &setup,
                &mut fixture,
                primary_day_one,
                "business",
                "1A",
                true,
                "BOOKED",
            )
            .await;
            link_booked_seat(&setup, active_day_one, active_day_one_seat, None).await;

            let cancelled_day_one = insert_booking(
                &setup,
                &mut fixture,
                primary_day_one,
                "business",
                "MOCK_BITCOIN",
                Some("CANCELLED"),
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            let cancelled_day_one_seat = insert_inventory_seat(
                &setup,
                &mut fixture,
                primary_day_one,
                "business",
                "2A",
                true,
                "BOOKED",
            )
            .await;
            link_booked_seat(&setup, cancelled_day_one, cancelled_day_one_seat, None).await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                primary_day_one,
                "business",
                "3A",
                true,
                "AVAILABLE",
            )
            .await;

            let failed_day_one = insert_booking(
                &setup,
                &mut fixture,
                primary_day_one,
                "business",
                "STRIPE",
                None,
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            mark_payment_failed(&setup, failed_day_one).await;

            let first_day_one = insert_booking(
                &setup,
                &mut fixture,
                primary_day_one,
                "first",
                "MOCK_BITCOIN",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            let first_day_one_seat = insert_inventory_seat(
                &setup,
                &mut fixture,
                primary_day_one,
                "first",
                "1A",
                true,
                "BOOKED",
            )
            .await;
            link_booked_seat(&setup, first_day_one, first_day_one_seat, None).await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                primary_day_one,
                "first",
                "2A",
                true,
                "AVAILABLE",
            )
            .await;

            let legacy_day_one = insert_booking(
                &setup,
                &mut fixture,
                primary_day_one,
                "economy",
                "STRIPE",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            let legacy_day_one_seat = insert_inventory_seat(
                &setup,
                &mut fixture,
                primary_day_one,
                "economy",
                "1A",
                true,
                "BOOKED",
            )
            .await;
            link_booked_seat(&setup, legacy_day_one, legacy_day_one_seat, None).await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                primary_day_one,
                "premium-economy",
                "1A",
                true,
                "AVAILABLE",
            )
            .await;

            let (_, primary_day_two) = insert_service(
                &setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 1, 11).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            let active_day_two = insert_booking(
                &setup,
                &mut fixture,
                primary_day_two,
                "business",
                "MOCK_BITCOIN",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            let active_day_two_seat = insert_inventory_seat(
                &setup,
                &mut fixture,
                primary_day_two,
                "business",
                "1A",
                true,
                "BOOKED",
            )
            .await;
            link_booked_seat(&setup, active_day_two, active_day_two_seat, None).await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                primary_day_two,
                "business",
                "2A",
                true,
                "AVAILABLE",
            )
            .await;

            let (_, out_of_period) = insert_service(
                &setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 1, 12).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            let out_of_period_attempt = insert_booking(
                &setup,
                &mut fixture,
                out_of_period,
                "business",
                "STRIPE",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            let out_of_period_seat = insert_inventory_seat(
                &setup,
                &mut fixture,
                out_of_period,
                "business",
                "1A",
                true,
                "BOOKED",
            )
            .await;
            link_booked_seat(&setup, out_of_period_attempt, out_of_period_seat, None).await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                out_of_period,
                "business",
                "2A",
                true,
                "AVAILABLE",
            )
            .await;

            let (_, other_route) = insert_service(
                &setup,
                &mut fixture,
                "HND",
                "JFK",
                "Asia/Tokyo",
                NaiveDate::from_ymd_opt(2040, 1, 10).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            let other_route_attempt = insert_booking(
                &setup,
                &mut fixture,
                other_route,
                "business",
                "MOCK_BITCOIN",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            let other_route_seat = insert_inventory_seat(
                &setup,
                &mut fixture,
                other_route,
                "business",
                "1A",
                true,
                "BOOKED",
            )
            .await;
            link_booked_seat(&setup, other_route_attempt, other_route_seat, None).await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                other_route,
                "business",
                "2A",
                true,
                "AVAILABLE",
            )
            .await;

            // Omitting cabin intentionally exercises the locked default BUSINESS+FIRST
            // selection and proves that legacy ECONOMY/PREMIUM_ECONOMY rows stay out.
            let requested = ExternalAnalyticsFilter::parse(
                Some("2040-01-10"),
                Some("2040-01-11"),
                Some("BKK-DXB"),
                None,
                DateTime::parse_from_rfc3339(BOUNDARY_NOW)
                    .unwrap()
                    .with_timezone(&Utc),
            )
            .unwrap();
            let payload = summary(&setup, &runtime, &fixture, requested).await;
            let first_day = fixture.date(NaiveDate::from_ymd_opt(2040, 1, 10).unwrap());
            let second_day = fixture.date(NaiveDate::from_ymd_opt(2040, 1, 11).unwrap());
            assert_eq!(
                payload["period"],
                serde_json::json!({
                    "from": first_day.to_string(),
                    "to": second_day.to_string()
                })
            );
            assert_eq!(payload["totalBookings"], 4);
            assert_eq!(payload["ticketsIssued"], 4);
            assert_eq!(payload["cancelledBookings"], 1);
            assert_eq!(payload["bookedSeats"], 3);
            assert_eq!(payload["sellableSeats"], 7);
            assert_eq!(payload["occupancyPercent"], 42.86);
        },
    )
    .await;
}

#[tokio::test]
async fn summary_preserves_counting_grains_for_multi_seat_booking() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = Fixture::new(&setup).await;
    run_fixture_body(
        setup.clone(),
        runtime.clone(),
        fixture,
        move |setup, runtime, mut fixture| async move {
            let (_, instance) = insert_service(
                &setup,
                &mut fixture,
                "BKK",
                "DXB",
                "Asia/Bangkok",
                NaiveDate::from_ymd_opt(2040, 3, 1).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            let successful_attempt = insert_booking(
                &setup,
                &mut fixture,
                instance,
                "business",
                "STRIPE",
                Some("ISSUED"),
                "2040-01-01T00:00:00Z",
            )
            .await
            .0;
            add_passengers_for_attempt(&setup, successful_attempt, 2).await;
            insert_failed_attempt_for_hold(&setup, &mut fixture, successful_attempt).await;

            let first_seat = insert_inventory_seat(
                &setup,
                &mut fixture,
                instance,
                "business",
                "1A",
                true,
                "BOOKED",
            )
            .await;
            let second_seat = insert_inventory_seat(
                &setup,
                &mut fixture,
                instance,
                "business",
                "2A",
                true,
                "BOOKED",
            )
            .await;
            insert_inventory_seat(
                &setup,
                &mut fixture,
                instance,
                "business",
                "3A",
                true,
                "AVAILABLE",
            )
            .await;
            link_booked_seat_for_passenger(&setup, successful_attempt, first_seat, 1, None).await;
            link_booked_seat_for_passenger(&setup, successful_attempt, second_seat, 2, None).await;

            // tickets.payment_attempt_id is UNIQUE in the current domain: two
            // passengers share one booking's issued ticket while their seats remain
            // separate one-to-many finalized-seat rows.
            let payload = summary(
                &setup,
                &runtime,
                &fixture,
                filter("2040-03-01", "2040-03-01"),
            )
            .await;
            assert_eq!(payload["totalBookings"], 1);
            assert_eq!(payload["ticketsIssued"], 1);
            assert_eq!(payload["cancelledBookings"], 0);
            assert_eq!(payload["bookedSeats"], 2);
            assert_eq!(payload["sellableSeats"], 3);
            assert_eq!(payload["occupancyPercent"], 66.67);
        },
    )
    .await;
}

#[test]
fn analytics_filter_rejects_legacy_cabins_and_invalid_ranges() {
    let now = DateTime::parse_from_rfc3339(BOUNDARY_NOW)
        .unwrap()
        .with_timezone(&Utc);
    assert!(ExternalAnalyticsFilter::parse(
        Some("2040-01-01"),
        Some("2040-01-01"),
        None,
        Some("economy"),
        now
    )
    .is_err());
    assert!(ExternalAnalyticsFilter::parse(
        Some("2040-01-02"),
        Some("2040-01-01"),
        None,
        None,
        now
    )
    .is_err());
    assert!(ExternalAnalyticsFilter::parse(
        Some("2040-01-01"),
        Some("2041-01-02"),
        None,
        None,
        now
    )
    .is_err());
    let parsed = ExternalAnalyticsFilter::parse(
        Some("2040-01-01"),
        Some("2040-01-01"),
        None,
        Some("business"),
        now,
    )
    .unwrap();
    assert_eq!(parsed.cabin, Some(CabinClass::Business));
}
