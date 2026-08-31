use std::{env, sync::Arc, time::Duration};

use chrono::{Duration as ChronoDuration, NaiveDate};
use sqlx::{PgPool, Row};
use tokio::sync::Barrier;

use x_fly_api::domain::{
    entities::{CreateSeatHold, FlightSelection},
    repositories::{SeatHoldRepository, SeatHoldRepositoryError},
    value_objects::{CabinClass, PassengerCounts, SeatNumber},
};
use x_fly_api::infrastructure::database::{prepare_database, SqlxSeatHoldRepository};

async fn test_pool() -> PgPool {
    dotenvy::dotenv().ok();
    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must point to an isolated PostgreSQL test database");
    let database_name = database_url.rsplit('/').next().unwrap_or_default();
    assert!(
        database_name.ends_with("_test"),
        "refusing to run repository tests outside a *_test database"
    );

    let pool = PgPool::connect(&database_url)
        .await
        .expect("connect to isolated test PostgreSQL");
    prepare_database(&pool)
        .await
        .expect("apply clean migrations and reference inventory");
    pool
}

fn test_date() -> NaiveDate {
    let offset = (uuid::Uuid::new_v4().as_u128() % 100_000) as i64;
    NaiveDate::from_ymd_opt(2100, 1, 1).unwrap() + ChronoDuration::days(offset)
}

fn selection(flight_id: &str, departure_date: NaiveDate) -> FlightSelection {
    FlightSelection {
        flight_id: flight_id.to_owned(),
        departure_date,
        cabin: CabinClass::Economy,
    }
}

fn seats(values: &[&str]) -> Vec<SeatNumber> {
    values
        .iter()
        .map(|value| SeatNumber::parse(value).unwrap())
        .collect()
}

fn command(
    flight_id: &str,
    departure_date: NaiveDate,
    seat_values: &[&str],
    token_byte: u8,
) -> CreateSeatHold {
    CreateSeatHold {
        selection: selection(flight_id, departure_date),
        passengers: PassengerCounts::new(1, seat_values.len().saturating_sub(1) as u8, 0).unwrap(),
        seats: seats(seat_values),
        token_hash: [token_byte; 32],
    }
}

#[tokio::test]
async fn holds_one_or_multiple_available_seats_and_owner_can_revalidate() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let single_date = test_date();
    let multiple_date = test_date();

    let single = repository
        .create_hold(
            command("xf-201", single_date, &["20A"], 1),
            Duration::from_secs(600),
        )
        .await
        .unwrap();
    assert_eq!(single.seats, seats(&["20A"]));

    let multiple = repository
        .create_hold(
            command("xf-201", multiple_date, &["20A", "20B"], 2),
            Duration::from_secs(600),
        )
        .await
        .unwrap();
    assert_eq!(multiple.seats, seats(&["20A", "20B"]));

    let revalidated = repository.get_hold(multiple.id, [2; 32]).await.unwrap();
    assert_eq!(revalidated.seats, multiple.seats);
    assert!(revalidated.expires_at > revalidated.server_time);
}

#[tokio::test]
async fn rejects_nonexistent_wrong_flight_and_mismatched_seat_counts() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let nonexistent_date = test_date();
    let wrong_flight_date = test_date();
    let mismatch_date = test_date();

    let nonexistent = repository
        .create_hold(
            command("xf-201", nonexistent_date, &["99Z"], 3),
            Duration::from_secs(600),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        nonexistent,
        SeatHoldRepositoryError::SeatNotFound(_)
    ));

    repository
        .create_hold(
            command("xf-201", wrong_flight_date, &["26A"], 4),
            Duration::from_secs(600),
        )
        .await
        .unwrap();
    let wrong_flight = repository
        .create_hold(
            command("xf-315", wrong_flight_date, &["26A"], 5),
            Duration::from_secs(600),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        wrong_flight,
        SeatHoldRepositoryError::SeatNotFound(_)
    ));

    let mut mismatch = command("xf-201", mismatch_date, &["20A", "20B", "20C"], 6);
    mismatch.passengers = PassengerCounts::new(2, 0, 0).unwrap();
    assert!(matches!(
        repository
            .create_hold(mismatch, Duration::from_secs(600))
            .await,
        Err(SeatHoldRepositoryError::SeatCountMismatch)
    ));
}

#[tokio::test]
async fn active_hold_conflicts_but_expired_hold_does_not_block_inventory() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let conflict_date = test_date();
    let expiry_date = test_date();

    repository
        .create_hold(
            command("xf-201", conflict_date, &["20A"], 7),
            Duration::from_secs(600),
        )
        .await
        .unwrap();
    let conflict = repository
        .create_hold(
            command("xf-201", conflict_date, &["20A"], 8),
            Duration::from_secs(600),
        )
        .await
        .unwrap_err();
    assert_eq!(
        match conflict {
            SeatHoldRepositoryError::SeatConflict(values) => values,
            other => panic!("expected seat conflict, got {other:?}"),
        },
        vec!["20A"]
    );

    let expired = repository
        .create_hold(command("xf-201", expiry_date, &["20A"], 9), Duration::ZERO)
        .await
        .unwrap();
    repository
        .create_hold(
            command("xf-201", expiry_date, &["20A"], 10),
            Duration::from_secs(600),
        )
        .await
        .unwrap();
    assert!(matches!(
        repository.get_hold(expired.id, [9; 32]).await,
        Err(SeatHoldRepositoryError::HoldExpired)
    ));
}

#[tokio::test]
async fn exactly_one_concurrent_request_wins_the_same_seat() {
    let repository = Arc::new(SqlxSeatHoldRepository::new(test_pool().await));
    let barrier = Arc::new(Barrier::new(2));
    let departure_date = test_date();

    let attempts = [11_u8, 12_u8].map(|token_byte| {
        let repository = Arc::clone(&repository);
        let barrier = Arc::clone(&barrier);
        tokio::spawn(async move {
            barrier.wait().await;
            repository
                .create_hold(
                    command("xf-201", departure_date, &["20A"], token_byte),
                    Duration::from_secs(600),
                )
                .await
        })
    });

    let [first_task, second_task] = attempts;
    let first = first_task.await.expect("first task joins");
    let second = second_task.await.expect("second task joins");
    let successes = [&first, &second]
        .iter()
        .filter(|result| result.is_ok())
        .count();
    let conflicts = [&first, &second]
        .iter()
        .filter(|result| matches!(result, Err(SeatHoldRepositoryError::SeatConflict(_))))
        .count();

    assert_eq!(successes, 1);
    assert_eq!(conflicts, 1);
}

#[tokio::test]
async fn release_returns_inventory_to_available() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let departure_date = test_date();
    let hold = repository
        .create_hold(
            command("xf-201", departure_date, &["20A"], 13),
            Duration::from_secs(600),
        )
        .await
        .unwrap();

    repository.release_hold(hold.id, [13; 32]).await.unwrap();
    repository
        .create_hold(
            command("xf-201", departure_date, &["20A"], 14),
            Duration::from_secs(600),
        )
        .await
        .unwrap();
    assert!(matches!(
        repository.get_hold(hold.id, [13; 32]).await,
        Err(SeatHoldRepositoryError::HoldReleased)
    ));
}

#[tokio::test]
async fn booked_seat_cannot_be_held() {
    let pool = test_pool().await;
    let repository = SqlxSeatHoldRepository::new(pool.clone());
    let departure_date = test_date();
    repository
        .seat_map(&selection("xf-201", departure_date), None)
        .await
        .unwrap();

    sqlx::query(
        "UPDATE flight_seats AS seat
         SET booking_status = 'BOOKED', booked_at = NOW(), hold_id = NULL
         FROM flight_instances AS instance
         JOIN flight_services AS service ON service.id = instance.flight_service_id
         WHERE seat.flight_instance_id = instance.id
           AND service.public_id = $1
           AND instance.departure_date = $2
           AND seat.seat_number = $3",
    )
    .bind("xf-201")
    .bind(departure_date)
    .bind("20A")
    .execute(&pool)
    .await
    .unwrap();

    assert!(matches!(
        repository
            .create_hold(
                command("xf-201", departure_date, &["20A"], 15),
                Duration::from_secs(600),
            )
            .await,
        Err(SeatHoldRepositoryError::SeatConflict(_))
    ));
}

#[tokio::test]
async fn replacing_seats_is_atomic_and_keeps_the_original_expiry() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let departure_date = test_date();
    let original = repository
        .create_hold(
            command("xf-201", departure_date, &["20A"], 16),
            Duration::from_secs(600),
        )
        .await
        .unwrap();

    let replaced = repository
        .replace_seats(original.id, [16; 32], seats(&["20B"]))
        .await
        .unwrap();
    assert_eq!(replaced.seats, seats(&["20B"]));
    assert_eq!(replaced.expires_at, original.expires_at);

    let count: i64 = sqlx::query("SELECT COUNT(*) AS count FROM flight_seats WHERE hold_id = $1")
        .bind(original.id)
        .fetch_one(repository.pool())
        .await
        .unwrap()
        .get("count");
    assert_eq!(count, 1);
}
