mod common;

use std::{sync::OnceLock, time::Duration};

use chrono::{NaiveDate, NaiveTime};
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;
use x_fly_api::{
    domain::{
        entities::{CreateSeatHold, FlightSelection},
        flight::{FlightCommand, FlightManagementError, FlightStatus},
        payment::{PaymentAttemptTransition, PaymentStatus},
        repositories::{PaymentRepository, PaymentRepositoryError, SeatHoldRepository},
        value_objects::{CabinClass, PassengerCounts, SeatNumber},
    },
    infrastructure::database::{
        prepare_test_database, SqlxFlightRepository, SqlxSeatHoldRepository,
    },
};

async fn fixture_guard() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn test_pool() -> PgPool {
    let database_url = common::test_database_url();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();
    prepare_test_database(&pool).await.unwrap();
    sqlx::raw_sql(
        "DELETE FROM flight_management_audit WHERE flight_service_id IN
             (SELECT id FROM flight_services WHERE flight_number IN ('XF 951','XF 954'))
             OR actor_staff_user_id IN (SELECT id FROM staff_users WHERE email LIKE 'repo-%@flight-management.test');
         DELETE FROM tickets WHERE payment_attempt_id IN (SELECT attempt.id FROM payment_attempts attempt JOIN seat_holds hold ON hold.id=attempt.seat_hold_id JOIN flight_instances instance ON instance.id=hold.flight_instance_id JOIN flight_services service ON service.id=instance.flight_service_id WHERE service.flight_number IN ('XF 951','XF 954'));
         DELETE FROM payment_attempt_seats WHERE payment_attempt_id IN (SELECT attempt.id FROM payment_attempts attempt JOIN seat_holds hold ON hold.id=attempt.seat_hold_id JOIN flight_instances instance ON instance.id=hold.flight_instance_id JOIN flight_services service ON service.id=instance.flight_service_id WHERE service.flight_number IN ('XF 951','XF 954'));
         DELETE FROM payment_attempts WHERE seat_hold_id IN (SELECT hold.id FROM seat_holds hold JOIN flight_instances instance ON instance.id=hold.flight_instance_id JOIN flight_services service ON service.id=instance.flight_service_id WHERE service.flight_number IN ('XF 951','XF 954'));
         DELETE FROM flight_seats WHERE flight_instance_id IN (SELECT instance.id FROM flight_instances instance JOIN flight_services service ON service.id=instance.flight_service_id WHERE service.flight_number IN ('XF 951','XF 954'));
         DELETE FROM seat_holds WHERE flight_instance_id IN (SELECT instance.id FROM flight_instances instance JOIN flight_services service ON service.id=instance.flight_service_id WHERE service.flight_number IN ('XF 951','XF 954'));
         DELETE FROM flight_service_seat_templates WHERE flight_service_id IN
             (SELECT id FROM flight_services WHERE flight_number IN ('XF 951','XF 954'));
         DELETE FROM flight_instances WHERE flight_service_id IN
             (SELECT id FROM flight_services WHERE flight_number IN ('XF 951','XF 954'));
         DELETE FROM flight_service_cabins WHERE flight_service_id IN
             (SELECT id FROM flight_services WHERE flight_number IN ('XF 951','XF 954'));
         DELETE FROM flight_services WHERE flight_number IN ('XF 951','XF 954');
         DELETE FROM staff_sessions;
         DELETE FROM staff_user_roles;
         DELETE FROM staff_users WHERE email LIKE 'repo-%@flight-management.test';",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

#[tokio::test]
async fn cancellation_blocks_a_preexisting_hold_from_finalizing_payment() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let actor = actor(&pool).await;
    let flights = SqlxFlightRepository::new(pool.clone());
    let mut input = command("XF 954");
    input.operating_date = NaiveDate::from_ymd_opt(2026, 10, 11);
    let flight = flights.create(actor, input).await.unwrap();
    let booking = SqlxSeatHoldRepository::new(pool.clone());
    let token = [42_u8; 32];
    let hold = booking
        .create_hold(
            CreateSeatHold {
                selection: FlightSelection {
                    flight_id: flight.public_id.clone(),
                    departure_date: flight.operating_date.unwrap(),
                    cabin: CabinClass::Business,
                },
                passengers: PassengerCounts::new(1, 0, 0).unwrap(),
                seats: vec![SeatNumber::parse("3A").unwrap()],
                token_hash: token,
            },
            Duration::from_secs(600),
        )
        .await
        .unwrap();
    let attempt: Uuid = sqlx::query_scalar(
        "INSERT INTO payment_attempts (seat_hold_id,request_id,request_fingerprint,provider,payment_method,status,amount,currency_code,review_priced_at)
         VALUES ($1,$2,$3,'MOCK_BITCOIN','BITCOIN','AWAITING_PAYMENT',46900,'THB',NOW()) RETURNING id",
    ).bind(hold.id).bind(Uuid::new_v4()).bind([7_u8;32].as_slice()).fetch_one(&pool).await.unwrap();
    flights
        .cancel(actor, flight.id, flight.version)
        .await
        .unwrap();
    let result = booking
        .transition_payment_attempt(
            hold.id,
            token,
            attempt,
            PaymentAttemptTransition {
                status: PaymentStatus::Succeeded,
                provider_reference: Some("btc-test".into()),
                failure: None,
            },
        )
        .await;
    assert!(
        matches!(result, Err(PaymentRepositoryError::FlightUnavailable)),
        "{result:?}"
    );
    let state: (String, Option<String>) =
        sqlx::query_as("SELECT status,failure_code FROM payment_attempts WHERE id=$1")
            .bind(attempt)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        state,
        ("FAILED".to_owned(), Some("FLIGHT_CANCELLED".to_owned()))
    );
    let ticket_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM tickets WHERE payment_attempt_id=$1")
            .bind(attempt)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(ticket_count, 0);
}

async fn actor(pool: &PgPool) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO staff_users (email, password_hash) VALUES ($1, 'test-hash') RETURNING id",
    )
    .bind(format!("repo-{}@flight-management.test", Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .unwrap()
}

fn command(number: &str) -> FlightCommand {
    FlightCommand {
        flight_number: number.to_owned(),
        origin_code: "BKK".to_owned(),
        destination_code: "DXB".to_owned(),
        operating_date: NaiveDate::from_ymd_opt(2026, 10, 8),
        departure_time: NaiveTime::from_hms_opt(9, 20, 0).unwrap(),
        arrival_time: NaiveTime::from_hms_opt(13, 5, 0).unwrap(),
        arrival_day_offset: 0,
        aircraft_code: "Boeing 787-9".to_owned(),
        business_price_amount: 46_900,
        first_price_amount: 78_900,
        currency_code: "THB".to_owned(),
        business_capacity: 16,
        first_capacity: 4,
    }
}

#[tokio::test]
async fn creates_updates_and_cancels_audited_business_first_inventory() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let actor = actor(&pool).await;
    let repository = SqlxFlightRepository::new(pool.clone());

    let created = repository.create(actor, command("XF 951")).await.unwrap();
    assert_eq!(created.flight_number, "XF 951");
    assert_eq!(created.status, FlightStatus::Scheduled);
    assert_eq!(created.business.capacity, 16);
    assert_eq!(created.first.capacity, 4);
    assert_eq!(created.version, 1);
    let cabins: Vec<String> = sqlx::query_scalar(
        "SELECT cabin FROM flight_service_cabins WHERE flight_service_id=$1 ORDER BY cabin",
    )
    .bind(created.id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(cabins, vec!["business", "first"]);

    let mut repriced = command("XF 951");
    repriced.business_price_amount = 50_000;
    let updated = repository
        .update(actor, created.id, created.version, repriced)
        .await
        .unwrap();
    assert_eq!(updated.business.price_amount, Some(50_000));
    assert_eq!(updated.version, 2);
    assert_eq!(
        repository
            .update(actor, created.id, 1, command("XF 951"))
            .await
            .unwrap_err(),
        FlightManagementError::Conflict
    );

    sqlx::query("INSERT INTO flight_instances (flight_service_id, departure_date) VALUES ($1, $2)")
        .bind(created.id)
        .bind(created.operating_date)
        .execute(&pool)
        .await
        .unwrap();
    let mut structural = command("XF 951");
    structural.aircraft_code = "Airbus A350-900".to_owned();
    assert_eq!(
        repository
            .update(actor, created.id, updated.version, structural)
            .await
            .unwrap_err(),
        FlightManagementError::StructuralConflict
    );

    let instance_id: Uuid =
        sqlx::query_scalar("SELECT id FROM flight_instances WHERE flight_service_id=$1")
            .bind(created.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let historical_hold: Uuid = sqlx::query_scalar(
        "INSERT INTO seat_holds (flight_instance_id,cabin,adults,children,infants,access_token_hash,expires_at,consumed_at)
         VALUES ($1,'business',1,0,0,$2,NOW()+INTERVAL '1 hour',NOW()) RETURNING id",
    )
    .bind(instance_id)
    .bind([9_u8; 32].as_slice())
    .fetch_one(&pool)
    .await
    .unwrap();
    let paid_attempt: Uuid = sqlx::query_scalar(
        "INSERT INTO payment_attempts (seat_hold_id,request_id,request_fingerprint,provider,payment_method,status,amount,currency_code,review_priced_at,succeeded_at)
         VALUES ($1,$2,$3,'MOCK_BITCOIN','BITCOIN','SUCCEEDED',50000,'THB',NOW(),NOW()) RETURNING id",
    )
    .bind(historical_hold)
    .bind(Uuid::new_v4())
    .bind([10_u8; 32].as_slice())
    .fetch_one(&pool)
    .await
    .unwrap();
    let historical_ticket: Uuid = sqlx::query_scalar(
        "INSERT INTO tickets (payment_attempt_id,booking_reference,ticket_number,status)
         VALUES ($1,'XFABCDEFGH','XFTABCDEFGHIJKL','ISSUED') RETURNING id",
    )
    .bind(paid_attempt)
    .fetch_one(&pool)
    .await
    .unwrap();
    let mut future_price = command("XF 951");
    future_price.business_price_amount = 52_000;
    let repriced = repository
        .update(actor, created.id, updated.version, future_price)
        .await
        .unwrap();
    let historical_amount: i64 =
        sqlx::query_scalar("SELECT amount FROM payment_attempts WHERE id=$1")
            .bind(paid_attempt)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(historical_amount, 50_000);
    assert_eq!(repriced.business.price_amount, Some(52_000));

    let cancelled = repository
        .cancel(actor, created.id, repriced.version)
        .await
        .unwrap();
    assert_eq!(cancelled.status, FlightStatus::Cancelled);
    assert_eq!(cancelled.version, 4);
    let preserved: (String, String, i64) = sqlx::query_as(
        "SELECT ticket.status,attempt.status,
            (SELECT COUNT(*) FROM booking_cancellations WHERE ticket_id=ticket.id)
         FROM tickets ticket JOIN payment_attempts attempt ON attempt.id=ticket.payment_attempt_id
         WHERE ticket.id=$1",
    )
    .bind(historical_ticket)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(preserved, ("ISSUED".to_owned(), "SUCCEEDED".to_owned(), 0));
    let persisted: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM flight_services WHERE id=$1)")
            .bind(created.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(persisted);
    let actions: Vec<String> = sqlx::query_scalar(
        "SELECT action FROM flight_management_audit WHERE flight_service_id=$1 ORDER BY created_at",
    )
    .bind(created.id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        actions,
        vec![
            "FLIGHT_CREATED",
            "FLIGHT_EDITED",
            "FLIGHT_EDITED",
            "FLIGHT_CANCELLED"
        ]
    );
    let audit_actor: Uuid = sqlx::query_scalar(
        "SELECT actor_staff_user_id FROM flight_management_audit WHERE flight_service_id=$1 LIMIT 1",
    )
    .bind(created.id).fetch_one(&pool).await.unwrap();
    assert_eq!(audit_actor, actor);
}

#[tokio::test]
async fn rejects_unknown_airports_and_arrival_before_departure_in_authoritative_zones() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let actor = actor(&pool).await;
    let repository = SqlxFlightRepository::new(pool);
    let mut unknown = command("XF 952");
    unknown.destination_code = "ZZZ".to_owned();
    assert_eq!(
        repository.create(actor, unknown).await.unwrap_err(),
        FlightManagementError::Validation
    );

    let mut unknown_aircraft = command("XF 951");
    unknown_aircraft.aircraft_code = "Imaginary Aircraft".to_owned();
    assert_eq!(
        repository
            .create(actor, unknown_aircraft)
            .await
            .unwrap_err(),
        FlightManagementError::Validation
    );

    let mut backwards = command("XF 953");
    backwards.destination_code = "BKK".to_owned();
    backwards.origin_code = "DXB".to_owned();
    backwards.departure_time = NaiveTime::from_hms_opt(23, 0, 0).unwrap();
    backwards.arrival_time = NaiveTime::from_hms_opt(1, 0, 0).unwrap();
    backwards.arrival_day_offset = 0;
    assert_eq!(
        repository.create(actor, backwards).await.unwrap_err(),
        FlightManagementError::Validation
    );
}
