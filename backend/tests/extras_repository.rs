use std::{env, time::Duration};

use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, Utc};
use sqlx::PgPool;

use x_fly_api::{
    domain::{
        entities::{CreateSeatHold, FlightSelection, SeatHold},
        extras::ExtraSelectionInput,
        passengers::{Gender, PassengerInput, PassengerType, Title},
        repositories::{
            ExtraRepository, ExtraRepositoryError, PassengerRepository, SeatHoldRepository,
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
    Utc::now().date_naive()
        + ChronoDuration::days(60 + (uuid::Uuid::new_v4().as_u128() % 400) as i64)
}

async fn complete_hold(
    repository: &SqlxSeatHoldRepository,
    token: [u8; 32],
    counts: PassengerCounts,
) -> SeatHold {
    sqlx::query("DELETE FROM seat_holds WHERE access_token_hash = $1")
        .bind(token.as_slice())
        .execute(repository.pool())
        .await
        .unwrap();
    let departure = test_date();
    let seats = ["20A", "20B", "20C"]
        .iter()
        .take(counts.required_seats())
        .map(|seat| SeatNumber::parse(seat).unwrap())
        .collect();
    let hold = repository
        .create_hold(
            CreateSeatHold {
                selection: FlightSelection {
                    flight_id: "xf-201".to_owned(),
                    departure_date: departure,
                    cabin: CabinClass::Economy,
                },
                passengers: counts,
                seats,
                token_hash: token,
            },
            Duration::from_secs(600),
        )
        .await
        .unwrap();
    let passenger_types = [
        PassengerType::Adult,
        PassengerType::Child,
        PassengerType::Infant,
    ];
    let inputs = passenger_types
        .into_iter()
        .take(usize::from(
            counts.adults() + counts.children() + counts.infants(),
        ))
        .enumerate()
        .map(|(index, passenger_type)| passenger(index as u8 + 1, passenger_type, departure))
        .collect();
    repository
        .save_passengers(hold.id, token, inputs)
        .await
        .unwrap();
    hold
}

fn passenger(ordinal: u8, passenger_type: PassengerType, departure: NaiveDate) -> PassengerInput {
    let date_of_birth = match passenger_type {
        PassengerType::Adult => departure.with_year(departure.year() - 30).unwrap(),
        PassengerType::Child => departure.with_year(departure.year() - 8).unwrap(),
        PassengerType::Infant => Utc::now().date_naive(),
    };
    PassengerInput {
        ordinal,
        passenger_type,
        title: Title::Ms,
        given_name: "Nara".to_owned(),
        middle_name: None,
        family_name: "Suri".to_owned(),
        date_of_birth,
        gender: Gender::Female,
        nationality_code: "TH".to_owned(),
        passport_number: format!("EX{ordinal}{:08X}", uuid::Uuid::new_v4().as_u128() as u32),
        passport_issuing_country_code: "TH".to_owned(),
        email: format!("extra{ordinal}@example.com"),
        phone_country_code: "+66".to_owned(),
        phone_number: format!("81234567{ordinal}"),
        emergency_contact: None,
    }
}

fn selection(passenger_ordinal: u8, product_code: &str) -> ExtraSelectionInput {
    ExtraSelectionInput {
        passenger_ordinal,
        product_code: product_code.to_owned(),
        quantity: 1,
    }
}

#[tokio::test]
async fn saves_reloads_and_explicitly_acknowledges_empty_extras_without_extending_hold() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [71; 32];
    let hold = complete_hold(&repository, token, PassengerCounts::new(1, 1, 0).unwrap()).await;

    let initial = repository.get_extras(hold.id, token).await.unwrap();
    assert!(!initial.ready_to_continue);
    assert!(initial.saved_at.is_none());
    assert!(initial.selections.is_empty());

    let saved = repository
        .save_extras(
            hold.id,
            token,
            vec![selection(1, "BAG_20KG"), selection(2, "MEAL_CHILD")],
        )
        .await
        .unwrap();
    assert_eq!(saved.hold.expires_at, hold.expires_at);
    assert!(saved.ready_to_continue);
    assert!(saved.saved_at.is_some());
    assert_eq!(saved.total.amount, 2_800);

    let reloaded = repository.get_extras(hold.id, token).await.unwrap();
    assert_eq!(reloaded.selections.len(), 2);
    assert_eq!(reloaded.total.amount, 2_800);

    let empty = repository
        .save_extras(hold.id, token, Vec::new())
        .await
        .unwrap();
    assert!(empty.ready_to_continue);
    assert!(empty.saved_at.is_some());
    assert!(empty.selections.is_empty());
    assert_eq!(empty.total.amount, 0);
}

#[tokio::test]
async fn invalid_replacement_does_not_overwrite_saved_extras() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let token = [72; 32];
    let hold = complete_hold(&repository, token, PassengerCounts::new(1, 0, 0).unwrap()).await;
    repository
        .save_extras(hold.id, token, vec![selection(1, "BAG_10KG")])
        .await
        .unwrap();

    let invalid = repository
        .save_extras(hold.id, token, vec![selection(2, "BAG_30KG")])
        .await;
    assert!(matches!(
        invalid,
        Err(ExtraRepositoryError::InvalidPassenger)
    ));

    let reloaded = repository.get_extras(hold.id, token).await.unwrap();
    assert_eq!(reloaded.selections.len(), 1);
    assert_eq!(reloaded.selections[0].product_code, "BAG_10KG");
    assert_eq!(reloaded.total.amount, 1_500);
}

#[tokio::test]
async fn rejects_expired_released_and_other_hold_credentials() {
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    let expired_token = [73; 32];
    let expired = complete_hold(
        &repository,
        expired_token,
        PassengerCounts::new(1, 0, 0).unwrap(),
    )
    .await;
    sqlx::query("UPDATE seat_holds SET expires_at = NOW() WHERE id = $1")
        .bind(expired.id)
        .execute(repository.pool())
        .await
        .unwrap();
    assert!(matches!(
        repository.get_extras(expired.id, expired_token).await,
        Err(ExtraRepositoryError::HoldExpired)
    ));

    let released_token = [74; 32];
    let released = complete_hold(
        &repository,
        released_token,
        PassengerCounts::new(1, 0, 0).unwrap(),
    )
    .await;
    repository
        .release_hold(released.id, released_token)
        .await
        .unwrap();
    assert!(matches!(
        repository.get_extras(released.id, released_token).await,
        Err(ExtraRepositoryError::HoldReleased)
    ));

    let active_token = [75; 32];
    let active = complete_hold(
        &repository,
        active_token,
        PassengerCounts::new(1, 0, 0).unwrap(),
    )
    .await;
    assert!(matches!(
        repository.get_extras(active.id, [76; 32]).await,
        Err(ExtraRepositoryError::Unauthorized)
    ));
}
