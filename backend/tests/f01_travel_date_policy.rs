mod common;

use std::{sync::Arc, time::Duration};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, Utc};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::ServiceExt;
use uuid::Uuid;
use x_fly_api::{
    application::flight::{FlightManagement, FlightRepository, PublicFlightFilter},
    domain::{
        cancellation::Clock,
        entities::{CreateSeatHold, FlightSelection},
        flight::FlightCommand,
        repositories::{SeatHoldRepository, SeatHoldRepositoryError},
        travel_date::CustomerTravelDatePolicy,
        value_objects::{CabinClass, PassengerCounts, SeatNumber},
    },
    infrastructure::{
        database::{prepare_test_database, SqlxFlightRepository, SqlxSeatHoldRepository},
        http::build_router,
    },
    state::AppState,
};

const ORIGIN_LOCAL_TODAY: NaiveDate =
    NaiveDate::from_ymd_opt(2030, 6, 16).expect("valid deterministic origin date");
const YESTERDAY: NaiveDate =
    NaiveDate::from_ymd_opt(2030, 6, 15).expect("valid deterministic yesterday");
const LATEST_ALLOWED: NaiveDate =
    NaiveDate::from_ymd_opt(2031, 6, 16).expect("valid deterministic upper boundary");
const FIRST_REJECTED: NaiveDate =
    NaiveDate::from_ymd_opt(2031, 6, 17).expect("valid deterministic first rejected date");
const RECURRING_FLIGHT_ID: &str = "xf-201";
const ONE_OFF_INSIDE_ID: &str = "xf-961-20300626";
const ONE_OFF_OUTSIDE_ID: &str = "xf-962-20310617";

#[derive(Clone)]
struct FixedClock(DateTime<Utc>);

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.0
    }
}

fn fixed_reference_now() -> DateTime<Utc> {
    // 23:30 UTC is 06:30 on the following day in the seeded BKK origin zone.
    DateTime::parse_from_rfc3339("2030-06-15T23:30:00Z")
        .expect("valid fixed reference instant")
        .with_timezone(&Utc)
}

fn fixed_clock() -> Arc<dyn Clock> {
    Arc::new(FixedClock(fixed_reference_now()))
}

async fn setup_pool() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(6)
        .connect(&common::test_database_url())
        .await
        .expect("connect to dedicated TEST setup database");
    prepare_test_database(&pool)
        .await
        .expect("prepare dedicated TEST database");
    pool
}

async fn runtime_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(6)
        .connect(&common::test_runtime_database_url())
        .await
        .expect("connect to dedicated TEST runtime database")
}

fn seat_repository(pool: PgPool) -> SqlxSeatHoldRepository {
    SqlxSeatHoldRepository::new_with_clock(pool, fixed_clock())
}

fn flight_repository(pool: PgPool) -> SqlxFlightRepository {
    SqlxFlightRepository::new_with_clock(pool, fixed_clock())
}

fn selection(flight_id: &str, departure_date: NaiveDate) -> FlightSelection {
    FlightSelection {
        flight_id: flight_id.to_owned(),
        departure_date,
        cabin: CabinClass::Business,
    }
}

fn hold_command(flight_id: &str, departure_date: NaiveDate) -> CreateSeatHold {
    CreateSeatHold {
        selection: selection(flight_id, departure_date),
        passengers: PassengerCounts::new(1, 0, 0).expect("valid passenger count"),
        seats: vec![SeatNumber::parse("3A").expect("valid seat")],
        token_hash: [91; 32],
    }
}

async fn cleanup_recurring(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(
        "DELETE FROM seat_holds
         WHERE flight_instance_id IN (
             SELECT instance.id FROM flight_instances instance
             JOIN flight_services service ON service.id=instance.flight_service_id
             WHERE service.public_id='xf-201'
               AND instance.departure_date BETWEEN '2030-06-15' AND '2031-06-17'
         );
         DELETE FROM flight_seats
         WHERE flight_instance_id IN (
             SELECT instance.id FROM flight_instances instance
             JOIN flight_services service ON service.id=instance.flight_service_id
             WHERE service.public_id='xf-201'
               AND instance.departure_date BETWEEN '2030-06-15' AND '2031-06-17'
         );
         DELETE FROM flight_instances
         WHERE flight_service_id=(SELECT id FROM flight_services WHERE public_id='xf-201')
           AND departure_date BETWEEN '2030-06-15' AND '2031-06-17';",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn cleanup_one_offs(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(
        "DELETE FROM flight_management_audit
         WHERE flight_service_id IN (
             SELECT id FROM flight_services WHERE public_id IN ('xf-961-20300626','xf-962-20310617')
         );
         DELETE FROM seat_holds
         WHERE flight_instance_id IN (
             SELECT instance.id FROM flight_instances instance
             JOIN flight_services service ON service.id=instance.flight_service_id
             WHERE service.public_id IN ('xf-961-20300626','xf-962-20310617')
         );
         DELETE FROM flight_seats
         WHERE flight_instance_id IN (
             SELECT instance.id FROM flight_instances instance
             JOIN flight_services service ON service.id=instance.flight_service_id
             WHERE service.public_id IN ('xf-961-20300626','xf-962-20310617')
         );
         DELETE FROM flight_instances
         WHERE flight_service_id IN (
             SELECT id FROM flight_services WHERE public_id IN ('xf-961-20300626','xf-962-20310617')
         );
         DELETE FROM flight_service_seat_templates
         WHERE flight_service_id IN (
             SELECT id FROM flight_services WHERE public_id IN ('xf-961-20300626','xf-962-20310617')
         );
         DELETE FROM flight_service_cabins
         WHERE flight_service_id IN (
             SELECT id FROM flight_services WHERE public_id IN ('xf-961-20300626','xf-962-20310617')
         );
         DELETE FROM flight_services
         WHERE public_id IN ('xf-961-20300626','xf-962-20310617');
         DELETE FROM staff_users WHERE email='f01-fixture@x-fly.test';",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn persistent_counts(pool: &PgPool, flight_id: &str, date: NaiveDate) -> (i64, i64, i64) {
    let instance_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM flight_instances instance
         JOIN flight_services service ON service.id=instance.flight_service_id
         WHERE service.public_id=$1 AND instance.departure_date=$2",
    )
    .bind(flight_id)
    .bind(date)
    .fetch_one(pool)
    .await
    .expect("inspect TEST flight instance state");
    let seat_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM flight_seats seat
         JOIN flight_instances instance ON instance.id=seat.flight_instance_id
         JOIN flight_services service ON service.id=instance.flight_service_id
         WHERE service.public_id=$1 AND instance.departure_date=$2",
    )
    .bind(flight_id)
    .bind(date)
    .fetch_one(pool)
    .await
    .expect("inspect TEST flight seat state");
    let hold_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM seat_holds hold
         JOIN flight_instances instance ON instance.id=hold.flight_instance_id
         JOIN flight_services service ON service.id=instance.flight_service_id
         WHERE service.public_id=$1 AND instance.departure_date=$2",
    )
    .bind(flight_id)
    .bind(date)
    .fetch_one(pool)
    .await
    .expect("inspect TEST seat hold state");
    (instance_count, seat_count, hold_count)
}

fn one_off_command(flight_number: &str, operating_date: NaiveDate) -> FlightCommand {
    FlightCommand {
        flight_number: flight_number.to_owned(),
        origin_code: "BKK".to_owned(),
        destination_code: "DXB".to_owned(),
        operating_date: Some(operating_date),
        departure_time: chrono::NaiveTime::from_hms_opt(9, 20, 0).expect("valid departure time"),
        arrival_time: chrono::NaiveTime::from_hms_opt(13, 5, 0).expect("valid arrival time"),
        arrival_day_offset: 0,
        aircraft_code: "Boeing 787-9".to_owned(),
        business_price_amount: 46_900,
        first_price_amount: 78_900,
        currency_code: "THB".to_owned(),
        business_capacity: 16,
        first_capacity: 4,
        modeled_operating_cost_amount: None,
    }
}

async fn create_one_off_services(pool: &PgPool) -> (NaiveDate, NaiveDate) {
    let actor: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users(email,password_hash) VALUES('f01-fixture@x-fly.test','hash') RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("create F01 fixture staff actor");
    let inside_date = ORIGIN_LOCAL_TODAY + ChronoDuration::days(10);
    let outside_date = FIRST_REJECTED;
    let repository = SqlxFlightRepository::new(pool.clone());
    repository
        .create(actor, one_off_command("XF 961", inside_date))
        .await
        .expect("create F01 inside-window one-off service");
    repository
        .create(actor, one_off_command("XF 962", outside_date))
        .await
        .expect("create F01 outside-window one-off service");
    (inside_date, outside_date)
}

fn app(seat_pool: PgPool, flight_pool: PgPool) -> Router {
    let clock = fixed_clock();
    let booking = Arc::new(SqlxSeatHoldRepository::new_with_clock(
        seat_pool,
        clock.clone(),
    ));
    let flights = SqlxFlightRepository::new_with_clock(flight_pool, clock);
    build_router(
        AppState::new(
            booking.clone(),
            booking.clone(),
            booking.clone(),
            booking,
            Duration::from_secs(600),
            false,
            "http://localhost:3000".to_owned(),
        )
        .with_flights(FlightManagement::new(Arc::new(flights))),
    )
}

async fn get(router: &Router, uri: &str) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .expect("execute TEST HTTP request");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read TEST HTTP response")
        .to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn post_hold(router: &Router, flight_id: &str, date: NaiveDate) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/seat-holds")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "flightId": flight_id,
                        "departureDate": date,
                        "cabin": "business",
                        "passengers": { "adults": 1, "children": 0, "infants": 0 },
                        "seats": ["3A"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("execute TEST hold request");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read TEST hold response")
        .to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[test]
fn customer_travel_window_is_inclusive_and_origin_local() {
    assert!(!CustomerTravelDatePolicy::is_supported(
        YESTERDAY,
        ORIGIN_LOCAL_TODAY
    ));
    assert!(CustomerTravelDatePolicy::is_supported(
        ORIGIN_LOCAL_TODAY,
        ORIGIN_LOCAL_TODAY
    ));
    assert!(CustomerTravelDatePolicy::is_supported(
        LATEST_ALLOWED,
        ORIGIN_LOCAL_TODAY
    ));
    assert!(!CustomerTravelDatePolicy::is_supported(
        FIRST_REJECTED,
        ORIGIN_LOCAL_TODAY
    ));
}

#[tokio::test]
async fn recurring_service_enforces_origin_local_boundaries_before_materialization() {
    let _guard = common::acquire_test_fixture_lock().await;
    let setup = setup_pool().await;
    cleanup_recurring(&setup)
        .await
        .expect("clean F01 recurring fixture");
    let runtime = runtime_pool().await;
    let repository = seat_repository(runtime);

    for (date, accepted) in [
        (YESTERDAY, false),
        (ORIGIN_LOCAL_TODAY, true),
        (LATEST_ALLOWED, true),
        (FIRST_REJECTED, false),
    ] {
        let result = repository
            .seat_map(&selection(RECURRING_FLIGHT_ID, date), None)
            .await;
        let counts = persistent_counts(&setup, RECURRING_FLIGHT_ID, date).await;
        if accepted {
            assert!(
                result.is_ok(),
                "supported date {date} must return a seat map"
            );
            assert!(counts.0 > 0 && counts.1 > 0 && counts.2 == 0);
        } else {
            assert!(
                matches!(
                    result,
                    Err(SeatHoldRepositoryError::TravelDateOutsideWindow)
                ),
                "unsupported date {date} must fail with the date-window error"
            );
            assert_eq!(counts, (0, 0, 0), "rejected date {date} allocated state");
        }
    }

    cleanup_recurring(&setup)
        .await
        .expect("clean F01 recurring fixture");
}

#[tokio::test]
async fn origin_timezone_boundary_uses_bkk_local_date_instead_of_utc_date() {
    let _guard = common::acquire_test_fixture_lock().await;
    let setup = setup_pool().await;
    cleanup_recurring(&setup)
        .await
        .expect("clean F01 timezone fixture");
    let runtime = runtime_pool().await;
    let repository = seat_repository(runtime);

    // The fixed instant is June 15 UTC but June 16 in Asia/Bangkok. June 15
    // must therefore be rejected as yesterday while June 16 is accepted today.
    let yesterday = repository
        .seat_map(&selection(RECURRING_FLIGHT_ID, YESTERDAY), None)
        .await;
    assert!(matches!(
        yesterday,
        Err(SeatHoldRepositoryError::TravelDateOutsideWindow)
    ));
    let today = repository
        .seat_map(&selection(RECURRING_FLIGHT_ID, ORIGIN_LOCAL_TODAY), None)
        .await;
    assert!(today.is_ok());
    assert_eq!(
        persistent_counts(&setup, RECURRING_FLIGHT_ID, YESTERDAY).await,
        (0, 0, 0)
    );
    assert!(
        persistent_counts(&setup, RECURRING_FLIGHT_ID, ORIGIN_LOCAL_TODAY)
            .await
            .0
            > 0
    );

    cleanup_recurring(&setup)
        .await
        .expect("clean F01 timezone fixture");
}

#[tokio::test]
async fn one_off_services_follow_the_same_window_and_holds_cannot_bypass_it() {
    let _guard = common::acquire_test_fixture_lock().await;
    let setup = setup_pool().await;
    cleanup_one_offs(&setup)
        .await
        .expect("clean F01 one-off fixture");
    let runtime = runtime_pool().await;
    let repository = seat_repository(runtime.clone());
    let (inside_date, outside_date) = create_one_off_services(&setup).await;

    let inside = repository
        .seat_map(&selection(ONE_OFF_INSIDE_ID, inside_date), None)
        .await;
    assert!(
        inside.is_ok(),
        "inside-window one-off service must remain usable: {inside:?}"
    );
    assert!(
        persistent_counts(&setup, ONE_OFF_INSIDE_ID, inside_date)
            .await
            .1
            > 0
    );

    let outside = repository
        .seat_map(&selection(ONE_OFF_OUTSIDE_ID, outside_date), None)
        .await;
    assert!(matches!(
        outside,
        Err(SeatHoldRepositoryError::TravelDateOutsideWindow)
    ));
    assert_eq!(
        persistent_counts(&setup, ONE_OFF_OUTSIDE_ID, outside_date).await,
        (0, 0, 0)
    );

    let hold = repository
        .create_hold(
            hold_command(ONE_OFF_OUTSIDE_ID, outside_date),
            Duration::from_secs(600),
        )
        .await;
    assert!(matches!(
        hold,
        Err(SeatHoldRepositoryError::TravelDateOutsideWindow)
    ));
    assert_eq!(
        persistent_counts(&setup, ONE_OFF_OUTSIDE_ID, outside_date).await,
        (0, 0, 0),
        "rejected repository hold allocated inventory or hold state"
    );

    let (status, _) = post_hold(
        &app(runtime.clone(), runtime.clone()),
        ONE_OFF_OUTSIDE_ID,
        outside_date,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        persistent_counts(&setup, ONE_OFF_OUTSIDE_ID, outside_date).await,
        (0, 0, 0),
        "rejected hold allocated inventory or hold state"
    );

    cleanup_one_offs(&setup)
        .await
        .expect("clean F01 one-off fixture");
}

#[tokio::test]
async fn public_search_and_detail_reject_the_same_dates_as_seat_map() {
    let _guard = common::acquire_test_fixture_lock().await;
    let setup = setup_pool().await;
    cleanup_recurring(&setup)
        .await
        .expect("clean F01 public fixture");
    let runtime = runtime_pool().await;
    let repository = flight_repository(runtime);

    let inside = repository
        .search_public(PublicFlightFilter {
            origin: "BKK".to_owned(),
            destination: "LHR".to_owned(),
            departure: ORIGIN_LOCAL_TODAY,
            cabin: CabinClass::Business,
        })
        .await;
    assert!(inside.is_ok());
    assert!(repository
        .public_detail(
            RECURRING_FLIGHT_ID,
            ORIGIN_LOCAL_TODAY,
            CabinClass::Business
        )
        .await
        .is_ok());

    assert!(matches!(
        repository
            .search_public(PublicFlightFilter {
                origin: "BKK".to_owned(),
                destination: "LHR".to_owned(),
                departure: FIRST_REJECTED,
                cabin: CabinClass::Business,
            })
            .await,
        Err(x_fly_api::domain::flight::FlightManagementError::TravelDateOutsideWindow)
    ));
    assert!(matches!(
        repository
            .public_detail(RECURRING_FLIGHT_ID, FIRST_REJECTED, CabinClass::Business)
            .await,
        Err(x_fly_api::domain::flight::FlightManagementError::TravelDateOutsideWindow)
    ));

    cleanup_recurring(&setup)
        .await
        .expect("clean F01 public fixture");
}

#[tokio::test]
async fn public_http_contract_rejects_date_before_any_inventory_write() {
    let _guard = common::acquire_test_fixture_lock().await;
    let setup = setup_pool().await;
    cleanup_recurring(&setup)
        .await
        .expect("clean F01 HTTP fixture");
    let runtime = runtime_pool().await;
    let router = app(runtime.clone(), runtime);

    let (search_status, search_body) = get(
        &router,
        "/api/v1/flights?origin=BKK&destination=LHR&departure=2031-06-17&cabin=business",
    )
    .await;
    assert_eq!(search_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(search_body["error"]["code"], "FLIGHT_SEARCH_INVALID");

    let (detail_status, detail_body) = get(
        &router,
        "/api/v1/flights/xf-201?departure=2031-06-17&cabin=business",
    )
    .await;
    assert_eq!(detail_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(detail_body["error"]["code"], "FLIGHT_SEARCH_INVALID");

    let (seat_status, seat_body) = get(
        &router,
        "/api/v1/flights/xf-201/seats?departure=2031-06-17&cabin=business",
    )
    .await;
    assert_eq!(seat_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(seat_body["error"]["code"], "TRAVEL_DATE_UNAVAILABLE");

    let (hold_status, hold_body) = post_hold(&router, RECURRING_FLIGHT_ID, FIRST_REJECTED).await;
    assert_eq!(hold_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(hold_body["error"]["code"], "TRAVEL_DATE_UNAVAILABLE");
    assert_eq!(
        persistent_counts(&setup, RECURRING_FLIGHT_ID, FIRST_REJECTED).await,
        (0, 0, 0)
    );

    cleanup_recurring(&setup)
        .await
        .expect("clean F01 HTTP fixture");
}
