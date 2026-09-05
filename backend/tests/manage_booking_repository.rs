use std::{
    env,
    sync::{Arc, LazyLock},
    time::Duration as StdDuration,
};

use tokio::sync::{Mutex, MutexGuard};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::{Datelike, Duration, Utc};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use x_fly_api::{
    domain::{
        entities::{CreateSeatHold, FlightSelection},
        extras::ExtraSelectionInput,
        manage_booking::{BookingStatus, ManageBookingLookup, TravelDocumentStatus},
        passengers::{Gender, PassengerInput, PassengerType, Title},
        payment::{
            PaymentAttemptTransition, PaymentMethod, PaymentProvider, PaymentRepositoryCommand,
            PaymentStatus,
        },
        repositories::{
            ExtraRepository, ManageBookingRepository, ManageBookingRepositoryError,
            PassengerRepository, PaymentRepository, ReviewRepository, SeatHoldRepository,
            TicketRepository,
        },
        value_objects::{CabinClass, PassengerCounts, SeatNumber},
    },
    infrastructure::http::build_router,
    infrastructure::{
        database::{prepare_database, SqlxSeatHoldRepository},
        manage_booking::access::sign as sign_manage_booking_access,
    },
    state::AppState,
};

const MANAGE_SECRET: &str = "manage-booking-http-secret-that-is-at-least-32-characters";
const TICKET_SECRET: &str = "ticket-http-secret-that-is-at-least-32-characters";

const FIXTURE_REFERENCE_PREFIX: &str = "MB-";
const TIMEZONE_FIXTURE_PREFIX: &str = "test-time-zone-";

// Fixture-backed tests share one test database and run in parallel inside this binary,
// so they serialize on this lock; the sweep can then never touch a concurrent test's rows.
static FIXTURE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

async fn fixture_guard() -> MutexGuard<'static, ()> {
    FIXTURE_LOCK.lock().await
}

// Restores every Manage Booking test fixture to its pre-test state. Runs while the
// fixture lock is held: at test start it heals rows a previously panicked run leaked,
// at test end it leaves the shared database exactly as the test found it.
async fn sweep_manage_booking_fixtures(pool: &PgPool) {
    let mut transaction = pool.begin().await.unwrap();
    sqlx::query(
        "UPDATE flight_seats AS seat
         SET booking_status = 'AVAILABLE', booked_at = NULL, updated_at = NOW()
         WHERE seat.id IN (
              SELECT finalized.flight_seat_id
              FROM payment_attempt_seats AS finalized
              JOIN payment_attempts AS attempt ON attempt.id = finalized.payment_attempt_id
              WHERE attempt.provider_reference LIKE $1
         )",
    )
    .bind(format!("{FIXTURE_REFERENCE_PREFIX}%"))
    .execute(&mut *transaction)
    .await
    .unwrap();
    sqlx::query(
        "DELETE FROM tickets WHERE payment_attempt_id IN (
             SELECT id FROM payment_attempts WHERE provider_reference LIKE $1)",
    )
    .bind(format!("{FIXTURE_REFERENCE_PREFIX}%"))
    .execute(&mut *transaction)
    .await
    .unwrap();
    sqlx::query(
        "DELETE FROM seat_holds WHERE id IN (
             SELECT seat_hold_id FROM payment_attempts WHERE provider_reference LIKE $1)",
    )
    .bind(format!("{FIXTURE_REFERENCE_PREFIX}%"))
    .execute(&mut *transaction)
    .await
    .unwrap();
    sqlx::query(
        "DELETE FROM flight_instances WHERE flight_service_id IN (
             SELECT id FROM flight_services WHERE public_id LIKE $1)",
    )
    .bind(format!("{TIMEZONE_FIXTURE_PREFIX}%"))
    .execute(&mut *transaction)
    .await
    .unwrap();
    sqlx::query("DELETE FROM flight_services WHERE public_id LIKE $1")
        .bind(format!("{TIMEZONE_FIXTURE_PREFIX}%"))
        .execute(&mut *transaction)
        .await
        .unwrap();
    transaction.commit().await.unwrap();
}

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

fn passenger(ordinal: u8, email: &str) -> PassengerInput {
    PassengerInput {
        ordinal,
        passenger_type: PassengerType::Adult,
        title: Title::Ms,
        given_name: if ordinal == 1 { "Nara" } else { "Mali" }.to_owned(),
        middle_name: None,
        family_name: "Van der Meer".to_owned(),
        date_of_birth: Utc::now()
            .date_naive()
            .with_year(Utc::now().year() - 30)
            .unwrap(),
        gender: Gender::Female,
        nationality_code: "TH".to_owned(),
        passport_number: format!("MB{:08X}", Uuid::new_v4().as_u128() as u32),
        passport_issuing_country_code: "TH".to_owned(),
        email: email.to_owned(),
        phone_country_code: "+66".to_owned(),
        phone_number: "812345678".to_owned(),
        emergency_contact: None,
    }
}

async fn issued_booking(repository: &SqlxSeatHoldRepository) -> (Uuid, String) {
    issued_booking_for(repository, PaymentMethod::Bitcoin).await
}

async fn issued_booking_for(
    repository: &SqlxSeatHoldRepository,
    method: PaymentMethod,
) -> (Uuid, String) {
    let token = [141; 32];
    let departure =
        Utc::now().date_naive() + Duration::days(60 + (Uuid::new_v4().as_u128() % 20_000) as i64);
    let hold = repository
        .create_hold(
            CreateSeatHold {
                selection: FlightSelection {
                    flight_id: "xf-201".to_owned(),
                    departure_date: departure,
                    cabin: CabinClass::Economy,
                },
                passengers: PassengerCounts::new(2, 0, 0).unwrap(),
                seats: vec![
                    SeatNumber::parse("20A").unwrap(),
                    SeatNumber::parse("20B").unwrap(),
                ],
                token_hash: token,
            },
            StdDuration::from_secs(600),
        )
        .await
        .unwrap();
    repository
        .save_passengers(
            hold.id,
            token,
            vec![
                passenger(1, "first@example.com"),
                passenger(2, "second@example.com"),
            ],
        )
        .await
        .unwrap();
    repository
        .save_extras(
            hold.id,
            token,
            vec![
                ExtraSelectionInput {
                    passenger_ordinal: 1,
                    product_code: "BAG_30KG".to_owned(),
                    quantity: 1,
                },
                ExtraSelectionInput {
                    passenger_ordinal: 2,
                    product_code: "MEAL_VEGETARIAN".to_owned(),
                    quantity: 1,
                },
            ],
        )
        .await
        .unwrap();
    repository.get_review(hold.id, token).await.unwrap();
    let provider_reference = format!("MB-{method:?}-{}", Uuid::new_v4().simple());
    let attempt = repository
        .create_payment_attempt(
            hold.id,
            token,
            PaymentRepositoryCommand {
                request_id: Uuid::new_v4(),
                request_fingerprint: [8; 32],
                method,
                provider: match method {
                    PaymentMethod::Card => PaymentProvider::Stripe,
                    PaymentMethod::Bitcoin => PaymentProvider::MockBitcoin,
                },
            },
        )
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold.id,
            token,
            attempt.id,
            PaymentAttemptTransition::processing(format!("{provider_reference}-PROCESSING")),
        )
        .await
        .unwrap();
    repository
        .transition_payment_attempt(
            hold.id,
            token,
            attempt.id,
            PaymentAttemptTransition::succeeded(format!("{provider_reference}-SUCCEEDED")),
        )
        .await
        .unwrap();
    let ticket = repository
        .issue_ticket(hold.id, token, attempt.id)
        .await
        .unwrap();
    (ticket.id, ticket.booking_reference)
}

async fn response_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

#[tokio::test]
async fn timezone_migration_keeps_unknown_existing_origins_upgradeable() {
    let _fixtures = fixture_guard().await;
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    sweep_manage_booking_fixtures(repository.pool()).await;
    let suffix = Uuid::new_v4().simple().to_string();
    let public_id = format!("test-time-zone-{}", &suffix[..12]);
    let flight_number = format!("TZ {}", &suffix[..8]);

    let time_zone: Option<String> = sqlx::query_scalar(
        "INSERT INTO flight_services (
            public_id, flight_number, origin_code, destination_code, aircraft_code,
            departure_time
         ) VALUES ($1, $2, 'SYD', 'BKK', 'Boeing 787-9', '10:00')
         RETURNING origin_time_zone",
    )
    .bind(public_id)
    .bind(flight_number)
    .fetch_one(repository.pool())
    .await
    .unwrap();

    assert!(time_zone.is_none());
    sweep_manage_booking_fixtures(repository.pool()).await;
}

#[tokio::test]
async fn lookup_reads_authoritative_state_and_same_passenger_identity() {
    let _fixtures = fixture_guard().await;
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    sweep_manage_booking_fixtures(repository.pool()).await;
    let (ticket_id, reference) = issued_booking(&repository).await;
    let before: (String, Option<chrono::DateTime<Utc>>, i64, i64) = sqlx::query_as(
        "SELECT attempt.status, hold.consumed_at,
            (SELECT COUNT(*) FROM payment_attempt_seats WHERE payment_attempt_id = attempt.id),
            (SELECT COUNT(*) FROM tickets WHERE payment_attempt_id = attempt.id)
         FROM tickets AS ticket
         JOIN payment_attempts AS attempt ON attempt.id = ticket.payment_attempt_id
         JOIN seat_holds AS hold ON hold.id = attempt.seat_hold_id
         WHERE ticket.id = $1",
    )
    .bind(ticket_id)
    .fetch_one(repository.pool())
    .await
    .unwrap();

    let lookup =
        ManageBookingLookup::new(reference.to_ascii_lowercase(), " van  DER meer ".to_owned())
            .unwrap();
    let record = repository
        .lookup_manage_booking(&lookup, Utc::now())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(record.ticket_id, ticket_id);
    assert_eq!(record.booking.booking_reference, reference);
    assert_eq!(record.booking.status, BookingStatus::Confirmed);
    assert_eq!(record.booking.payment.status, PaymentStatus::Succeeded);
    assert_eq!(record.booking.journey.flight_number, "XF 201");
    assert_eq!(record.booking.journey.origin_code, "BKK");
    assert_eq!(record.booking.seats, vec!["20A", "20B"]);
    assert_eq!(record.booking.passengers.len(), 2);
    assert!(record
        .booking
        .passengers
        .iter()
        .all(|passenger| passenger.travel_document_status == TravelDocumentStatus::Complete));
    assert_eq!(record.booking.extras.len(), 2);
    assert!(record.booking.cancellation.cutoff_at.is_some());

    let after: (String, Option<chrono::DateTime<Utc>>, i64, i64) = sqlx::query_as(
        "SELECT attempt.status, hold.consumed_at,
            (SELECT COUNT(*) FROM payment_attempt_seats WHERE payment_attempt_id = attempt.id),
            (SELECT COUNT(*) FROM tickets WHERE payment_attempt_id = attempt.id)
         FROM tickets AS ticket
         JOIN payment_attempts AS attempt ON attempt.id = ticket.payment_attempt_id
         JOIN seat_holds AS hold ON hold.id = attempt.seat_hold_id
         WHERE ticket.id = $1",
    )
    .bind(ticket_id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    assert_eq!(after, before);

    let wrong_name = ManageBookingLookup::new(reference, "Different".to_owned()).unwrap();
    assert!(repository
        .lookup_manage_booking(&wrong_name, Utc::now())
        .await
        .unwrap()
        .is_none());

    let normalized_name = ManageBookingLookup::new(
        record.booking.booking_reference.clone(),
        "VAN DER MEER".to_owned(),
    )
    .unwrap();
    assert!(repository
        .lookup_manage_booking(&normalized_name, Utc::now())
        .await
        .unwrap()
        .is_some());
    sweep_manage_booking_fixtures(repository.pool()).await;
}

#[tokio::test]
async fn stripe_and_mock_bitcoin_successes_are_both_manageable() {
    let _fixtures = fixture_guard().await;
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    sweep_manage_booking_fixtures(repository.pool()).await;
    for method in [PaymentMethod::Card, PaymentMethod::Bitcoin] {
        let (_, reference) = issued_booking_for(&repository, method).await;
        let lookup = ManageBookingLookup::new(reference, "Van der Meer".to_owned()).unwrap();
        let booking = repository
            .lookup_manage_booking(&lookup, Utc::now())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(booking.booking.payment.status, PaymentStatus::Succeeded);
    }
    sweep_manage_booking_fixtures(repository.pool()).await;
}

#[tokio::test]
async fn cancelled_ticket_is_presented_as_cancelled_without_changing_payment() {
    let _fixtures = fixture_guard().await;
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    sweep_manage_booking_fixtures(repository.pool()).await;
    let (ticket_id, _) = issued_booking(&repository).await;
    sqlx::query("UPDATE tickets SET status = 'CANCELLED', cancelled_at = NOW() WHERE id = $1")
        .bind(ticket_id)
        .execute(repository.pool())
        .await
        .unwrap();

    let record = repository
        .get_manage_booking(ticket_id, Utc::now())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(record.booking.status, BookingStatus::Cancelled);
    assert_eq!(
        record.booking.ticket.status,
        x_fly_api::domain::ticket::TicketStatus::Cancelled
    );
    assert_eq!(record.booking.payment.status, PaymentStatus::Succeeded);
    assert_eq!(
        record.booking.cancellation.eligibility,
        x_fly_api::domain::manage_booking::CancellationEligibility::Unavailable
    );
    sweep_manage_booking_fixtures(repository.pool()).await;
}

#[tokio::test]
async fn rejects_inconsistent_successful_finalization_state() {
    let _fixtures = fixture_guard().await;
    let repository = SqlxSeatHoldRepository::new(test_pool().await);
    sweep_manage_booking_fixtures(repository.pool()).await;
    let (ticket_id, _) = issued_booking(&repository).await;
    let hold_id: Uuid = sqlx::query_scalar(
        "SELECT attempt.seat_hold_id FROM tickets AS ticket
         JOIN payment_attempts AS attempt ON attempt.id = ticket.payment_attempt_id
         WHERE ticket.id = $1",
    )
    .bind(ticket_id)
    .fetch_one(repository.pool())
    .await
    .unwrap();

    sqlx::query("UPDATE seat_holds SET consumed_at = NULL WHERE id = $1")
        .bind(hold_id)
        .execute(repository.pool())
        .await
        .unwrap();
    assert!(matches!(
        repository.get_manage_booking(ticket_id, Utc::now()).await,
        Err(ManageBookingRepositoryError::InconsistentState)
    ));

    sqlx::query("UPDATE seat_holds SET consumed_at = NOW() WHERE id = $1")
        .bind(hold_id)
        .execute(repository.pool())
        .await
        .unwrap();
    let removed_seat_id: Uuid = sqlx::query_scalar(
        "SELECT finalized.flight_seat_id FROM payment_attempt_seats AS finalized
         JOIN tickets AS ticket ON ticket.payment_attempt_id = finalized.payment_attempt_id
         WHERE ticket.id = $1 LIMIT 1",
    )
    .bind(ticket_id)
    .fetch_one(repository.pool())
    .await
    .unwrap();
    sqlx::query("DELETE FROM payment_attempt_seats WHERE flight_seat_id = $1")
        .bind(removed_seat_id)
        .execute(repository.pool())
        .await
        .unwrap();
    assert!(matches!(
        repository.get_manage_booking(ticket_id, Utc::now()).await,
        Err(ManageBookingRepositoryError::InconsistentState)
    ));
    // The sweep identifies finalized seats through payment_attempt_seats, so the row
    // deleted above must be restored before cleanup or its seat would stay BOOKED.
    sqlx::query(
        "UPDATE flight_seats SET booking_status = 'AVAILABLE', booked_at = NULL, updated_at = NOW()
         WHERE id = $1",
    )
    .bind(removed_seat_id)
    .execute(repository.pool())
    .await
    .unwrap();
    sweep_manage_booking_fixtures(repository.pool()).await;
}

#[tokio::test]
async fn http_lookup_establishes_scoped_private_access_with_generic_failures() {
    let _fixtures = fixture_guard().await;
    let repository = Arc::new(SqlxSeatHoldRepository::new(test_pool().await));
    sweep_manage_booking_fixtures(repository.pool()).await;
    let (ticket_id, reference) = issued_booking(&repository).await;
    let app = build_router(
        AppState::new(
            repository.clone(),
            repository.clone(),
            repository.clone(),
            repository.clone(),
            StdDuration::from_secs(600),
            false,
            "http://localhost:3000".to_owned(),
        )
        .with_tickets(repository.clone(), TICKET_SECRET.to_owned())
        .with_manage_bookings(repository.clone(), MANAGE_SECRET.to_owned()),
    );

    let lookup = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/manage-booking/lookup")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "bookingReference": reference,
                        "lastName": "VAN DER MEER"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(lookup.status(), StatusCode::OK);
    assert_eq!(lookup.headers()[header::CACHE_CONTROL], "no-store, private");
    let cookie = lookup.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let cookie_header = lookup.headers()[header::SET_COOKIE].to_str().unwrap();
    assert!(cookie_header.contains("HttpOnly"));
    assert!(cookie_header.contains("SameSite=Lax"));
    assert!(cookie_header.contains("Path=/api/v1/manage-booking"));
    let lookup_body = response_body(lookup).await;
    assert_eq!(lookup_body["payment"]["status"], "SUCCEEDED");
    assert_eq!(lookup_body["ticket"]["status"], "ISSUED");
    let serialized = lookup_body.to_string();
    for secret in [
        "second@example.com",
        "812345678",
        "providerReference",
        "clientPaymentSession",
        "passportNumber",
    ] {
        assert!(!serialized.contains(secret));
    }

    let current = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/manage-booking/current")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(current.status(), StatusCode::OK);
    assert_eq!(
        current.headers()[header::CACHE_CONTROL],
        "no-store, private"
    );
    let current_body = response_body(current).await;
    assert_eq!(current_body["bookingReference"], reference);
    assert!(current_body.get("checkIn").is_none());
    assert!(current_body.get("id").is_none());
    assert_eq!(
        x_fly_api::infrastructure::ticket::qr::verify(
            current_body["qrToken"].as_str().unwrap(),
            TICKET_SECRET
        )
        .unwrap(),
        ticket_id
    );

    let ticket = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/manage-booking/current/ticket")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ticket.status(), StatusCode::OK);
    let ticket_body = response_body(ticket).await;
    assert_eq!(ticket_body["ticket"]["id"], ticket_id.to_string());
    assert!(ticket_body["qrToken"].as_str().is_some());
    let ticket_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tickets WHERE id = $1")
        .bind(ticket_id)
        .fetch_one(repository.pool())
        .await
        .unwrap();
    assert_eq!(ticket_count, 1);

    let unauthorized = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/manage-booking/current")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    for invalid_cookie in [
        format!("{cookie}x"),
        format!(
            "x_fly_manage_booking={}",
            sign_manage_booking_access(
                ticket_id,
                Utc::now() - Duration::seconds(1),
                [4; 16],
                MANAGE_SECRET,
            )
            .unwrap()
        ),
    ] {
        let rejected = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/manage-booking/current")
                    .header(header::COOKIE, invalid_cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response_body(rejected).await["error"]["code"],
            "MANAGE_BOOKING_UNAUTHORIZED"
        );
    }

    let unprotected_id_route = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/bookings/{ticket_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unprotected_id_route.status(), StatusCode::NOT_FOUND);

    let mut failures = Vec::new();
    for body in [
        json!({ "bookingReference": "XF22222222", "lastName": "VAN DER MEER" }),
        json!({ "bookingReference": reference, "lastName": "WRONG" }),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/manage-booking/lookup")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        failures.push(response_body(response).await);
    }
    assert_eq!(failures[0], failures[1]);
    assert_eq!(failures[0]["error"]["code"], "BOOKING_NOT_FOUND");

    // A successful lookup must replace the old cookie, even when it is valid.
    let (next_ticket_id, next_reference) =
        issued_booking_for(&repository, PaymentMethod::Card).await;
    let next_lookup = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/manage-booking/lookup")
                .header(header::COOKIE, &cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "bookingReference": next_reference, "lastName": "VAN DER MEER" })
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(next_lookup.status(), StatusCode::OK);
    let next_cookie = next_lookup.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    assert_ne!(next_cookie, cookie);
    assert_eq!(
        response_body(next_lookup).await["bookingReference"],
        next_reference
    );
    let next_current = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/manage-booking/current")
                .header(header::COOKIE, &next_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(next_current.status(), StatusCode::OK);
    let next_body = response_body(next_current).await;
    assert_eq!(next_body["bookingReference"], next_reference);
    assert_ne!(
        next_body["bookingReference"],
        current_body["bookingReference"]
    );
    assert_eq!(
        x_fly_api::infrastructure::ticket::qr::verify(
            next_body["qrToken"].as_str().unwrap(),
            TICKET_SECRET
        )
        .unwrap(),
        next_ticket_id
    );
    assert!(!next_body.to_string().contains(&reference));
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tickets WHERE id = $1 OR id = $2")
        .bind(ticket_id)
        .bind(next_ticket_id)
        .fetch_one(repository.pool())
        .await
        .unwrap();
    assert_eq!(count, 2);
    sweep_manage_booking_fixtures(repository.pool()).await;
}
