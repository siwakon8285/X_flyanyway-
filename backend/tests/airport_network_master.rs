mod common;

use chrono::{NaiveDate, NaiveTime};
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;
use x_fly_api::{
    application::flight::{FlightRepository, PublicFlightFilter},
    domain::{flight::FlightCommand, value_objects::CabinClass},
    infrastructure::database::{prepare_test_database, SqlxFlightRepository},
};

async fn test_pool() -> PgPool {
    let url = common::test_database_url();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .unwrap();
    prepare_test_database(&pool).await.unwrap();
    pool
}

async fn clean_network_flight(pool: &PgPool) {
    sqlx::raw_sql(
        "DELETE FROM flight_management_audit WHERE flight_service_id IN (SELECT id FROM flight_services WHERE origin_code='SYD' AND destination_code='CDG' AND operating_date='2098-05-10');
         DELETE FROM flight_service_seat_templates WHERE flight_service_id IN (SELECT id FROM flight_services WHERE origin_code='SYD' AND destination_code='CDG' AND operating_date='2098-05-10');
         DELETE FROM flight_service_cabins WHERE flight_service_id IN (SELECT id FROM flight_services WHERE origin_code='SYD' AND destination_code='CDG' AND operating_date='2098-05-10');
         DELETE FROM flight_services WHERE origin_code='SYD' AND destination_code='CDG' AND operating_date='2098-05-10';
         DELETE FROM staff_users WHERE email LIKE 'network-%@x-fly.test';",
    )
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn authoritative_network_has_exactly_156_unique_countries_and_primary_airports() {
    let pool = test_pool().await;
    let counts: (i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT
            (SELECT COUNT(*) FROM supported_countries),
            (SELECT COUNT(DISTINCT code) FROM supported_countries),
            (SELECT COUNT(*) FROM airports),
            (SELECT COUNT(*) FROM airports WHERE is_primary),
            (SELECT COUNT(DISTINCT country_code) FROM airports WHERE is_primary)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(counts, (156, 156, 156, 156, 156));

    let invalid: (i64, i64, i64) = sqlx::query_as(
        "SELECT
            COUNT(*) FILTER (WHERE country.code !~ '^[A-Z]{2}$'),
            COUNT(*) FILTER (WHERE airport.code !~ '^[A-Z]{3}$'),
            COUNT(*) FILTER (WHERE timezone.name IS NULL)
         FROM supported_countries country
         JOIN airports airport ON airport.country_code=country.code AND airport.is_primary
         LEFT JOIN pg_timezone_names timezone ON timezone.name=airport.time_zone",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(invalid, (0, 0, 0));
}

#[tokio::test]
async fn expanded_airports_are_shared_by_management_and_truthful_public_search() {
    let pool = test_pool().await;
    clean_network_flight(&pool).await;
    let repository = SqlxFlightRepository::new(pool.clone());
    let references = repository.reference_data().await.unwrap();
    assert_eq!(references.airports.len(), 156);
    assert!(references.airports.iter().any(|airport| {
        airport.code == "SYD"
            && airport.country_code == "AU"
            && airport.time_zone == "Australia/Sydney"
    }));
    assert!(references
        .airports
        .iter()
        .any(|airport| { airport.code == "CDG" && airport.country_code == "FR" }));

    let actor: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash) VALUES ('network-master@x-fly.test','test-hash') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let result = repository
        .create(
            actor,
            FlightCommand {
                flight_number: "XF 959".to_owned(),
                origin_code: "SYD".to_owned(),
                destination_code: "CDG".to_owned(),
                operating_date: NaiveDate::from_ymd_opt(2098, 5, 10),
                departure_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                arrival_time: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
                arrival_day_offset: 0,
                aircraft_code: "Boeing 787-9".to_owned(),
                business_price_amount: 90_000,
                first_price_amount: 140_000,
                currency_code: "THB".to_owned(),
                business_capacity: 16,
                first_capacity: 4,
            },
        )
        .await
        .unwrap();
    assert_eq!(result.origin_time_zone, "Australia/Sydney");
    assert_eq!(result.destination_time_zone, "Europe/Paris");

    let no_service = repository
        .search_public(PublicFlightFilter {
            origin: "SYD".to_owned(),
            destination: "LHR".to_owned(),
            departure: NaiveDate::from_ymd_opt(2098, 5, 10).unwrap(),
            cabin: CabinClass::Business,
        })
        .await
        .unwrap();
    assert!(no_service.is_empty());

    let scheduled = repository
        .search_public(PublicFlightFilter {
            origin: "SYD".to_owned(),
            destination: "CDG".to_owned(),
            departure: NaiveDate::from_ymd_opt(2098, 5, 10).unwrap(),
            cabin: CabinClass::First,
        })
        .await
        .unwrap();
    assert_eq!(scheduled.len(), 1);
    assert_eq!(scheduled[0].cabin_prices.len(), 2);
    assert!(scheduled[0]
        .cabin_prices
        .iter()
        .all(|price| matches!(price.cabin.as_str(), "business" | "first")));
    clean_network_flight(&pool).await;
}
