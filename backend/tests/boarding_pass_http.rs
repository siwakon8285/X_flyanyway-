mod common;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use http_body_util::BodyExt;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{future::Future, sync::Arc, time::Duration};
use tower::ServiceExt;
use uuid::Uuid;
use x_fly_api::{
    application::staff_auth::StaffAuthService,
    infrastructure::{
        database::{
            prepare_test_database, verify_database_ready, SqlxSeatHoldRepository,
            SqlxStaffAuthRepository,
        },
        http::build_router,
        password::Argon2PasswordService,
    },
    state::AppState,
};

const SIGNING_SECRET: &str = "synthetic-ticket-signing-secret-at-least-32-characters";

async fn pool() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&common::test_database_url())
        .await
        .unwrap();
    prepare_test_database(&pool).await.unwrap();
    pool
}

async fn restricted_runtime_pool() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&common::test_runtime_database_url())
        .await
        .unwrap();
    verify_database_ready(&pool).await.unwrap();
    pool
}

fn app(pool: PgPool) -> axum::Router {
    let repository = Arc::new(SqlxSeatHoldRepository::new(pool.clone()));
    let auth = StaffAuthService::new(
        Arc::new(SqlxStaffAuthRepository::new(pool)),
        Argon2PasswordService::default(),
        Duration::from_secs(3600),
    )
    .unwrap();
    build_router(
        AppState::new(
            repository.clone(),
            repository.clone(),
            repository.clone(),
            repository.clone(),
            Duration::from_secs(600),
            true,
            "http://localhost:3000".into(),
        )
        .with_staff_auth(auth)
        .with_tickets(repository.clone(), SIGNING_SECRET.into())
        .with_ticket_operations(repository.clone())
        .with_boarding_passes(repository, SIGNING_SECRET.into()),
    )
}

async fn staff_cookie(pool: &PgPool, role: &str) -> String {
    let staff_id: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users(email,password_hash) VALUES($1,'synthetic-hash') RETURNING id",
    )
    .bind(format!(
        "boarding-pass-{}@x-fly.test",
        Uuid::new_v4().simple()
    ))
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO staff_user_roles(staff_user_id,role_code) VALUES($1,$2)")
        .bind(staff_id)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
    let token = Sha256::digest(format!("boarding-pass-{}", Uuid::new_v4()).as_bytes());
    let hash: [u8; 32] = Sha256::digest(token).into();
    sqlx::query("INSERT INTO staff_sessions(staff_user_id,token_hash,expires_at) VALUES($1,$2,NOW()+INTERVAL '1 hour')")
        .bind(staff_id)
        .bind(hash.as_slice())
        .execute(pool)
        .await
        .unwrap();
    format!("x_fly_staff_session={}", hex::encode(token))
}

#[derive(Clone)]
struct Fixture {
    public_id: String,
    ticket_number: String,
    passenger_ordinal: i16,
}

async fn fixture(pool: &PgPool, departure_in: ChronoDuration, with_seat: bool) -> Fixture {
    let service_id = Uuid::new_v4();
    let now: DateTime<Utc> = sqlx::query_scalar("SELECT NOW()")
        .fetch_one(pool)
        .await
        .unwrap();
    let local_departure = now + departure_in + ChronoDuration::hours(7);
    let departure_date = local_departure.date_naive();
    let departure_time = local_departure.time();
    let identity = Uuid::new_v4().simple().to_string().to_uppercase();
    let alpha = identity
        .chars()
        .map(|c| {
            if matches!(c, '0' | '1' | 'I' | 'O') {
                '2'
            } else {
                c
            }
        })
        .collect::<String>();
    let ticket_number = format!("XFT{}", &alpha[..12]);
    let booking_reference = format!("XF{}", &alpha[..8]);
    let public_id = format!("boarding-pass-{}", &alpha[..12]);
    let flight_number = format!(
        "XF {:03}",
        100 + (u32::from_str_radix(&identity[..6], 16).unwrap() % 900)
    );

    sqlx::query("INSERT INTO flight_services(id,public_id,flight_number,origin_code,destination_code,aircraft_code,origin_time_zone,departure_time,arrival_time,arrival_day_offset,duration_minutes,stops,status,operating_date) VALUES($1,$2,$3,'BKK','LHR','Airbus A350-1000','Asia/Bangkok',$4,'17:00',0,420,'DIRECT','SCHEDULED',$5)")
        .bind(service_id)
        .bind(&public_id)
        .bind(flight_number)
        .bind(departure_time)
        .bind(departure_date)
        .execute(pool)
        .await
        .unwrap();
    let instance_id: Uuid = sqlx::query_scalar(
        "INSERT INTO flight_instances(flight_service_id,departure_date) VALUES($1,$2) RETURNING id",
    )
    .bind(service_id)
    .bind(departure_date)
    .fetch_one(pool)
    .await
    .unwrap();
    let hold_id: Uuid = sqlx::query_scalar("INSERT INTO seat_holds(flight_instance_id,cabin,adults,children,infants,access_token_hash,expires_at,consumed_at,passenger_details_saved_at) VALUES($1,'business',1,0,0,$2,NOW()+INTERVAL '1 hour',NOW(),NOW()) RETURNING id")
        .bind(instance_id)
        .bind([3_u8; 32].as_slice())
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO hold_passengers(seat_hold_id,ordinal,passenger_type,title,given_name,family_name,date_of_birth,gender,nationality_code,passport_number,passport_issuing_country_code) VALUES($1,1,'ADULT','MR','Arun','Synthetic','1990-01-01','MALE','TH',$2,'TH')")
        .bind(hold_id)
        .bind(format!("SYN{}", &alpha[..8]))
        .execute(pool)
        .await
        .unwrap();
    let attempt_id: Uuid = sqlx::query_scalar("INSERT INTO payment_attempts(seat_hold_id,request_id,request_fingerprint,provider,payment_method,status,amount,currency_code,review_priced_at,provider_reference,succeeded_at) VALUES($1,$2,$3,'MOCK_BITCOIN','BITCOIN','SUCCEEDED',99500,'THB',NOW(),$4,NOW()) RETURNING id")
        .bind(hold_id)
        .bind(Uuid::new_v4())
        .bind([4_u8; 32].as_slice())
        .bind(format!("SYN-{}", &alpha[..8]))
        .fetch_one(pool)
        .await
        .unwrap();
    if with_seat {
        let seat_id: Uuid = sqlx::query_scalar("INSERT INTO flight_seats(flight_instance_id,seat_number,row_number,column_code,cabin,position,booking_status,booked_at) VALUES($1,'1A',1,'A','business','window','BOOKED',NOW()) RETURNING id")
            .bind(instance_id)
            .fetch_one(pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO payment_attempt_seats(payment_attempt_id,flight_seat_id,passenger_ordinal) VALUES($1,$2,1)")
            .bind(attempt_id)
            .bind(seat_id)
            .execute(pool)
            .await
            .unwrap();
    }
    sqlx::query("INSERT INTO tickets(payment_attempt_id,booking_reference,ticket_number,status) VALUES($1,$2,$3,'ISSUED')")
        .bind(attempt_id)
        .bind(booking_reference)
        .bind(&ticket_number)
        .execute(pool)
        .await
        .unwrap();

    Fixture {
        public_id,
        ticket_number,
        passenger_ordinal: 1,
    }
}

async fn cleanup_fixture(pool: &PgPool, _public_id: &str) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(
        "DELETE FROM boarding_pass_operations_audit WHERE ticket_id IN (
             SELECT ticket.id FROM tickets ticket
             JOIN payment_attempts attempt ON attempt.id=ticket.payment_attempt_id
             JOIN seat_holds hold ON hold.id=attempt.seat_hold_id
             JOIN flight_instances instance ON instance.id=hold.flight_instance_id
             JOIN flight_services service ON service.id=instance.flight_service_id
             WHERE service.public_id LIKE 'boarding-pass-%'
         );
         DELETE FROM boarding_passes WHERE ticket_id IN (
             SELECT ticket.id FROM tickets ticket
             JOIN payment_attempts attempt ON attempt.id=ticket.payment_attempt_id
             JOIN seat_holds hold ON hold.id=attempt.seat_hold_id
             JOIN flight_instances instance ON instance.id=hold.flight_instance_id
             JOIN flight_services service ON service.id=instance.flight_service_id
             WHERE service.public_id LIKE 'boarding-pass-%'
         );
         DELETE FROM tickets WHERE id IN (
             SELECT ticket.id FROM tickets ticket
             JOIN payment_attempts attempt ON attempt.id=ticket.payment_attempt_id
             JOIN seat_holds hold ON hold.id=attempt.seat_hold_id
             JOIN flight_instances instance ON instance.id=hold.flight_instance_id
             JOIN flight_services service ON service.id=instance.flight_service_id
             WHERE service.public_id LIKE 'boarding-pass-%'
         );
         DELETE FROM payment_attempt_seats WHERE payment_attempt_id IN (
             SELECT attempt.id FROM payment_attempts attempt
             JOIN seat_holds hold ON hold.id=attempt.seat_hold_id
             JOIN flight_instances instance ON instance.id=hold.flight_instance_id
             JOIN flight_services service ON service.id=instance.flight_service_id
             WHERE service.public_id LIKE 'boarding-pass-%'
         );
         DELETE FROM payment_attempts WHERE seat_hold_id IN (
             SELECT hold.id FROM seat_holds hold
             JOIN flight_instances instance ON instance.id=hold.flight_instance_id
             JOIN flight_services service ON service.id=instance.flight_service_id
             WHERE service.public_id LIKE 'boarding-pass-%'
         );
         DELETE FROM hold_passengers WHERE seat_hold_id IN (
             SELECT hold.id FROM seat_holds hold
             JOIN flight_instances instance ON instance.id=hold.flight_instance_id
             JOIN flight_services service ON service.id=instance.flight_service_id
             WHERE service.public_id LIKE 'boarding-pass-%'
         );
         DELETE FROM flight_seats WHERE flight_instance_id IN (
             SELECT instance.id FROM flight_instances instance
             JOIN flight_services service ON service.id=instance.flight_service_id
             WHERE service.public_id LIKE 'boarding-pass-%'
         );
         DELETE FROM seat_holds WHERE flight_instance_id IN (
             SELECT instance.id FROM flight_instances instance
             JOIN flight_services service ON service.id=instance.flight_service_id
             WHERE service.public_id LIKE 'boarding-pass-%'
         );
         DELETE FROM flight_instances WHERE flight_service_id IN (
             SELECT id FROM flight_services WHERE public_id LIKE 'boarding-pass-%'
         );
         DELETE FROM flight_services WHERE public_id LIKE 'boarding-pass-%';
         DELETE FROM staff_security_audit WHERE actor_staff_user_id IN (SELECT id FROM staff_users WHERE email LIKE 'boarding-pass-%@x-fly.test');
         DELETE FROM staff_sessions WHERE staff_user_id IN (SELECT id FROM staff_users WHERE email LIKE 'boarding-pass-%@x-fly.test');
         DELETE FROM staff_user_roles WHERE staff_user_id IN (SELECT id FROM staff_users WHERE email LIKE 'boarding-pass-%@x-fly.test');
         DELETE FROM staff_users WHERE email LIKE 'boarding-pass-%@x-fly.test';",
    )
    .execute(pool)
    .await?;
    sqlx::raw_sql("DELETE FROM staff_security_audit WHERE actor_staff_user_id IN (SELECT id FROM staff_users WHERE email LIKE 'boarding-pass-%@x-fly.test'); DELETE FROM staff_sessions WHERE staff_user_id IN (SELECT id FROM staff_users WHERE email LIKE 'boarding-pass-%@x-fly.test'); DELETE FROM staff_user_roles WHERE staff_user_id IN (SELECT id FROM staff_users WHERE email LIKE 'boarding-pass-%@x-fly.test'); DELETE FROM staff_users WHERE email LIKE 'boarding-pass-%@x-fly.test';")
        .execute(pool)
        .await?;
    Ok(())
}

async fn with_fixture<F, Fut>(departure_in: ChronoDuration, with_seat: bool, test: F)
where
    F: FnOnce(PgPool, Fixture, String) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let _guard = common::acquire_test_fixture_lock().await;
    let pool = pool().await;
    cleanup_fixture(&pool, "before-test").await.unwrap();
    let cleanup_key = Arc::new(tokio::sync::Mutex::new(None::<String>));
    let body_key = cleanup_key.clone();
    let cleanup_pool = pool.clone();
    common::run_fixture_body_with_cleanup(
        move || async move {
            let fixture = fixture(&pool, departure_in, with_seat).await;
            *body_key.lock().await = Some(fixture.public_id.clone());
            let cookie = staff_cookie(&pool, "TICKET_PASSENGER_OPERATIONS").await;
            test(pool, fixture, cookie).await;
        },
        move || async move {
            if let Some(public_id) = cleanup_key.lock().await.clone() {
                cleanup_fixture(&cleanup_pool, &public_id).await?;
            }
            Ok::<_, sqlx::Error>(())
        },
    )
    .await;
}

async fn send(
    app: &axum::Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
) -> axum::response::Response {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::ORIGIN, "http://localhost:3000")
        .header("x-x-fly-csrf", "1");
    if let Some(cookie) = cookie {
        request = request.header(header::COOKIE, cookie);
    }
    app.clone()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

fn boarding_pass_path(fixture: &Fixture) -> String {
    format!(
        "/api/v1/admin/tickets/{}/passengers/{}/boarding-pass",
        fixture.ticket_number, fixture.passenger_ordinal
    )
}

#[tokio::test]
async fn authorized_operator_can_issue_idempotently_and_detail_is_per_passenger() {
    with_fixture(ChronoDuration::hours(12), true, |pool, fixture, cookie| async move {
        let router = app(pool.clone());
        let path = boarding_pass_path(&fixture);
        let first = send(&router, "POST", &path, Some(&cookie)).await;
        assert_eq!(first.status(), StatusCode::OK);
        let first_body = json_body(first).await;
        assert_eq!(first_body["passengerOrdinal"], 1);
        assert_eq!(first_body["seat"], "1A");
        assert!(first_body["qrToken"].as_str().unwrap().starts_with("v1."));

        let second = send(&router, "POST", &path, Some(&cookie)).await;
        assert_eq!(second.status(), StatusCode::OK);
        let second_body = json_body(second).await;
        assert_eq!(first_body["boardingPassId"], second_body["boardingPassId"]);

        let detail = send(
            &router,
            "GET",
            &format!("/api/v1/admin/tickets/{}", fixture.ticket_number),
            Some(&cookie),
        ).await;
        assert_eq!(detail.status(), StatusCode::OK);
        let detail_body = json_body(detail).await;
        assert_eq!(detail_body["passengers"][0]["checkIn"]["status"], "CHECKED_IN");
        assert_eq!(detail_body["passengers"][0]["boardingPass"]["id"], first_body["boardingPassId"]);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM boarding_passes WHERE ticket_id=(SELECT id FROM tickets WHERE ticket_number=$1)")
            .bind(&fixture.ticket_number).fetch_one(&pool).await.unwrap();
        assert_eq!(count, 1);
        let audit_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM boarding_pass_operations_audit WHERE ticket_id=(SELECT id FROM tickets WHERE ticket_number=$1)")
            .bind(&fixture.ticket_number).fetch_one(&pool).await.unwrap();
        assert_eq!(audit_count, 1);
    }).await;
}

#[tokio::test]
async fn restricted_runtime_can_issue_without_hold_passenger_update_permission() {
    let _guard = common::acquire_test_fixture_lock().await;
    let setup_pool = pool().await;
    let runtime_pool = restricted_runtime_pool().await;
    cleanup_fixture(&setup_pool, "before-test").await.unwrap();
    let body_setup_pool = setup_pool.clone();
    let body_runtime_pool = runtime_pool.clone();
    let cleanup_pool = setup_pool.clone();
    common::run_fixture_body_with_cleanup(
        move || async move {
            let fixture = fixture(&body_setup_pool, ChronoDuration::hours(12), true).await;
            let cookie = staff_cookie(&body_setup_pool, "TICKET_PASSENGER_OPERATIONS").await;

            let current_user: String = sqlx::query_scalar("SELECT current_user")
                .fetch_one(&body_runtime_pool)
                .await
                .unwrap();
            assert_eq!(current_user, "x_fly_runtime");
            let can_select: bool = sqlx::query_scalar(
                "SELECT has_table_privilege(current_user, 'public.hold_passengers'::regclass, 'SELECT')",
            )
            .fetch_one(&body_runtime_pool)
            .await
            .unwrap();
            let can_update: bool = sqlx::query_scalar(
                "SELECT has_table_privilege(current_user, 'public.hold_passengers'::regclass, 'UPDATE')",
            )
            .fetch_one(&body_runtime_pool)
            .await
            .unwrap();
            assert!(can_select);
            assert!(!can_update);

            let update_error = sqlx::query(
                "UPDATE hold_passengers
                 SET given_name = given_name
                 WHERE seat_hold_id = $1 AND ordinal = $2",
            )
            .bind(
                sqlx::query_scalar::<_, Uuid>(
                    "SELECT seat_hold_id FROM tickets
                     JOIN payment_attempts ON payment_attempts.id = tickets.payment_attempt_id
                     WHERE ticket_number = $1",
                )
                .bind(&fixture.ticket_number)
                .fetch_one(&body_setup_pool)
                .await
                .unwrap(),
            )
            .bind(fixture.passenger_ordinal as i16)
            .execute(&body_runtime_pool)
            .await
            .expect_err("x_fly_runtime must not update hold_passengers");
            assert_eq!(
                update_error
                    .as_database_error()
                    .and_then(|error| error.code()),
                Some("42501".into())
            );

            let router = app(body_runtime_pool);
            let response = send(
                &router,
                "POST",
                &boarding_pass_path(&fixture),
                Some(&cookie),
            )
            .await;
            assert_eq!(response.status(), StatusCode::OK);
            let body = json_body(response).await;
            assert_eq!(body["passengerOrdinal"], fixture.passenger_ordinal);

            let counts: (i64, i64) = sqlx::query_as(
                "SELECT
                    (SELECT COUNT(*) FROM boarding_passes
                     WHERE ticket_id = (SELECT id FROM tickets WHERE ticket_number = $1)),
                    (SELECT COUNT(*) FROM boarding_pass_operations_audit
                     WHERE ticket_id = (SELECT id FROM tickets WHERE ticket_number = $1))",
            )
            .bind(&fixture.ticket_number)
            .fetch_one(&body_setup_pool)
            .await
            .unwrap();
            assert_eq!(counts, (1, 1));
        },
        move || async move { cleanup_fixture(&cleanup_pool, "after-test").await },
    )
    .await;
}

#[tokio::test]
async fn timing_and_authoritative_state_reject_invalid_issuance() {
    with_fixture(ChronoDuration::hours(48), true, |pool, fixture, cookie| async move {
        let router = app(pool.clone());
        let too_early = send(&router, "POST", &boarding_pass_path(&fixture), Some(&cookie)).await;
        assert_eq!(too_early.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json_body(too_early).await["error"]["code"], "BOARDING_PASS_CHECK_IN_NOT_OPEN");

        sqlx::query("UPDATE flight_services SET status='CANCELLED',cancelled_at=NOW() WHERE public_id=$1")
            .bind(&fixture.public_id).execute(&pool).await.unwrap();
        let cancelled_flight = send(&router, "POST", &boarding_pass_path(&fixture), Some(&cookie)).await;
        assert_eq!(cancelled_flight.status(), StatusCode::CONFLICT);
        assert_eq!(json_body(cancelled_flight).await["error"]["code"], "BOARDING_PASS_FLIGHT_CANCELLED");
    }).await;

    with_fixture(
        ChronoDuration::hours(12),
        false,
        |pool, fixture, cookie| async move {
            let router = app(pool.clone());
            let no_seat = send(
                &router,
                "POST",
                &boarding_pass_path(&fixture),
                Some(&cookie),
            )
            .await;
            assert_eq!(no_seat.status(), StatusCode::UNPROCESSABLE_ENTITY);
            assert_eq!(
                json_body(no_seat).await["error"]["code"],
                "BOARDING_PASS_NO_SEAT_ASSIGNMENT"
            );
        },
    )
    .await;

    with_fixture(
        ChronoDuration::hours(-1),
        true,
        |pool, fixture, cookie| async move {
            let router = app(pool.clone());
            let departed = send(
                &router,
                "POST",
                &boarding_pass_path(&fixture),
                Some(&cookie),
            )
            .await;
            assert_eq!(departed.status(), StatusCode::CONFLICT);
            assert_eq!(
                json_body(departed).await["error"]["code"],
                "BOARDING_PASS_FLIGHT_DEPARTED"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn authorization_requires_authentication_and_dedicated_issue_permission() {
    with_fixture(
        ChronoDuration::hours(12),
        true,
        |pool, fixture, _operator_cookie| async move {
            let router = app(pool.clone());
            let unauthenticated = send(&router, "POST", &boarding_pass_path(&fixture), None).await;
            assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);
            let system_cookie = staff_cookie(&pool, "SYSTEM_ADMIN").await;
            let system = send(
                &router,
                "POST",
                &boarding_pass_path(&fixture),
                Some(&system_cookie),
            )
            .await;
            assert_eq!(system.status(), StatusCode::FORBIDDEN);
        },
    )
    .await;
}

#[tokio::test]
async fn concurrent_issue_requests_share_one_record_and_one_audit_event() {
    with_fixture(ChronoDuration::hours(12), true, |pool, fixture, cookie| async move {
        let router = app(pool.clone());
        let path = boarding_pass_path(&fixture);
        let (first, second) = tokio::join!(
            send(&router, "POST", &path, Some(&cookie)),
            send(&router, "POST", &path, Some(&cookie)),
        );
        assert_eq!(first.status(), StatusCode::OK);
        assert_eq!(second.status(), StatusCode::OK);
        let first_body = json_body(first).await;
        let second_body = json_body(second).await;
        assert_eq!(first_body["boardingPassId"], second_body["boardingPassId"]);
        let counts: (i64, i64) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM boarding_passes WHERE ticket_id=(SELECT id FROM tickets WHERE ticket_number=$1)), (SELECT COUNT(*) FROM boarding_pass_operations_audit WHERE ticket_id=(SELECT id FROM tickets WHERE ticket_number=$1))")
            .bind(&fixture.ticket_number).fetch_one(&pool).await.unwrap();
        assert_eq!(counts, (1, 1));
    }).await;
}

#[tokio::test]
async fn public_verification_is_purpose_separated_minimized_and_state_aware() {
    with_fixture(ChronoDuration::hours(12), true, |pool, fixture, cookie| async move {
        let router = app(pool.clone());
        let issued = json_body(send(&router, "POST", &boarding_pass_path(&fixture), Some(&cookie)).await).await;
        let token = issued["qrToken"].as_str().unwrap().to_owned();
        let verified = send(&router, "GET", &format!("/api/v1/boarding-passes/verify/{token}"), None).await;
        assert_eq!(verified.status(), StatusCode::OK);
        let verified_body = json_body(verified).await;
        assert_eq!(verified_body["valid"], true);
        assert!(verified_body.get("passengerName").is_none());
        assert!(verified_body.get("passportNumber").is_none());
        assert!(verified_body.get("phone").is_none());
        assert!(verified_body.get("payment").is_none());

        let mut tampered = token.clone();
        let replacement = if tampered.ends_with('0') { '1' } else { '0' };
        tampered.pop();
        tampered.push(replacement);
        let invalid = send(&router, "GET", &format!("/api/v1/boarding-passes/verify/{tampered}"), None).await;
        assert_eq!(invalid.status(), StatusCode::OK);
        assert_eq!(json_body(invalid).await["valid"], false);

        sqlx::query("UPDATE tickets SET status='CANCELLED',cancelled_at=NOW() WHERE ticket_number=$1")
            .bind(&fixture.ticket_number).execute(&pool).await.unwrap();
        let cancelled = send(&router, "GET", &format!("/api/v1/boarding-passes/verify/{token}"), None).await;
        let cancelled_body = json_body(cancelled).await;
        assert_eq!(cancelled_body["valid"], false);
        assert_eq!(cancelled_body["invalidReason"], "TICKET_CANCELLED");
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM boarding_passes WHERE ticket_id=(SELECT id FROM tickets WHERE ticket_number=$1)")
            .bind(&fixture.ticket_number).fetch_one(&pool).await.unwrap();
        assert_eq!(count, 1);
    }).await;
}

#[tokio::test]
async fn public_verification_expires_after_departure() {
    with_fixture(ChronoDuration::hours(12), true, |pool, fixture, cookie| async move {
        let router = app(pool.clone());
        let issued = json_body(send(&router, "POST", &boarding_pass_path(&fixture), Some(&cookie)).await).await;
        let token = issued["qrToken"].as_str().unwrap();
        sqlx::query("UPDATE flight_instances SET departure_date=CURRENT_DATE-1 WHERE id=(SELECT instance.id FROM flight_instances instance JOIN flight_services service ON service.id=instance.flight_service_id WHERE service.public_id=$1)")
            .bind(&fixture.public_id).execute(&pool).await.unwrap();
        let expired = send(&router, "GET", &format!("/api/v1/boarding-passes/verify/{token}"), None).await;
        let body = json_body(expired).await;
        assert_eq!(body["valid"], false);
        assert_eq!(body["invalidReason"], "EXPIRED");
    }).await;
}
