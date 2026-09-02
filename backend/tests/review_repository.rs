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
        extras::ExtraSelectionInput,
        passengers::{Gender, PassengerInput, PassengerType, Title},
        repositories::{
            ExtraRepository, PassengerRepository, ReviewRepository, ReviewRepositoryError,
            SeatHoldRepository,
        },
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
    Utc::now().date_naive()
        + ChronoDuration::days(400 + NEXT_OFFSET.fetch_add(1, Ordering::Relaxed))
}

fn passenger(ordinal: u8, passenger_type: PassengerType, departure: NaiveDate) -> PassengerInput {
    PassengerInput {
        ordinal,
        passenger_type,
        title: Title::Ms,
        given_name: ["Nara", "Mali", "Dara"][(ordinal - 1) as usize].to_owned(),
        middle_name: None,
        family_name: "Review".to_owned(),
        date_of_birth: match passenger_type {
            PassengerType::Adult => departure.with_year(departure.year() - 30).unwrap(),
            PassengerType::Child => departure.with_year(departure.year() - 8).unwrap(),
            PassengerType::Infant => Utc::now().date_naive(),
        },
        gender: Gender::Unspecified,
        nationality_code: "TH".to_owned(),
        passport_number: format!("RV{ordinal}{:08X}", uuid::Uuid::new_v4().as_u128() as u32),
        passport_issuing_country_code: "TH".to_owned(),
        email: format!("review{ordinal}@example.com"),
        phone_country_code: "+66".to_owned(),
        phone_number: format!("81234567{ordinal}"),
        emergency_contact: None,
    }
}

async fn create_ready_passengers(repository: &SqlxSeatHoldRepository, token: [u8; 32]) -> SeatHold {
    let departure = test_date();
    let counts = PassengerCounts::new(1, 1, 1).unwrap();
    let hold = repository
        .create_hold(
            CreateSeatHold {
                selection: FlightSelection {
                    flight_id: "xf-201".to_owned(),
                    departure_date: departure,
                    cabin: CabinClass::Economy,
                },
                passengers: counts,
                seats: vec![
                    SeatNumber::parse("20A").unwrap(),
                    SeatNumber::parse("20B").unwrap(),
                ],
                token_hash: token,
            },
            Duration::from_secs(600),
        )
        .await
        .unwrap();
    repository
        .save_passengers(
            hold.id,
            token,
            vec![
                passenger(1, PassengerType::Adult, departure),
                passenger(2, PassengerType::Child, departure),
                passenger(3, PassengerType::Infant, departure),
            ],
        )
        .await
        .unwrap();
    hold
}

#[tokio::test]
async fn materializes_one_authoritative_snapshot_without_extending_the_hold() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [91; 32];
    let hold = create_ready_passengers(&repository, token).await;
    repository
        .save_extras(
            hold.id,
            token,
            vec![ExtraSelectionInput {
                passenger_ordinal: 1,
                product_code: "BAG_20KG".to_owned(),
                quantity: 1,
            }],
        )
        .await
        .unwrap();

    let first = repository.get_review(hold.id, token).await.unwrap();
    let second = repository.get_review(hold.id, token).await.unwrap();

    assert_eq!(first.hold.expires_at, hold.expires_at);
    assert_eq!(second.hold.expires_at, hold.expires_at);
    assert_eq!(first.journey.flight_number, "XF 201");
    assert_eq!(first.journey.origin_code, "BKK");
    assert_eq!(first.journey.destination_code, "LHR");
    assert_eq!(first.journey.departure_time, "09:15");
    assert_eq!(first.journey.arrival_time, "16:40");
    assert_eq!(first.journey.aircraft_code, "Airbus A350-900");
    assert_eq!(first.passengers.len(), 3);
    assert_eq!(first.passengers[0].display_name, "MS Nara Review");
    assert_eq!(first.seats.len(), 2);
    assert_eq!(first.seats[0].seat_number.as_str(), "20A");
    assert_eq!(first.passengers[0].extras[0].product_code, "BAG_20KG");
    assert_eq!(first.pricing.base_fare.amount.amount, 43_800);
    assert_eq!(first.pricing.extras.amount, 2_800);
    assert_eq!(first.pricing.taxes[0].amount.amount, 1_400);
    assert_eq!(first.pricing.fees[0].amount.amount, 1_000);
    assert_eq!(first.pricing.fees[1].amount.amount, 300);
    assert_eq!(first.pricing.grand_total.amount, 49_300);
    assert_eq!(first.pricing.priced_at, second.pricing.priced_at);
    assert!(first.ready_for_payment);
    assert!(first.fare_conditions.fixture);
    assert!(!first.fare_conditions.refundable);
    assert!(!first.fare_conditions.changes_allowed);

    let snapshot_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM hold_review_pricing WHERE seat_hold_id = $1")
            .bind(hold.id)
            .fetch_one(repository.pool())
            .await
            .unwrap();
    assert_eq!(snapshot_count, 1);
}

#[tokio::test]
async fn requires_explicit_extras_readiness_and_accepts_an_explicit_empty_save() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [92; 32];
    let hold = create_ready_passengers(&repository, token).await;

    assert!(matches!(
        repository.get_review(hold.id, token).await,
        Err(ReviewRepositoryError::ExtrasNotReady)
    ));

    repository
        .save_extras(hold.id, token, Vec::new())
        .await
        .unwrap();
    let review = repository.get_review(hold.id, token).await.unwrap();
    assert!(review
        .passengers
        .iter()
        .all(|passenger| passenger.extras.is_empty()));
    assert_eq!(review.pricing.extras.amount, 0);
    assert_eq!(review.pricing.grand_total.amount, 46_500);
}

#[tokio::test]
async fn upstream_saves_invalidate_and_rematerialize_review_pricing() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [93; 32];
    let hold = create_ready_passengers(&repository, token).await;
    repository
        .save_extras(hold.id, token, Vec::new())
        .await
        .unwrap();
    let initial = repository.get_review(hold.id, token).await.unwrap();

    repository
        .save_extras(
            hold.id,
            token,
            vec![ExtraSelectionInput {
                passenger_ordinal: 1,
                product_code: "BAG_10KG".to_owned(),
                quantity: 1,
            }],
        )
        .await
        .unwrap();
    let repriced = repository.get_review(hold.id, token).await.unwrap();
    assert_eq!(repriced.pricing.grand_total.amount, 48_000);
    assert_ne!(repriced.pricing.priced_at, initial.pricing.priced_at);

    let passengers = vec![
        passenger(1, PassengerType::Adult, hold.departure_date),
        passenger(2, PassengerType::Child, hold.departure_date),
        passenger(3, PassengerType::Infant, hold.departure_date),
    ];
    repository
        .save_passengers(hold.id, token, passengers)
        .await
        .unwrap();
    let count_after_passenger_save: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM hold_review_pricing WHERE seat_hold_id = $1")
            .bind(hold.id)
            .fetch_one(repository.pool())
            .await
            .unwrap();
    assert_eq!(count_after_passenger_save, 0);
}

#[tokio::test]
async fn seeds_complete_authoritative_fares_for_every_available_cabin() {
    let pool = test_pool().await;
    let available_cabins: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM flight_service_cabins")
        .fetch_one(&pool)
        .await
        .unwrap();
    let priced_cabins: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM flight_service_cabins
         WHERE base_fare_amount IS NOT NULL AND currency_code = 'THB'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(available_cabins, 74);
    assert_eq!(priced_cabins, available_cabins);

    let scheduled_services: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM flight_services
         WHERE departure_time IS NOT NULL AND arrival_time IS NOT NULL
           AND arrival_day_offset IS NOT NULL AND duration_minutes IS NOT NULL
           AND stops IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(scheduled_services, 19);

    let economy_fare: i64 = sqlx::query_scalar(
        "SELECT cabin.base_fare_amount
         FROM flight_service_cabins AS cabin
         JOIN flight_services AS service ON service.id = cabin.flight_service_id
         WHERE service.public_id = 'xf-201' AND cabin.cabin = 'economy'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(economy_fare, 21_900);
}
