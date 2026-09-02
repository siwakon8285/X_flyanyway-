use std::{
    env,
    sync::atomic::{AtomicI64, Ordering},
    time::Duration,
};

use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, Utc};
use sqlx::PgPool;

use x_fly_api::{
    domain::{
        entities::{CreateSeatHold, FlightSelection, SeatHold},
        passengers::{Gender, PassengerInput, PassengerType, Title},
        repositories::{PassengerRepository, PassengerRepositoryError, SeatHoldRepository},
        value_objects::{CabinClass, PassengerCounts, SeatNumber},
    },
    infrastructure::database::{prepare_database, SqlxSeatHoldRepository},
};

async fn test_pool() -> PgPool {
    dotenvy::dotenv().ok();
    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must point to an isolated PostgreSQL test database");
    assert!(database_url
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .ends_with("_test"));
    let pool = PgPool::connect(&database_url).await.unwrap();
    prepare_database(&pool).await.unwrap();
    pool
}

fn test_date() -> NaiveDate {
    static NEXT_OFFSET: AtomicI64 = AtomicI64::new(0);
    Utc::now().date_naive() + ChronoDuration::days(30 + NEXT_OFFSET.fetch_add(1, Ordering::Relaxed))
}

async fn create_hold(
    repository: &SqlxSeatHoldRepository,
    departure_date: NaiveDate,
    counts: PassengerCounts,
    token: [u8; 32],
    ttl: Duration,
) -> SeatHold {
    sqlx::query("DELETE FROM seat_holds WHERE access_token_hash = $1")
        .bind(token.as_slice())
        .execute(repository.pool())
        .await
        .unwrap();
    let seats = (0..counts.required_seats())
        .map(|index| SeatNumber::parse(["20A", "20B", "20C"][index]).unwrap())
        .collect();
    repository
        .create_hold(
            CreateSeatHold {
                selection: FlightSelection {
                    flight_id: "xf-201".to_owned(),
                    departure_date,
                    cabin: CabinClass::Economy,
                },
                passengers: counts,
                seats,
                token_hash: token,
            },
            ttl,
        )
        .await
        .unwrap()
}

fn passenger(ordinal: u8, passenger_type: PassengerType, dob: NaiveDate) -> PassengerInput {
    PassengerInput {
        ordinal,
        passenger_type,
        title: Title::Ms,
        given_name: " Nara ".to_owned(),
        middle_name: None,
        family_name: " Suri ".to_owned(),
        date_of_birth: dob,
        gender: Gender::Female,
        nationality_code: "th".to_owned(),
        passport_number: format!("th{ordinal}234567"),
        passport_issuing_country_code: "th".to_owned(),
        email: format!("nara{ordinal}@example.com"),
        phone_country_code: "+66".to_owned(),
        phone_number: format!("81234567{ordinal}"),
        emergency_contact: None,
    }
}

#[tokio::test]
async fn saves_and_reloads_a_normalized_full_draft() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let departure = test_date();
    let hold = create_hold(
        &repository,
        departure,
        PassengerCounts::new(1, 1, 1).unwrap(),
        [41; 32],
        Duration::from_secs(600),
    )
    .await;
    let adult_dob = departure.with_year(departure.year() - 30).unwrap();
    let child_dob = departure.with_year(departure.year() - 8).unwrap();
    let infant_dob = Utc::now().date_naive();

    let saved = repository
        .save_passengers(
            hold.id,
            [41; 32],
            vec![
                passenger(1, PassengerType::Adult, adult_dob),
                passenger(2, PassengerType::Child, child_dob),
                passenger(3, PassengerType::Infant, infant_dob),
            ],
        )
        .await
        .unwrap();
    assert_eq!(saved.hold.expires_at, hold.expires_at);
    assert!(saved.ready_to_continue);
    assert_eq!(saved.passengers.len(), 3);
    assert_eq!(saved.passengers[0].given_name, "Nara");
    assert_eq!(
        saved.expected_passengers[2].passenger_type,
        PassengerType::Infant
    );

    let reloaded = repository.get_passengers(hold.id, [41; 32]).await.unwrap();
    assert!(reloaded.ready_to_continue);
    assert_eq!(reloaded.hold.expires_at, hold.expires_at);
    assert_eq!(reloaded.passengers[2].ordinal, 3);

    repository
        .replace_seats(hold.id, [41; 32], vec![SeatNumber::parse("20A").unwrap()])
        .await
        .unwrap();
    let partial = repository.get_passengers(hold.id, [41; 32]).await.unwrap();
    assert!(!partial.ready_to_continue);
    assert_eq!(partial.passengers.len(), 3);
}

#[tokio::test]
async fn rejects_wrong_types_without_overwriting_the_previous_draft() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let departure = test_date();
    let hold = create_hold(
        &repository,
        departure,
        PassengerCounts::new(1, 1, 0).unwrap(),
        [42; 32],
        Duration::from_secs(600),
    )
    .await;
    let adult_dob = departure.with_year(departure.year() - 30).unwrap();
    let child_dob = departure.with_year(departure.year() - 8).unwrap();
    repository
        .save_passengers(
            hold.id,
            [42; 32],
            vec![
                passenger(1, PassengerType::Adult, adult_dob),
                passenger(2, PassengerType::Child, child_dob),
            ],
        )
        .await
        .unwrap();

    let rejected = repository
        .save_passengers(
            hold.id,
            [42; 32],
            vec![
                passenger(1, PassengerType::Child, child_dob),
                passenger(2, PassengerType::Adult, adult_dob),
            ],
        )
        .await;
    assert!(matches!(
        rejected,
        Err(PassengerRepositoryError::TypeMismatch)
    ));
    assert_eq!(
        repository
            .get_passengers(hold.id, [42; 32])
            .await
            .unwrap()
            .passengers[0]
            .passenger_type,
        PassengerType::Adult
    );
}

#[tokio::test]
async fn expired_released_and_other_hold_credentials_cannot_access_passengers() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let expired = create_hold(
        &repository,
        test_date(),
        PassengerCounts::new(1, 0, 0).unwrap(),
        [43; 32],
        Duration::ZERO,
    )
    .await;
    assert!(matches!(
        repository.get_passengers(expired.id, [43; 32]).await,
        Err(PassengerRepositoryError::HoldExpired)
    ));
    assert!(matches!(
        repository
            .save_passengers(
                expired.id,
                [43; 32],
                vec![passenger(
                    1,
                    PassengerType::Adult,
                    expired
                        .departure_date
                        .with_year(expired.departure_date.year() - 30)
                        .unwrap(),
                )],
            )
            .await,
        Err(PassengerRepositoryError::HoldExpired)
    ));

    let released = create_hold(
        &repository,
        test_date(),
        PassengerCounts::new(1, 0, 0).unwrap(),
        [44; 32],
        Duration::from_secs(600),
    )
    .await;
    repository
        .release_hold(released.id, [44; 32])
        .await
        .unwrap();
    assert!(matches!(
        repository.get_passengers(released.id, [44; 32]).await,
        Err(PassengerRepositoryError::HoldReleased)
    ));
    assert!(matches!(
        repository
            .save_passengers(
                released.id,
                [44; 32],
                vec![passenger(
                    1,
                    PassengerType::Adult,
                    released
                        .departure_date
                        .with_year(released.departure_date.year() - 30)
                        .unwrap(),
                )],
            )
            .await,
        Err(PassengerRepositoryError::HoldReleased)
    ));

    let active = create_hold(
        &repository,
        test_date(),
        PassengerCounts::new(1, 0, 0).unwrap(),
        [45; 32],
        Duration::from_secs(600),
    )
    .await;
    assert!(matches!(
        repository.get_passengers(active.id, [46; 32]).await,
        Err(PassengerRepositoryError::Unauthorized)
    ));
}
