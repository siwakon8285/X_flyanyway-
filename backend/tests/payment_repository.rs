use std::{env, sync::Arc, time::Duration};

use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, Utc};
use sqlx::PgPool;
use tokio::sync::Barrier;
use uuid::Uuid;

use x_fly_api::{
    domain::{
        entities::{CreateSeatHold, FlightSelection, SeatAvailability},
        extras::ExtraSelectionInput,
        passengers::{Gender, PassengerInput, PassengerType, Title},
        payment::{
            PaymentAttemptTransition, PaymentMethod, PaymentProvider, PaymentRepositoryCommand,
            PaymentStatus,
        },
        repositories::{
            ExtraRepository, PassengerRepository, PaymentRepository, PaymentRepositoryError,
            ReviewRepository, SeatHoldRepository, SeatHoldRepositoryError,
        },
        value_objects::{CabinClass, PassengerCounts, SeatNumber},
    },
    infrastructure::database::{prepare_database, SqlxSeatHoldRepository},
};

async fn test_pool() -> PgPool {
    dotenvy::dotenv().ok();
    let database_url = env::var("TEST_DATABASE_URL").unwrap();
    assert!(database_url
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .ends_with("_test"));
    let pool = PgPool::connect(&database_url).await.unwrap();
    prepare_database(&pool).await.unwrap();
    pool
}

fn departure_date() -> NaiveDate {
    Utc::now().date_naive() + ChronoDuration::days(60 + (Uuid::new_v4().as_u128() % 100_000) as i64)
}

fn passenger() -> PassengerInput {
    PassengerInput {
        ordinal: 1,
        passenger_type: PassengerType::Adult,
        title: Title::Ms,
        given_name: "Nara".to_owned(),
        middle_name: None,
        family_name: "Payment".to_owned(),
        date_of_birth: Utc::now()
            .date_naive()
            .with_year(Utc::now().year() - 30)
            .unwrap(),
        gender: Gender::Female,
        nationality_code: "TH".to_owned(),
        passport_number: format!("PY{:08X}", Uuid::new_v4().as_u128() as u32),
        passport_issuing_country_code: "TH".to_owned(),
        email: "payment@example.com".to_owned(),
        phone_country_code: "+66".to_owned(),
        phone_number: "812345678".to_owned(),
        emergency_contact: None,
    }
}

async fn ready_hold(repository: &SqlxSeatHoldRepository, token: [u8; 32]) -> Uuid {
    let departure = departure_date();
    let hold = repository
        .create_hold(
            CreateSeatHold {
                selection: FlightSelection {
                    flight_id: "xf-201".to_owned(),
                    departure_date: departure,
                    cabin: CabinClass::Economy,
                },
                passengers: PassengerCounts::new(1, 0, 0).unwrap(),
                seats: vec![SeatNumber::parse("20A").unwrap()],
                token_hash: token,
            },
            Duration::from_secs(600),
        )
        .await
        .unwrap();
    repository
        .save_passengers(hold.id, token, vec![passenger()])
        .await
        .unwrap();
    repository
        .save_extras(
            hold.id,
            token,
            vec![ExtraSelectionInput {
                passenger_ordinal: 1,
                product_code: "BAG_30KG".to_owned(),
                quantity: 1,
            }],
        )
        .await
        .unwrap();
    repository.get_review(hold.id, token).await.unwrap();
    hold.id
}

fn command(method: PaymentMethod) -> PaymentRepositoryCommand {
    PaymentRepositoryCommand {
        request_id: Uuid::new_v4(),
        request_fingerprint: [7; 32],
        method,
        provider: match method {
            PaymentMethod::Card => PaymentProvider::Stripe,
            PaymentMethod::Bitcoin => PaymentProvider::MockBitcoin,
        },
    }
}

fn stripe_reference(state: &str) -> String {
    format!("pi_{}_{}", state, Uuid::new_v4())
}

async fn expire_hold(repository: &SqlxSeatHoldRepository, hold_id: Uuid) {
    sqlx::query("UPDATE seat_holds SET expires_at = NOW() - INTERVAL '1 minute' WHERE id = $1")
        .bind(hold_id)
        .execute(repository.pool())
        .await
        .unwrap();
}

async fn hold_departure(repository: &SqlxSeatHoldRepository, hold_id: Uuid) -> NaiveDate {
    sqlx::query_scalar(
        "SELECT instance.departure_date
         FROM seat_holds AS hold
         JOIN flight_instances AS instance ON instance.id = hold.flight_instance_id
         WHERE hold.id = $1",
    )
    .bind(hold_id)
    .fetch_one(repository.pool())
    .await
    .unwrap()
}

async fn competing_hold(
    repository: &SqlxSeatHoldRepository,
    departure: NaiveDate,
    token: [u8; 32],
) -> Result<x_fly_api::domain::entities::SeatHold, SeatHoldRepositoryError> {
    repository
        .create_hold(
            CreateSeatHold {
                selection: FlightSelection {
                    flight_id: "xf-201".to_owned(),
                    departure_date: departure,
                    cabin: CabinClass::Economy,
                },
                passengers: PassengerCounts::new(1, 0, 0).unwrap(),
                seats: vec![SeatNumber::parse("20A").unwrap()],
                token_hash: token,
            },
            Duration::from_secs(600),
        )
        .await
}

#[tokio::test]
async fn reserves_only_the_authoritative_review_amount() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [121; 32];
    let hold_id = ready_hold(&repository, token).await;

    let attempt = repository
        .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
        .await
        .unwrap();

    assert_eq!(attempt.amount.amount, 27_300);
    assert_eq!(attempt.amount.currency_code, "THB");
    assert_eq!(attempt.status, PaymentStatus::Created);
}

#[tokio::test]
async fn successful_payment_atomically_consumes_the_hold_and_books_its_seat() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [122; 32];
    let hold_id = ready_hold(&repository, token).await;
    let attempt = repository
        .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::processing(stripe_reference("processing")),
        )
        .await
        .unwrap();

    let succeeded = repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::succeeded(stripe_reference("succeeded")),
        )
        .await
        .unwrap();

    assert_eq!(succeeded.status, PaymentStatus::Succeeded);
    let consumed_at: Option<chrono::DateTime<Utc>> =
        sqlx::query_scalar("SELECT consumed_at FROM seat_holds WHERE id = $1")
            .bind(hold_id)
            .fetch_one(repository.pool())
            .await
            .unwrap();
    assert!(consumed_at.is_some());
    let seat: (String, Option<Uuid>, Option<chrono::DateTime<Utc>>) = sqlx::query_as(
        "SELECT booking_status, hold_id, booked_at FROM flight_seats WHERE seat_number = '20A' AND flight_instance_id = (SELECT flight_instance_id FROM seat_holds WHERE id = $1)",
    )
    .bind(hold_id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    assert_eq!(seat.0, "BOOKED");
    assert_eq!(seat.1, None);
    assert!(seat.2.is_some());
    let finalized_seats: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM payment_attempt_seats WHERE payment_attempt_id = $1",
    )
    .bind(attempt.id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    assert_eq!(finalized_seats, 1);
    let context = repository.get_payment(hold_id, token).await.unwrap();
    assert_eq!(context.hold.seats, vec![SeatNumber::parse("20A").unwrap()]);
}

#[tokio::test]
async fn failed_attempt_leaves_the_active_hold_and_inventory_unchanged() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [123; 32];
    let hold_id = ready_hold(&repository, token).await;
    let attempt = repository
        .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
        .await
        .unwrap();

    let failed = repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::failed("MOCK_CARD_DECLINED", "Demo card declined."),
        )
        .await
        .unwrap();

    assert_eq!(failed.status, PaymentStatus::Failed);
    let hold_state: (Option<chrono::DateTime<Utc>>, Option<chrono::DateTime<Utc>>) =
        sqlx::query_as("SELECT consumed_at, released_at FROM seat_holds WHERE id = $1")
            .bind(hold_id)
            .fetch_one(repository.pool())
            .await
            .unwrap();
    assert_eq!(hold_state, (None, None));
    let seat: (String, Option<Uuid>) = sqlx::query_as(
        "SELECT booking_status, hold_id FROM flight_seats WHERE seat_number = '20A' AND flight_instance_id = (SELECT flight_instance_id FROM seat_holds WHERE id = $1)",
    )
    .bind(hold_id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    assert_eq!(seat, ("AVAILABLE".to_owned(), Some(hold_id)));
}

#[tokio::test]
async fn failed_inventory_finalization_cannot_leave_payment_succeeded() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [124; 32];
    let hold_id = ready_hold(&repository, token).await;
    let attempt = repository
        .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::processing(stripe_reference("processing")),
        )
        .await
        .unwrap();
    sqlx::query("UPDATE flight_seats SET hold_id = NULL WHERE hold_id = $1")
        .bind(hold_id)
        .execute(repository.pool())
        .await
        .unwrap();

    assert!(matches!(
        repository
            .transition_payment_attempt(
                hold_id,
                token,
                attempt.id,
                PaymentAttemptTransition::succeeded(stripe_reference("succeeded")),
            )
            .await,
        Err(PaymentRepositoryError::SeatsNotReady)
    ));
    let persisted: String = sqlx::query_scalar("SELECT status FROM payment_attempts WHERE id = $1")
        .bind(attempt.id)
        .fetch_one(repository.pool())
        .await
        .unwrap();
    assert_eq!(persisted, "PROCESSING");
    let consumed: Option<chrono::DateTime<Utc>> =
        sqlx::query_scalar("SELECT consumed_at FROM seat_holds WHERE id = $1")
            .bind(hold_id)
            .fetch_one(repository.pool())
            .await
            .unwrap();
    assert!(consumed.is_none());
}

#[tokio::test]
async fn stripe_success_after_normal_expiry_still_atomically_finalizes_inventory() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [125; 32];
    let hold_id = ready_hold(&repository, token).await;
    let attempt = repository
        .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::processing(stripe_reference("processing")),
        )
        .await
        .unwrap();
    sqlx::query("UPDATE seat_holds SET expires_at = NOW() WHERE id = $1")
        .bind(hold_id)
        .execute(repository.pool())
        .await
        .unwrap();

    let succeeded = repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::succeeded(stripe_reference("succeeded")),
        )
        .await
        .unwrap();
    assert_eq!(succeeded.status, PaymentStatus::Succeeded);
    let persisted: (String, Option<String>) =
        sqlx::query_as("SELECT status, failure_code FROM payment_attempts WHERE id = $1")
            .bind(attempt.id)
            .fetch_one(repository.pool())
            .await
            .unwrap();
    assert_eq!(persisted, ("SUCCEEDED".to_owned(), None));
    let inventory: (Option<chrono::DateTime<Utc>>, String, Option<Uuid>) = sqlx::query_as(
        "SELECT hold.consumed_at, seat.booking_status, seat.hold_id
         FROM seat_holds AS hold
         JOIN flight_seats AS seat ON seat.flight_instance_id = hold.flight_instance_id AND seat.seat_number = '20A'
         WHERE hold.id = $1",
    )
    .bind(hold_id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    assert!(inventory.0.is_some());
    assert_eq!(inventory.1, "BOOKED");
    assert_eq!(inventory.2, None);
}

#[tokio::test]
async fn paid_seat_stays_booked_after_the_original_hold_expiry() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [126; 32];
    let hold_id = ready_hold(&repository, token).await;
    let attempt = repository
        .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::processing(stripe_reference("processing")),
        )
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::succeeded(stripe_reference("succeeded")),
        )
        .await
        .unwrap();
    sqlx::query("UPDATE seat_holds SET expires_at = NOW() - INTERVAL '1 minute' WHERE id = $1")
        .bind(hold_id)
        .execute(repository.pool())
        .await
        .unwrap();

    let departure: NaiveDate =
        sqlx::query_scalar("SELECT departure_date FROM flight_instances AS instance JOIN seat_holds AS hold ON hold.flight_instance_id = instance.id WHERE hold.id = $1")
            .bind(hold_id)
            .fetch_one(repository.pool())
            .await
            .unwrap();
    let conflict = repository
        .create_hold(
            CreateSeatHold {
                selection: FlightSelection {
                    flight_id: "xf-201".to_owned(),
                    departure_date: departure,
                    cabin: CabinClass::Economy,
                },
                passengers: PassengerCounts::new(1, 0, 0).unwrap(),
                seats: vec![SeatNumber::parse("20A").unwrap()],
                token_hash: [127; 32],
            },
            Duration::from_secs(600),
        )
        .await;
    assert!(matches!(
        conflict,
        Err(x_fly_api::domain::repositories::SeatHoldRepositoryError::SeatConflict(_))
    ));
}

#[tokio::test]
async fn failed_and_cancelled_attempts_allow_a_new_attempt_on_the_active_hold() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [128; 32];
    let hold_id = ready_hold(&repository, token).await;
    let first = repository
        .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold_id,
            token,
            first.id,
            PaymentAttemptTransition::failed("MOCK_CARD_DECLINED", "Demo card declined."),
        )
        .await
        .unwrap();
    let second = repository
        .create_payment_attempt(hold_id, token, command(PaymentMethod::Bitcoin))
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold_id,
            token,
            second.id,
            PaymentAttemptTransition::awaiting_payment("XFBTC-AWAITING"),
        )
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold_id,
            token,
            second.id,
            PaymentAttemptTransition::cancelled(
                "MOCK_BITCOIN_CANCELLED",
                "Demo invoice cancelled.",
            ),
        )
        .await
        .unwrap();

    assert!(repository
        .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
        .await
        .is_ok());
    let unchanged: (Option<chrono::DateTime<Utc>>, i64) = sqlx::query_as(
        "SELECT consumed_at, (SELECT COUNT(*) FROM flight_seats WHERE hold_id = $1) FROM seat_holds WHERE id = $1",
    )
    .bind(hold_id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    assert_eq!(unchanged, (None, 1));
}

#[tokio::test]
async fn concurrent_payment_requests_produce_one_success_and_one_finalization() {
    let repository = Arc::new(SqlxSeatHoldRepository::new(test_pool().await));
    let token = [129; 32];
    let hold_id = ready_hold(&repository, token).await;
    let barrier = Arc::new(Barrier::new(2));
    let tasks = [command(PaymentMethod::Card), command(PaymentMethod::Card)].map(|command| {
        let repository = Arc::clone(&repository);
        let barrier = Arc::clone(&barrier);
        tokio::spawn(async move {
            barrier.wait().await;
            let attempt = repository
                .create_payment_attempt(hold_id, token, command)
                .await?;
            repository
                .transition_payment_attempt(
                    hold_id,
                    token,
                    attempt.id,
                    PaymentAttemptTransition::processing(stripe_reference("processing")),
                )
                .await?;
            repository
                .transition_payment_attempt(
                    hold_id,
                    token,
                    attempt.id,
                    PaymentAttemptTransition::succeeded(stripe_reference("succeeded")),
                )
                .await
        })
    });
    let [first, second] = tasks;
    let results = [first.await.unwrap(), second.await.unwrap()];

    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    let successes: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM payment_attempts WHERE seat_hold_id = $1 AND status = 'SUCCEEDED'",
    )
    .bind(hold_id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    let finalized: (bool, i64) = sqlx::query_as(
        "SELECT consumed_at IS NOT NULL,
            (SELECT COUNT(*) FROM flight_seats WHERE flight_instance_id = seat_holds.flight_instance_id AND booking_status = 'BOOKED')
         FROM seat_holds WHERE id = $1",
    )
    .bind(hold_id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    assert_eq!(successes, 1);
    assert_eq!(finalized, (true, 1));
}

#[tokio::test]
async fn payment_creation_never_extends_the_hold_and_stores_no_card_fields() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [130; 32];
    let hold_id = ready_hold(&repository, token).await;
    let before: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT expires_at FROM seat_holds WHERE id = $1")
            .bind(hold_id)
            .fetch_one(repository.pool())
            .await
            .unwrap();

    repository
        .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
        .await
        .unwrap();

    let after: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT expires_at FROM seat_holds WHERE id = $1")
            .bind(hold_id)
            .fetch_one(repository.pool())
            .await
            .unwrap();
    assert_eq!(after, before);
    let columns: Vec<String> = sqlx::query_scalar(
        "SELECT column_name FROM information_schema.columns
         WHERE table_schema = 'public' AND table_name = 'payment_attempts'",
    )
    .fetch_all(repository.pool())
    .await
    .unwrap();
    for forbidden in [
        "card_number",
        "pan",
        "cvc",
        "cvv",
        "expiry",
        "private_key",
        "seed",
    ] {
        assert!(!columns.iter().any(|column| column == forbidden));
    }
}

#[tokio::test]
async fn a_different_request_after_success_is_rejected_without_a_second_payment() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [131; 32];
    let hold_id = ready_hold(&repository, token).await;
    let attempt = repository
        .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::processing(stripe_reference("processing")),
        )
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::succeeded(stripe_reference("succeeded")),
        )
        .await
        .unwrap();

    assert!(matches!(
        repository
            .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
            .await,
        Err(PaymentRepositoryError::AlreadySucceeded)
    ));
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM payment_attempts WHERE seat_hold_id = $1 AND status = 'SUCCEEDED'",
    )
    .bind(hold_id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn inactive_holds_cannot_start_payment() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);

    for (token, lifecycle, expected) in [
        ([132; 32], "expired", PaymentRepositoryError::HoldExpired),
        ([133; 32], "released", PaymentRepositoryError::HoldReleased),
        ([134; 32], "consumed", PaymentRepositoryError::HoldConsumed),
    ] {
        let hold_id = ready_hold(&repository, token).await;
        let statement = match lifecycle {
            "expired" => "UPDATE seat_holds SET expires_at = NOW() WHERE id = $1",
            "released" => "UPDATE seat_holds SET released_at = NOW() WHERE id = $1",
            _ => "UPDATE seat_holds SET consumed_at = NOW() WHERE id = $1",
        };
        sqlx::query(statement)
            .bind(hold_id)
            .execute(repository.pool())
            .await
            .unwrap();
        let result = repository
            .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
            .await;
        assert_eq!(result.unwrap_err().to_string(), expected.to_string());
    }
}

#[tokio::test]
async fn stripe_attempt_deadline_is_fixed_from_original_expiry_and_idempotent() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [135; 32];
    let hold_id = ready_hold(&repository, token).await;
    let stripe_command = command(PaymentMethod::Card);
    let original_expiry: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT expires_at FROM seat_holds WHERE id = $1")
            .bind(hold_id)
            .fetch_one(repository.pool())
            .await
            .unwrap();

    let first = repository
        .create_payment_attempt(hold_id, token, stripe_command.clone())
        .await
        .unwrap();
    let first_deadline: chrono::DateTime<Utc> = sqlx::query_scalar(
        "SELECT payment_finalization_deadline FROM payment_attempts WHERE id = $1",
    )
    .bind(first.id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    assert_eq!(first_deadline, original_expiry + ChronoDuration::minutes(5));
    let unchanged_expiry: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT expires_at FROM seat_holds WHERE id = $1")
            .bind(hold_id)
            .fetch_one(repository.pool())
            .await
            .unwrap();
    assert_eq!(unchanged_expiry, original_expiry);

    expire_hold(&repository, hold_id).await;
    let replay = repository
        .create_payment_attempt(hold_id, token, stripe_command.clone())
        .await
        .unwrap();
    assert_eq!(replay.id, first.id);
    let replayed_deadline: chrono::DateTime<Utc> = sqlx::query_scalar(
        "SELECT payment_finalization_deadline FROM payment_attempts WHERE id = $1",
    )
    .bind(first.id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    assert_eq!(replayed_deadline, first_deadline);

    assert!(matches!(
        repository
            .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
            .await,
        Err(PaymentRepositoryError::HoldExpired)
    ));
    let conflicting = PaymentRepositoryCommand {
        method: PaymentMethod::Bitcoin,
        provider: PaymentProvider::MockBitcoin,
        request_fingerprint: [8; 32],
        ..stripe_command
    };
    assert!(matches!(
        repository
            .create_payment_attempt(hold_id, token, conflicting)
            .await,
        Err(PaymentRepositoryError::IdempotencyKeyReused)
    ));
}

#[tokio::test]
async fn unresolved_stripe_attempt_protects_expired_inventory_even_after_its_deadline() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [136; 32];
    let hold_id = ready_hold(&repository, token).await;
    let departure = hold_departure(&repository, hold_id).await;
    let attempt = repository
        .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::processing(stripe_reference("protected")),
        )
        .await
        .unwrap();
    expire_hold(&repository, hold_id).await;
    sqlx::query(
        "UPDATE payment_attempts
         SET payment_finalization_deadline = NOW() - INTERVAL '1 minute'
         WHERE id = $1",
    )
    .bind(attempt.id)
    .execute(repository.pool())
    .await
    .unwrap();

    assert!(matches!(
        competing_hold(&repository, departure, [137; 32]).await,
        Err(SeatHoldRepositoryError::SeatConflict(_))
    ));
    let map = repository
        .seat_map(
            &FlightSelection {
                flight_id: "xf-201".to_owned(),
                departure_date: departure,
                cabin: CabinClass::Economy,
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        map.seats
            .iter()
            .find(|seat| seat.seat_number.as_str() == "20A")
            .unwrap()
            .status,
        SeatAvailability::Unavailable
    );
    let assigned_hold: Option<Uuid> = sqlx::query_scalar(
        "SELECT hold_id FROM flight_seats
         WHERE flight_instance_id = (SELECT flight_instance_id FROM seat_holds WHERE id = $1)
           AND seat_number = '20A'",
    )
    .bind(hold_id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    assert_eq!(assigned_hold, Some(hold_id));
    assert!(repository.release_hold(hold_id, token).await.is_err());
}

#[tokio::test]
async fn terminal_stripe_attempts_and_bitcoin_do_not_receive_inventory_grace() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);

    for (token_byte, terminal) in [
        (138, PaymentStatus::Failed),
        (140, PaymentStatus::Cancelled),
    ] {
        let token = [token_byte; 32];
        let hold_id = ready_hold(&repository, token).await;
        let departure = hold_departure(&repository, hold_id).await;
        let attempt = repository
            .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
            .await
            .unwrap();
        if terminal == PaymentStatus::Cancelled {
            repository
                .transition_payment_attempt(
                    hold_id,
                    token,
                    attempt.id,
                    PaymentAttemptTransition::processing(stripe_reference("cancel")),
                )
                .await
                .unwrap();
        }
        expire_hold(&repository, hold_id).await;
        let transition = if terminal == PaymentStatus::Failed {
            PaymentAttemptTransition::failed("CARD_FAILED", "Card payment failed.")
        } else {
            PaymentAttemptTransition::cancelled("CARD_CANCELLED", "Card payment cancelled.")
        };
        repository
            .transition_payment_attempt(hold_id, token, attempt.id, transition)
            .await
            .unwrap();
        competing_hold(&repository, departure, [token_byte + 1; 32])
            .await
            .unwrap();
    }

    let bitcoin_token = [142; 32];
    let bitcoin_hold = ready_hold(&repository, bitcoin_token).await;
    let bitcoin_departure = hold_departure(&repository, bitcoin_hold).await;
    let bitcoin_attempt = repository
        .create_payment_attempt(bitcoin_hold, bitcoin_token, command(PaymentMethod::Bitcoin))
        .await
        .unwrap();
    let bitcoin_deadline: Option<chrono::DateTime<Utc>> = sqlx::query_scalar(
        "SELECT payment_finalization_deadline FROM payment_attempts WHERE id = $1",
    )
    .bind(bitcoin_attempt.id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    assert!(bitcoin_deadline.is_none());
    expire_hold(&repository, bitcoin_hold).await;
    competing_hold(&repository, bitcoin_departure, [143; 32])
        .await
        .unwrap();
}

#[tokio::test]
async fn protected_stripe_attempt_blocks_upstream_mutations_and_explicit_release() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);

    let seat_token = [144; 32];
    let seat_hold = ready_hold(&repository, seat_token).await;
    repository
        .create_payment_attempt(seat_hold, seat_token, command(PaymentMethod::Card))
        .await
        .unwrap();
    assert_eq!(
        repository
            .replace_seats(
                seat_hold,
                seat_token,
                vec![SeatNumber::parse("20B").unwrap()],
            )
            .await
            .unwrap_err()
            .to_string(),
        "payment finalization is in progress"
    );
    assert_eq!(
        repository
            .release_hold(seat_hold, seat_token)
            .await
            .unwrap_err()
            .to_string(),
        "payment finalization is in progress"
    );

    let passenger_token = [145; 32];
    let passenger_hold = ready_hold(&repository, passenger_token).await;
    repository
        .create_payment_attempt(
            passenger_hold,
            passenger_token,
            command(PaymentMethod::Card),
        )
        .await
        .unwrap();
    assert_eq!(
        repository
            .save_passengers(passenger_hold, passenger_token, vec![passenger()])
            .await
            .err()
            .unwrap()
            .to_string(),
        "payment finalization is in progress"
    );

    let extras_token = [146; 32];
    let extras_hold = ready_hold(&repository, extras_token).await;
    repository
        .create_payment_attempt(extras_hold, extras_token, command(PaymentMethod::Card))
        .await
        .unwrap();
    assert_eq!(
        repository
            .save_extras(extras_hold, extras_token, Vec::new())
            .await
            .err()
            .unwrap()
            .to_string(),
        "payment finalization is in progress"
    );

    expire_hold(&repository, passenger_hold).await;
    assert!(matches!(
        repository
            .save_passengers(passenger_hold, passenger_token, vec![passenger()])
            .await,
        Err(x_fly_api::domain::repositories::PassengerRepositoryError::HoldExpired)
    ));
    expire_hold(&repository, extras_hold).await;
    assert!(matches!(
        repository
            .save_extras(extras_hold, extras_token, Vec::new())
            .await,
        Err(x_fly_api::domain::repositories::ExtraRepositoryError::HoldExpired)
    ));
}

#[tokio::test]
async fn existing_stripe_attempt_can_enter_processing_after_normal_expiry() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [147; 32];
    let hold_id = ready_hold(&repository, token).await;
    let attempt = repository
        .create_payment_attempt(hold_id, token, command(PaymentMethod::Card))
        .await
        .unwrap();
    expire_hold(&repository, hold_id).await;

    let processing = repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::processing(stripe_reference("after_expiry")),
        )
        .await
        .unwrap();
    assert_eq!(processing.status, PaymentStatus::Processing);
}
