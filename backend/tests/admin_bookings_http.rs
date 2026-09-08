mod common;

use std::{
    sync::{Arc, OnceLock},
    time::Duration as StdDuration,
};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::{Duration, NaiveDate, Utc};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::ServiceExt;
use uuid::Uuid;
use x_fly_api::{
    application::{cancellation::CancellationService, staff_auth::StaffAuthService},
    domain::{
        cancellation::{Clock, StaffCancellationActor},
        repositories::CancellationRepository,
    },
    infrastructure::{
        database::{prepare_database, SqlxSeatHoldRepository, SqlxStaffAuthRepository},
        http::build_router,
        password::Argon2PasswordService,
    },
    state::AppState,
};

async fn fixture_guard() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn test_pool() -> PgPool {
    let url = common::test_database_url();
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&url)
        .await
        .unwrap();
    prepare_database(&pool).await.unwrap();
    cleanup(&pool).await;
    pool
}

async fn cleanup(pool: &PgPool) {
    sqlx::raw_sql(
        "DELETE FROM booking_operations_audit WHERE booking_reference LIKE 'XF22%';
         DELETE FROM booking_cancellations WHERE ticket_id IN (SELECT id FROM tickets WHERE booking_reference LIKE 'XF22%');
         DELETE FROM tickets WHERE booking_reference LIKE 'XF22%';
         DELETE FROM seat_holds WHERE id IN (
             SELECT hold.id FROM seat_holds hold JOIN flight_instances instance ON instance.id=hold.flight_instance_id
             JOIN flight_services service ON service.id=instance.flight_service_id WHERE service.public_id LIKE 'booking-http-%');
         DELETE FROM flight_instances WHERE flight_service_id IN (SELECT id FROM flight_services WHERE public_id LIKE 'booking-http-%');
         DELETE FROM flight_management_audit WHERE flight_service_id IN (SELECT id FROM flight_services WHERE public_id LIKE 'booking-http-%');
         DELETE FROM flight_service_seat_templates WHERE flight_service_id IN (SELECT id FROM flight_services WHERE public_id LIKE 'booking-http-%');
         DELETE FROM flight_service_cabins WHERE flight_service_id IN (SELECT id FROM flight_services WHERE public_id LIKE 'booking-http-%');
         DELETE FROM flight_services WHERE public_id LIKE 'booking-http-%';
         DELETE FROM staff_sessions WHERE staff_user_id IN (SELECT id FROM staff_users WHERE email LIKE '%@booking-http.test');
         DELETE FROM staff_user_roles WHERE staff_user_id IN (SELECT id FROM staff_users WHERE email LIKE '%@booking-http.test');
         DELETE FROM staff_users WHERE email LIKE '%@booking-http.test';",
    ).execute(pool).await.unwrap();
}

fn app(pool: PgPool) -> axum::Router {
    let booking = Arc::new(SqlxSeatHoldRepository::new(pool.clone()));
    let auth = StaffAuthService::new(
        Arc::new(SqlxStaffAuthRepository::new(pool)),
        Argon2PasswordService::default(),
        StdDuration::from_secs(3600),
    )
    .unwrap();
    build_router(
        AppState::new(
            booking.clone(),
            booking.clone(),
            booking.clone(),
            booking.clone(),
            StdDuration::from_secs(600),
            true,
            "http://localhost:3000".to_owned(),
        )
        .with_staff_auth(auth)
        .with_cancellations(CancellationService::new(
            booking.clone(),
            Arc::new(x_fly_api::domain::cancellation::SystemClock),
        ))
        .with_booking_management(booking),
    )
}

async fn cookie(pool: &PgPool, role: &str, suffix: &str) -> String {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users(email,password_hash) VALUES($1,'synthetic-hash') RETURNING id",
    )
    .bind(format!("{suffix}@booking-http.test"))
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO staff_user_roles(staff_user_id,role_code) VALUES($1,$2)")
        .bind(id)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
    let token = Sha256::digest(format!("booking-http-{suffix}").as_bytes());
    let hash: [u8; 32] = Sha256::digest(token).into();
    sqlx::query("INSERT INTO staff_sessions(staff_user_id,token_hash,expires_at) VALUES($1,$2,NOW()+INTERVAL '1 hour')")
        .bind(id).bind(hash.as_slice()).execute(pool).await.unwrap();
    format!("x_fly_staff_session={}", hex::encode(token))
}

async fn send(
    router: &axum::Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
    trusted: bool,
) -> axum::response::Response {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    if trusted {
        builder = builder
            .header(header::ORIGIN, "http://localhost:3000")
            .header("x-x-fly-csrf", "1");
    }
    router
        .clone()
        .oneshot(
            builder
                .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

struct BookingFixture {
    ticket_id: Uuid,
    reference: String,
    departure_at: chrono::DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)] // Explicit scenario values keep booking fixtures readable at call sites.
async fn booking_fixture(
    pool: &PgPool,
    suffix: &str,
    reference: &str,
    flight_number: &str,
    cabin: &str,
    flight_status: &str,
    amount: i64,
    passenger_name: (&str, &str),
    days: i64,
) -> BookingFixture {
    let service_id = Uuid::new_v4();
    let cancelled_at = (flight_status == "CANCELLED").then(Utc::now);
    sqlx::query("INSERT INTO flight_services(id,public_id,flight_number,origin_code,destination_code,aircraft_code,origin_time_zone,departure_time,arrival_time,arrival_day_offset,duration_minutes,stops,status,operating_date,cancelled_at) VALUES($1,$2,$3,'SYD','CDG','Airbus A350-1000','Australia/Sydney','10:00','19:00',0,1200,'DIRECT',$4,$5,$6)")
        .bind(service_id).bind(format!("booking-http-{suffix}")).bind(flight_number).bind(flight_status)
        .bind(Utc::now().date_naive() + Duration::days(days)).bind(cancelled_at).execute(pool).await.unwrap();
    let travel_date: NaiveDate = Utc::now().date_naive() + Duration::days(days);
    let instance_id: Uuid = sqlx::query_scalar(
        "INSERT INTO flight_instances(flight_service_id,departure_date) VALUES($1,$2) RETURNING id",
    )
    .bind(service_id)
    .bind(travel_date)
    .fetch_one(pool)
    .await
    .unwrap();
    let hold_id: Uuid = sqlx::query_scalar("INSERT INTO seat_holds(flight_instance_id,cabin,adults,children,infants,access_token_hash,expires_at,consumed_at,passenger_details_saved_at) VALUES($1,$2,1,0,0,$3,NOW()+INTERVAL '1 hour',NOW(),NOW()) RETURNING id")
        .bind(instance_id).bind(cabin).bind([7_u8;32].as_slice()).fetch_one(pool).await.unwrap();
    sqlx::query("INSERT INTO hold_passengers(seat_hold_id,ordinal,passenger_type,title,given_name,family_name,date_of_birth,gender,nationality_code,passport_number,passport_issuing_country_code,passport_expiry_date,phone_country_code,phone_number) VALUES($1,1,'ADULT','MS',$2,$3,'1990-01-01','FEMALE','TH',$4,'TH','2099-01-01','+66','812345678')")
        .bind(hold_id).bind(passenger_name.0).bind(passenger_name.1).bind(format!("SYN{}", &Uuid::new_v4().simple().to_string()[..8]).to_uppercase()).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO booking_contacts(seat_hold_id,phone_country_code,phone_number,preferred_locale) VALUES($1,'+66','812345678','EN')").bind(hold_id).execute(pool).await.unwrap();
    let seat_id: Uuid = sqlx::query_scalar("INSERT INTO flight_seats(flight_instance_id,seat_number,row_number,column_code,cabin,position,booking_status,booked_at) VALUES($1,'1K',1,'K',$2,'window','BOOKED',NOW()) RETURNING id")
        .bind(instance_id).bind(cabin).fetch_one(pool).await.unwrap();
    let attempt_id: Uuid = sqlx::query_scalar("INSERT INTO payment_attempts(seat_hold_id,request_id,request_fingerprint,provider,payment_method,status,amount,currency_code,review_priced_at,provider_reference,succeeded_at) VALUES($1,$2,$3,'MOCK_BITCOIN','BITCOIN','SUCCEEDED',$4,'THB',NOW(),$5,NOW()) RETURNING id")
        .bind(hold_id).bind(Uuid::new_v4()).bind([8_u8;32].as_slice()).bind(amount).bind(format!("BK22-{suffix}")).fetch_one(pool).await.unwrap();
    sqlx::query(
        "INSERT INTO payment_attempt_seats(payment_attempt_id,flight_seat_id) VALUES($1,$2)",
    )
    .bind(attempt_id)
    .bind(seat_id)
    .execute(pool)
    .await
    .unwrap();
    let alphabet = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let digest = Sha256::digest(suffix.as_bytes());
    let ticket_number = format!(
        "XFT{}",
        digest[..12]
            .iter()
            .map(|byte| alphabet[*byte as usize % alphabet.len()] as char)
            .collect::<String>()
    );
    let ticket_id: Uuid = sqlx::query_scalar("INSERT INTO tickets(payment_attempt_id,booking_reference,ticket_number,status) VALUES($1,$2,$3,'ISSUED') RETURNING id")
        .bind(attempt_id).bind(reference).bind(&ticket_number).fetch_one(pool).await.unwrap();
    let departure_at: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT (instance.departure_date+service.departure_time) AT TIME ZONE service.origin_time_zone FROM flight_instances instance JOIN flight_services service ON service.id=instance.flight_service_id WHERE instance.id=$1")
        .bind(instance_id).fetch_one(pool).await.unwrap();
    BookingFixture {
        ticket_id,
        reference: reference.to_owned(),
        departure_at,
    }
}

#[tokio::test]
async fn booking_reads_use_effective_permissions_server_filters_exact_detail_and_historical_cabins()
{
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let business = booking_fixture(
        &pool,
        "BUSINESS0001",
        "XF22AAA222",
        "XF 980",
        "business",
        "SCHEDULED",
        50_000,
        ("Nara", "Service"),
        40,
    )
    .await;
    let economy = booking_fixture(
        &pool,
        "ECONOMY00001",
        "XF22BBB222",
        "XF 981",
        "economy",
        "SCHEDULED",
        20_000,
        ("Legacy", "Economy"),
        41,
    )
    .await;
    let premium = booking_fixture(
        &pool,
        "PREMIUM00001",
        "XF22CCC222",
        "XF 982",
        "premium-economy",
        "SCHEDULED",
        30_000,
        ("Legacy", "Premium"),
        42,
    )
    .await;
    let xf880 = booking_fixture(
        &pool,
        "XF8800000001",
        "XF22DDD222",
        "XF 880",
        "first",
        "CANCELLED",
        99_500,
        ("History", "Preserved"),
        43,
    )
    .await;
    let router = app(pool.clone());
    assert_eq!(
        send(&router, "GET", "/api/v1/admin/bookings", None, None, false)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    for (role, suffix) in [
        ("SYSTEM_ADMIN", "system"),
        ("EXECUTIVE", "executive"),
        ("FLIGHT_MANAGER", "flight"),
        ("TICKET_PASSENGER_OPERATIONS", "ticket"),
        ("BAGGAGE_STAFF", "baggage"),
    ] {
        let denied = cookie(&pool, role, suffix).await;
        assert_eq!(
            send(
                &router,
                "GET",
                "/api/v1/admin/bookings",
                Some(&denied),
                None,
                false
            )
            .await
            .status(),
            StatusCode::FORBIDDEN
        );
    }
    let operator = cookie(&pool, "BOOKING_OPERATIONS", "operator").await;
    let filtered = send(&router,"POST","/api/v1/admin/bookings/search",Some(&operator),Some(json!({"passengerName":" nara  service ","flightNumber":"xf980","travelDate":Utc::now().date_naive()+Duration::days(40),"bookingStatus":"CONFIRMED","cabin":"BUSINESS","origin":"syd","destination":"cdg","limit":1,"offset":0})),true).await;
    assert_eq!(filtered.status(), StatusCode::OK);
    let filtered = json_body(filtered).await;
    assert_eq!(filtered["items"].as_array().unwrap().len(), 1);
    assert_eq!(filtered["items"][0]["bookingReference"], business.reference);
    assert!(filtered["nextOffset"].is_null());
    let reference_search = json_body(
        send(
            &router,
            "POST",
            "/api/v1/admin/bookings/search",
            Some(&operator),
            Some(
                json!({"bookingReference":business.reference.to_lowercase(),"limit":50,"offset":0}),
            ),
            true,
        )
        .await,
    )
    .await;
    assert_eq!(reference_search["items"].as_array().unwrap().len(), 1);
    assert_eq!(
        reference_search["items"][0]["bookingReference"],
        business.reference
    );
    assert_eq!(
        send(
            &router,
            "POST",
            "/api/v1/admin/bookings/search",
            Some(&operator),
            Some(json!({"limit":51,"offset":0})),
            true
        )
        .await
        .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    for fixture in [&economy, &premium, &xf880] {
        let response = send(
            &router,
            "GET",
            &format!("/api/v1/admin/bookings/{}", fixture.reference),
            Some(&operator),
            None,
            false,
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let detail = json_body(response).await;
        assert_eq!(detail["bookingReference"], fixture.reference);
        assert_eq!(detail["payment"]["status"], "SUCCEEDED");
        assert_eq!(detail["ticket"]["status"], "ISSUED");
    }
    let xf880_detail = json_body(
        send(
            &router,
            "GET",
            &format!("/api/v1/admin/bookings/{}", xf880.reference),
            Some(&operator),
            None,
            false,
        )
        .await,
    )
    .await;
    assert_eq!(xf880_detail["journey"]["flightStatus"], "CANCELLED");
    assert_eq!(xf880_detail["bookingStatus"], "CONFIRMED");
    assert_eq!(xf880_detail["journey"]["cabin"], "first");
    assert_eq!(xf880_detail["payment"]["amount"]["amount"], 99_500);
    assert_eq!(
        send(
            &router,
            "GET",
            "/api/v1/admin/bookings/XF22ZZZ222",
            Some(&operator),
            None,
            false
        )
        .await
        .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(
            &router,
            "GET",
            &format!("/api/v1/admin/bookings/{}", business.reference),
            Some(&operator),
            None,
            false
        )
        .await
        .status(),
        StatusCode::OK
    );
    cleanup(&pool).await;
}

#[tokio::test]
async fn staff_cancellation_enforces_csrf_policy_idempotency_audit_and_history_preservation() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let fixture = booking_fixture(
        &pool,
        "CANCEL000001",
        "XF22EEE222",
        "XF 983",
        "business",
        "SCHEDULED",
        45_000,
        ("Safe", "Cancel"),
        40,
    )
    .await;
    let router = app(pool.clone());
    let operator = cookie(&pool, "BOOKING_OPERATIONS", "cancel-operator").await;
    let system = cookie(&pool, "SYSTEM_ADMIN", "cancel-system").await;
    let path = format!("/api/v1/admin/bookings/{}/cancel", fixture.reference);
    assert_eq!(
        send(&router, "POST", &path, Some(&system), None, true)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        send(&router, "POST", &path, Some(&operator), None, false)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    let first = send(&router, "POST", &path, Some(&operator), None, true).await;
    assert_eq!(first.status(), StatusCode::OK);
    let first = json_body(first).await;
    assert_eq!(first["bookingStatus"], "CANCELLED");
    assert_eq!(first["payment"]["status"], "SUCCEEDED");
    assert_eq!(first["ticket"]["status"], "CANCELLED");
    assert_eq!(first["cancellation"]["refundAmount"]["amount"], 45_000);
    assert_eq!(first["audit"].as_array().unwrap().len(), 1);
    assert_eq!(
        send(&router, "POST", &path, Some(&operator), None, true)
            .await
            .status(),
        StatusCode::OK
    );
    let state: (i64,i64,i64,String) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM booking_cancellations WHERE ticket_id=$1),(SELECT COUNT(*) FROM booking_operations_audit WHERE ticket_id=$1),(SELECT COUNT(*) FROM payment_attempt_seats ps JOIN tickets t ON t.payment_attempt_id=ps.payment_attempt_id WHERE t.id=$1 AND ps.released_at IS NOT NULL),(SELECT p.status FROM payment_attempts p JOIN tickets t ON t.payment_attempt_id=p.id WHERE t.id=$1)")
        .bind(fixture.ticket_id).fetch_one(&pool).await.unwrap();
    assert_eq!(state, (1, 1, 1, "SUCCEEDED".to_owned()));
    cleanup(&pool).await;
}

struct FixedClock(chrono::DateTime<Utc>);
impl Clock for FixedClock {
    fn now(&self) -> chrono::DateTime<Utc> {
        self.0
    }
}

#[tokio::test]
async fn cancellation_exact_boundary_and_staff_customer_race_create_one_refund() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let repository = Arc::new(SqlxSeatHoldRepository::new(pool.clone()));
    let exact = booking_fixture(
        &pool,
        "BOUNDARY0001",
        "XF22FFF222",
        "XF 984",
        "first",
        "SCHEDULED",
        60_000,
        ("Exact", "Boundary"),
        45,
    )
    .await;
    let actor = StaffCancellationActor {
        staff_user_id: Uuid::new_v4(),
        email: "synthetic.operator@booking-http.test".to_owned(),
    };
    repository
        .cancel_booking(
            exact.ticket_id,
            &FixedClock(exact.departure_at - Duration::hours(24)),
            Some(&actor),
        )
        .await
        .unwrap();
    let audit_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM booking_operations_audit WHERE ticket_id=$1")
            .bind(exact.ticket_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(audit_count, 1);

    let denied = booking_fixture(
        &pool,
        "DENIED000001",
        "XF22GGG222",
        "XF 985",
        "business",
        "SCHEDULED",
        40_000,
        ("Late", "Request"),
        46,
    )
    .await;
    assert!(repository
        .cancel_booking(
            denied.ticket_id,
            &FixedClock(denied.departure_at - Duration::hours(24) + Duration::seconds(1)),
            Some(&actor)
        )
        .await
        .is_err());

    let racing = booking_fixture(
        &pool,
        "RACING000001",
        "XF22HHH222",
        "XF 986",
        "business",
        "SCHEDULED",
        70_000,
        ("Race", "Safe"),
        47,
    )
    .await;
    let staff_repo = repository.clone();
    let customer_repo = repository.clone();
    let staff_actor = actor.clone();
    let now = racing.departure_at - Duration::hours(48);
    let ticket_id = racing.ticket_id;
    let staff = tokio::spawn(async move {
        staff_repo
            .cancel_booking(ticket_id, &FixedClock(now), Some(&staff_actor))
            .await
    });
    let customer = tokio::spawn(async move {
        customer_repo
            .cancel_booking(ticket_id, &FixedClock(now), None)
            .await
    });
    let (staff_result, customer_result) = tokio::join!(staff, customer);
    assert_eq!(
        staff_result.unwrap().unwrap().id,
        customer_result.unwrap().unwrap().id
    );
    let counts: (i64,i64,i64) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM booking_cancellations WHERE ticket_id=$1),(SELECT COUNT(*) FROM booking_operations_audit WHERE ticket_id=$1),(SELECT COUNT(*) FROM payment_attempt_seats ps JOIN tickets t ON t.payment_attempt_id=ps.payment_attempt_id WHERE t.id=$1 AND ps.released_at IS NOT NULL)")
        .bind(racing.ticket_id).fetch_one(&pool).await.unwrap();
    assert_eq!(counts.0, 1);
    assert!(counts.1 <= 1);
    assert_eq!(counts.2, 1);
    cleanup(&pool).await;
}
