use std::{env, sync::Arc, time::Duration};

use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, Utc};
use sqlx::PgPool;
use tokio::sync::Barrier;
use uuid::Uuid;

use x_fly_api::{
    domain::{
        entities::{CreateSeatHold, FlightSelection},
        extras::ExtraSelectionInput,
        passengers::{Gender, PassengerInput, PassengerType, Title},
        payment::{
            PaymentAttemptTransition, PaymentMethod, PaymentProvider, PaymentRepositoryCommand,
            PaymentStatus,
        },
        repositories::{
            ExtraRepository, PassengerRepository, PaymentRepository, PaymentRepositoryError,
            ReviewRepository, SeatHoldRepository,
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
            PaymentMethod::Card => PaymentProvider::MockCard,
            PaymentMethod::Bitcoin => PaymentProvider::MockBitcoin,
        },
    }
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
            PaymentAttemptTransition::processing("XFCARD-PROCESSING"),
        )
        .await
        .unwrap();

    let succeeded = repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::succeeded("XFCARD-SUCCESS"),
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
            PaymentAttemptTransition::processing("XFCARD-PROCESSING"),
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
                PaymentAttemptTransition::succeeded("XFCARD-SUCCESS"),
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
async fn expiry_before_success_persists_failure_without_finalizing_inventory() {
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
            PaymentAttemptTransition::processing("XFCARD-PROCESSING"),
        )
        .await
        .unwrap();
    sqlx::query("UPDATE seat_holds SET expires_at = NOW() WHERE id = $1")
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
                PaymentAttemptTransition::succeeded("XFCARD-SUCCESS"),
            )
            .await,
        Err(PaymentRepositoryError::HoldExpired)
    ));
    let persisted: (String, Option<String>) =
        sqlx::query_as("SELECT status, failure_code FROM payment_attempts WHERE id = $1")
            .bind(attempt.id)
            .fetch_one(repository.pool())
            .await
            .unwrap();
    assert_eq!(
        persisted,
        ("FAILED".to_owned(), Some("HOLD_EXPIRED".to_owned()))
    );
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
    assert_eq!(inventory, (None, "AVAILABLE".to_owned(), Some(hold_id)));
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
            PaymentAttemptTransition::processing("XFCARD-PROCESSING"),
        )
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::succeeded("XFCARD-SUCCESS"),
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
                    PaymentAttemptTransition::processing("XFCARD-PROCESSING"),
                )
                .await?;
            repository
                .transition_payment_attempt(
                    hold_id,
                    token,
                    attempt.id,
                    PaymentAttemptTransition::succeeded("XFCARD-SUCCESS"),
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
            PaymentAttemptTransition::processing("XFCARD-PROCESSING"),
        )
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold_id,
            token,
            attempt.id,
            PaymentAttemptTransition::succeeded("XFCARD-SUCCESS"),
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
