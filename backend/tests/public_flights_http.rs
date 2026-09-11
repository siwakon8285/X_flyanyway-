mod common;

use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::ServiceExt;
use uuid::Uuid;
use x_fly_api::{
    application::flight::FlightManagement,
    domain::flight::FlightCommand,
    infrastructure::{
        database::{prepare_test_database, SqlxFlightRepository, SqlxSeatHoldRepository},
        http::build_router,
    },
    state::AppState,
};

async fn fixture_guard() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> PgPool {
    let url = common::test_database_url();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .unwrap();
    prepare_test_database(&pool).await.unwrap();
    sqlx::raw_sql(
        "DELETE FROM flight_management_audit WHERE flight_service_id IN (SELECT id FROM flight_services WHERE flight_number='XF 953');
         DELETE FROM flight_service_seat_templates WHERE flight_service_id IN (SELECT id FROM flight_services WHERE flight_number='XF 953');
         DELETE FROM flight_seats WHERE flight_instance_id IN (SELECT instance.id FROM flight_instances instance JOIN flight_services service ON service.id=instance.flight_service_id WHERE service.flight_number='XF 953');
         DELETE FROM seat_holds WHERE flight_instance_id IN (SELECT instance.id FROM flight_instances instance JOIN flight_services service ON service.id=instance.flight_service_id WHERE service.flight_number='XF 953');
         DELETE FROM flight_instances WHERE flight_service_id IN (SELECT id FROM flight_services WHERE flight_number='XF 953');
         DELETE FROM flight_service_cabins WHERE flight_service_id IN (SELECT id FROM flight_services WHERE flight_number='XF 953');
         DELETE FROM flight_services WHERE flight_number='XF 953';
         DELETE FROM staff_user_roles WHERE staff_user_id IN (SELECT id FROM staff_users WHERE email='public-flight@flight-management.test');
         DELETE FROM staff_users WHERE email='public-flight@flight-management.test';",
    ).execute(&pool).await.unwrap();
    pool
}

fn command() -> FlightCommand {
    serde_json::from_value(serde_json::json!({"flightNumber":"XF 953","originCode":"BKK","destinationCode":"DXB","operatingDate":"2026-10-10",
        "departureTime":"09:20:00","arrivalTime":"13:05:00","arrivalDayOffset":0,"aircraftCode":"Boeing 787-9",
        "businessPriceAmount":46900,"firstPriceAmount":78900,"currencyCode":"THB","businessCapacity":16,"firstCapacity":4})).unwrap()
}

async fn fixture(
    pool: &PgPool,
) -> (
    SqlxFlightRepository,
    x_fly_api::domain::flight::FlightRecord,
) {
    let actor: Uuid = sqlx::query_scalar("INSERT INTO staff_users (email,password_hash) VALUES ('public-flight@flight-management.test','hash') RETURNING id")
        .fetch_one(pool).await.unwrap();
    let repository = SqlxFlightRepository::new(pool.clone());
    let flight = repository.create(actor, command()).await.unwrap();
    (repository, flight)
}

fn app(pool: PgPool) -> axum::Router {
    let booking = Arc::new(SqlxSeatHoldRepository::new(pool.clone()));
    build_router(
        AppState::new(
            booking.clone(),
            booking.clone(),
            booking.clone(),
            booking,
            Duration::from_secs(600),
            false,
            "http://localhost:3000".into(),
        )
        .with_flights(FlightManagement::new(Arc::new(SqlxFlightRepository::new(
            pool,
        )))),
    )
}

async fn get(router: &axum::Router, uri: &str) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

#[tokio::test]
async fn public_search_and_detail_use_postgresql_and_hide_cancelled_flights() {
    let _guard = fixture_guard().await;
    let pool = pool().await;
    let (repository, flight) = fixture(&pool).await;
    let router = app(pool);
    let (status, body) = get(
        &router,
        "/api/v1/flights?origin=BKK&destination=DXB&departure=2026-10-10&cabin=business",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let public = body
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == flight.public_id)
        .unwrap();
    assert_eq!(public["cabinPrices"].as_array().unwrap().len(), 2);
    let (detail_status, detail) = get(
        &router,
        &format!(
            "/api/v1/flights/{}?departure=2026-10-10&cabin=business",
            flight.public_id
        ),
    )
    .await;
    assert_eq!(detail_status, StatusCode::OK);
    assert_eq!(detail["flightNumber"], "XF 953");

    let actor: Uuid = sqlx::query_scalar(
        "SELECT id FROM staff_users WHERE email='public-flight@flight-management.test'",
    )
    .fetch_one(repository.pool())
    .await
    .unwrap();
    repository
        .cancel(actor, flight.id, flight.version)
        .await
        .unwrap();
    let (_, hidden) = get(
        &router,
        "/api/v1/flights?origin=BKK&destination=DXB&departure=2026-10-10&cabin=business",
    )
    .await;
    assert!(!hidden
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"] == flight.public_id));
    let (seat_status, _) = get(
        &router,
        &format!(
            "/api/v1/flights/{}/seats?departure=2026-10-10&cabin=business",
            flight.public_id
        ),
    )
    .await;
    assert_eq!(seat_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn public_airport_master_exposes_the_expanded_supported_network() {
    let _guard = fixture_guard().await;
    let pool = pool().await;
    let router = app(pool);
    let (status, body) = get(&router, "/api/v1/airports").await;

    assert_eq!(status, StatusCode::OK);
    let airports = body.as_array().unwrap();
    assert_eq!(airports.len(), 156);
    assert!(airports.iter().any(|airport| {
        airport["code"] == "SYD"
            && airport["city"] == "Sydney (Mascot)"
            && airport["countryCode"] == "AU"
            && airport["countryName"] == "Australia"
            && airport["timeZone"] == "Australia/Sydney"
    }));
}
