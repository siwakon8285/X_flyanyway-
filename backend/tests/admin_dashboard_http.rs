use std::{
    env,
    sync::{Arc, OnceLock},
    time::Duration,
};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::ServiceExt;
use uuid::Uuid;
use x_fly_api::{
    application::staff_auth::StaffAuthService,
    infrastructure::{
        database::{
            prepare_database, SqlxAnalyticsRepository, SqlxSeatHoldRepository,
            SqlxStaffAuthRepository,
        },
        http::build_router,
        password::Argon2PasswordService,
    },
    state::AppState,
};

const PASSWORD: &str = "change_me_for_local_development";

async fn fixture_guard() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn test_pool() -> PgPool {
    let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL is required");
    assert!(database_url
        .split('?')
        .next()
        .unwrap_or_default()
        .ends_with("_test"));
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();
    prepare_database(&pool).await.unwrap();
    clean_dashboard_fixtures(&pool).await;
    pool
}

async fn clean_dashboard_fixtures(pool: &PgPool) {
    sqlx::raw_sql(
        "DELETE FROM stripe_refund_events;
         DELETE FROM booking_cancellations;
         DELETE FROM tickets;
         DELETE FROM payment_attempt_seats;
         DELETE FROM stripe_webhook_events;
         DELETE FROM payment_attempts;
         DELETE FROM booking_confirmation_email_outbox;
         DELETE FROM booking_contacts;
         DELETE FROM hold_review_pricing;
         DELETE FROM hold_extras;
         DELETE FROM hold_passengers;
         DELETE FROM flight_seats;
         DELETE FROM seat_holds;
         DELETE FROM flight_instances;
         DELETE FROM staff_sessions;
         DELETE FROM staff_login_throttles;
         DELETE FROM staff_user_roles;
         DELETE FROM staff_users;
         DELETE FROM role_permissions WHERE role_code = 'SYSTEM_ADMIN' AND permission_code IN ('dashboard:read','analytics:read','reports:read');
         DELETE FROM role_permissions WHERE role_code LIKE 'DASH_TEST_%';
         DELETE FROM roles WHERE code LIKE 'DASH_TEST_%';
         DELETE FROM flight_services WHERE public_id LIKE 'dashboard-%';",
    )
    .execute(pool)
    .await
    .unwrap();
}

fn app(pool: PgPool) -> axum::Router {
    let booking = Arc::new(SqlxSeatHoldRepository::new(pool.clone()));
    let auth = StaffAuthService::new(
        Arc::new(SqlxStaffAuthRepository::new(pool.clone())),
        Argon2PasswordService::default(),
        Duration::from_secs(3600),
    )
    .unwrap();
    build_router(
        AppState::new(
            booking.clone(),
            booking.clone(),
            booking.clone(),
            booking,
            Duration::from_secs(600),
            true,
            "http://localhost:3000".to_owned(),
        )
        .with_staff_auth(auth)
        .with_analytics(Arc::new(SqlxAnalyticsRepository::new(pool))),
    )
}

async fn session_cookie(pool: &PgPool, suffix: &str, permissions: &[&str]) -> String {
    let role = if permissions.len() == 3 {
        "EXECUTIVE"
    } else {
        "SYSTEM_ADMIN"
    };
    let email = format!("{suffix}@dashboard.test").to_lowercase();
    let raw_token = Sha256::digest(format!("dashboard-token-{suffix}").as_bytes());
    let token_hash: [u8; 32] = Sha256::digest(raw_token).into();
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO staff_users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(email)
        .bind(PASSWORD)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO staff_user_roles (staff_user_id, role_code) VALUES ($1, $2)")
        .bind(user_id)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO staff_sessions (staff_user_id, token_hash, expires_at) VALUES ($1, $2, NOW() + INTERVAL '1 hour')")
        .bind(user_id).bind(token_hash.as_slice()).execute(pool).await.unwrap();
    format!("x_fly_staff_session={}", hex::encode(raw_token))
}

async fn get(router: &axum::Router, uri: &str, cookie: Option<&str>) -> axum::response::Response {
    let mut request = Request::builder().uri(uri);
    if let Some(cookie) = cookie {
        request = request.header(header::COOKIE, cookie);
    }
    router
        .clone()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn body(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn dashboard_authenticates_and_does_not_treat_system_admin_as_a_superuser() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let router = app(pool.clone());

    let unauthenticated = get(&router, "/api/v1/admin/dashboard?from=not-a-date", None).await;
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        unauthenticated.headers()[header::CACHE_CONTROL],
        "no-store, private"
    );

    let cookie = session_cookie(&pool, "system_admin", &[]).await;
    let response = get(
        &router,
        "/api/v1/admin/dashboard?from=not-a-date",
        Some(&cookie),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store, private"
    );
    assert_eq!(
        body(response).await["error"]["code"],
        "STAFF_PERMISSION_DENIED"
    );
    clean_dashboard_fixtures(&pool).await;
}

#[tokio::test]
async fn dashboard_strictly_validates_filters_and_returns_private_sanitized_errors() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let cookie = session_cookie(
        &pool,
        "executive_validation",
        &["dashboard:read", "analytics:read", "reports:read"],
    )
    .await;
    let router = app(pool.clone());
    for uri in [
        "/api/v1/admin/dashboard?from=2026-8-01&to=2026-08-03",
        "/api/v1/admin/dashboard?from=2026-08-04&to=2026-08-03",
        "/api/v1/admin/dashboard?from=2025-01-01&to=2026-08-03",
        "/api/v1/admin/dashboard?route=bkk-HND",
        "/api/v1/admin/dashboard?route=BKK-BKK",
        "/api/v1/admin/dashboard?cabin=coach",
        "/api/v1/admin/dashboard?provider=stripe",
        "/api/v1/admin/dashboard?provider=MOCK_CARD",
        "/api/v1/admin/dashboard?currency=USD",
        "/api/v1/admin/dashboard?extra=true",
    ] {
        let response = get(&router, uri, Some(&cookie)).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY, "{uri}");
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "no-store, private"
        );
        let payload = body(response).await;
        assert_eq!(
            payload,
            json!({"error":{"code":"DASHBOARD_FILTER_INVALID","message":"The dashboard filters are invalid."}})
        );
    }
    clean_dashboard_fixtures(&pool).await;
}

async fn insert_service(
    pool: &PgPool,
    flight_number: &str,
    origin: &str,
    destination: &str,
) -> Uuid {
    sqlx::query_scalar("INSERT INTO flight_services (public_id, flight_number, origin_code, destination_code, aircraft_code) VALUES ($1, $2, $3, $4, 'A320') RETURNING id")
        .bind(format!("dashboard-{flight_number}").to_lowercase()).bind(flight_number).bind(origin).bind(destination)
        .fetch_one(pool).await.unwrap()
}

#[allow(clippy::too_many_arguments)]
async fn insert_booking(
    pool: &PgPool,
    service_id: Uuid,
    departure: &str,
    cabin: &str,
    provider: &str,
    status: &str,
    amount: i64,
    currency: &str,
    succeeded_at: Option<&str>,
    ticket_status: Option<&str>,
    refund: Option<(&str, i64)>,
) {
    let instance_id: Uuid = sqlx::query_scalar("INSERT INTO flight_instances (flight_service_id, departure_date) VALUES ($1, $2::date) ON CONFLICT (flight_service_id, departure_date) DO UPDATE SET updated_at=flight_instances.updated_at RETURNING id")
        .bind(service_id).bind(departure).fetch_one(pool).await.unwrap();
    let hold_id: Uuid = sqlx::query_scalar("INSERT INTO seat_holds (flight_instance_id, cabin, adults, children, infants, access_token_hash, expires_at, consumed_at) VALUES ($1, $2, 1, 0, 0, $3, NOW() + INTERVAL '1 hour', NOW()) RETURNING id")
        .bind(instance_id).bind(cabin).bind([7_u8; 32].as_slice()).fetch_one(pool).await.unwrap();
    let attempt_id: Uuid = sqlx::query_scalar(
        "INSERT INTO payment_attempts (seat_hold_id, request_id, request_fingerprint, provider, payment_method, status, amount, currency_code, review_priced_at, provider_reference, failure_code, failure_message, succeeded_at, payment_finalization_deadline)
         VALUES ($1, $2, $3, $4, CASE WHEN $4='MOCK_BITCOIN' THEN 'BITCOIN' ELSE 'CARD' END, $5, $6, $7, NOW(), CASE WHEN $4='STRIPE' THEN $8 ELSE NULL END, CASE WHEN $5 IN ('FAILED','CANCELLED') THEN 'DECLINED' ELSE NULL END, CASE WHEN $5 IN ('FAILED','CANCELLED') THEN 'declined' ELSE NULL END, $9::timestamptz, CASE WHEN $4='STRIPE' THEN NOW() + INTERVAL '1 hour' ELSE NULL END) RETURNING id"
    ).bind(hold_id).bind(Uuid::new_v4()).bind([8_u8; 32].as_slice()).bind(provider).bind(status).bind(amount).bind(currency)
        .bind(format!("pi_{}", Uuid::new_v4())).bind(succeeded_at).fetch_one(pool).await.unwrap();
    if let Some(ticket_status) = ticket_status {
        let cancelled_at = (ticket_status == "CANCELLED").then_some("2026-08-20T00:00:00Z");
        let reference = Uuid::new_v4()
            .simple()
            .to_string()
            .to_uppercase()
            .replace('0', "A")
            .replace('1', "B");
        let number = Uuid::new_v4()
            .simple()
            .to_string()
            .to_uppercase()
            .replace('0', "A")
            .replace('1', "B");
        let ticket_id: Uuid = sqlx::query_scalar("INSERT INTO tickets (payment_attempt_id, booking_reference, ticket_number, status, cancelled_at) VALUES ($1, $2, $3, $4, $5::timestamptz) RETURNING id")
            .bind(attempt_id).bind(format!("XF{}", &reference[..8]))
            .bind(format!("XFT{}", &number[..12])).bind(ticket_status).bind(cancelled_at)
            .fetch_one(pool).await.unwrap();
        if let Some((refund_status, refund_amount)) = refund {
            let refunded_at = (refund_status == "SUCCEEDED").then_some("2026-08-21T00:00:00Z");
            sqlx::query("INSERT INTO booking_cancellations (ticket_id, payment_attempt_id, refund_provider, refund_status, refund_amount, currency_code, requested_at, cancelled_at, refunded_at) VALUES ($1, $2, $3, $4, $5, $6, '2026-08-20T00:00:00Z', '2026-08-20T00:00:00Z', $7::timestamptz)")
                .bind(ticket_id).bind(attempt_id).bind(provider).bind(refund_status).bind(refund_amount).bind(currency).bind(refunded_at)
                .execute(pool).await.unwrap();
        }
    }
}

async fn insert_inventory(
    pool: &PgPool,
    service_id: Uuid,
    departure: &str,
    cabin: &str,
    statuses: &[(&str, bool)],
) {
    let instance_id: Uuid = sqlx::query_scalar("INSERT INTO flight_instances (flight_service_id, departure_date) VALUES ($1, $2::date) ON CONFLICT (flight_service_id, departure_date) DO UPDATE SET updated_at=flight_instances.updated_at RETURNING id")
        .bind(service_id).bind(departure).fetch_one(pool).await.unwrap();
    for (index, (booking_status, sellable)) in statuses.iter().enumerate() {
        let seat = format!("{}A", index + 1);
        sqlx::query("INSERT INTO flight_seats (flight_instance_id, seat_number, row_number, column_code, cabin, position, sellable, booking_status, booked_at) VALUES ($1, $2, $3, 'A', $4, 'window', $5, $6, CASE WHEN $6='BOOKED' THEN NOW() ELSE NULL END)")
            .bind(instance_id).bind(seat).bind((index + 1) as i16).bind(cabin).bind(sellable).bind(*booking_status)
            .execute(pool).await.unwrap();
    }
}

#[tokio::test]
async fn dashboard_aggregates_literal_cohort_inventory_filters_and_exact_public_shape() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let bkk_hnd = insert_service(&pool, "DA901", "BKK", "HND").await;
    let hnd_bkk = insert_service(&pool, "DA902", "HND", "BKK").await;
    insert_booking(
        &pool,
        bkk_hnd,
        "2026-09-10",
        "economy",
        "STRIPE",
        "SUCCEEDED",
        1000,
        "THB",
        Some("2026-07-31T17:00:00Z"),
        Some("CANCELLED"),
        Some(("SUCCEEDED", 1000)),
    )
    .await;
    insert_booking(
        &pool,
        bkk_hnd,
        "2026-09-10",
        "economy",
        "STRIPE",
        "SUCCEEDED",
        2000,
        "THB",
        Some("2026-08-02T04:00:00Z"),
        Some("CANCELLED"),
        Some(("PENDING", 2000)),
    )
    .await;
    insert_booking(
        &pool,
        hnd_bkk,
        "2026-09-11",
        "business",
        "STRIPE",
        "SUCCEEDED",
        3000,
        "THB",
        Some("2026-08-03T02:00:00Z"),
        Some("CANCELLED"),
        Some(("REQUIRES_ATTENTION", 3000)),
    )
    .await;
    insert_booking(
        &pool,
        hnd_bkk,
        "2026-09-12",
        "business",
        "STRIPE",
        "SUCCEEDED",
        4000,
        "THB",
        Some("2026-08-03T16:59:59Z"),
        Some("ISSUED"),
        None,
    )
    .await;
    insert_booking(
        &pool,
        bkk_hnd,
        "2026-09-13",
        "economy",
        "STRIPE",
        "FAILED",
        9000,
        "THB",
        None,
        None,
        None,
    )
    .await;
    insert_booking(
        &pool,
        bkk_hnd,
        "2026-09-14",
        "economy",
        "MOCK_BITCOIN",
        "SUCCEEDED",
        8000,
        "THB",
        Some("2026-08-02T04:00:00Z"),
        Some("ISSUED"),
        None,
    )
    .await;
    insert_booking(
        &pool,
        bkk_hnd,
        "2026-09-15",
        "economy",
        "STRIPE",
        "SUCCEEDED",
        7000,
        "USD",
        Some("2026-08-02T04:00:00Z"),
        Some("ISSUED"),
        None,
    )
    .await;
    insert_booking(
        &pool,
        bkk_hnd,
        "2026-09-16",
        "economy",
        "STRIPE",
        "SUCCEEDED",
        6000,
        "THB",
        Some("2026-08-03T17:00:00Z"),
        Some("ISSUED"),
        None,
    )
    .await;
    insert_inventory(
        &pool,
        bkk_hnd,
        "2026-08-01",
        "economy",
        &[
            ("BOOKED", true),
            ("BOOKED", true),
            ("AVAILABLE", true),
            ("AVAILABLE", true),
            ("AVAILABLE", false),
        ],
    )
    .await;
    insert_inventory(
        &pool,
        bkk_hnd,
        "2026-08-02",
        "economy",
        &[("AVAILABLE", true), ("AVAILABLE", true)],
    )
    .await;
    insert_inventory(
        &pool,
        hnd_bkk,
        "2026-08-03",
        "business",
        &[("BOOKED", true)],
    )
    .await;
    let cookie = session_cookie(
        &pool,
        "executive_metrics",
        &["dashboard:read", "analytics:read", "reports:read"],
    )
    .await;
    let router = app(pool.clone());

    let response = get(
        &router,
        "/api/v1/admin/dashboard?from=2026-08-01&to=2026-08-03",
        Some(&cookie),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store, private"
    );
    let payload = body(response).await;
    assert_eq!(
        payload
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        [
            "availableRoutes",
            "cabins",
            "currency",
            "flights",
            "from",
            "generatedAt",
            "inventory",
            "provider",
            "revenueFlights",
            "routes",
            "summary",
            "timeZone",
            "to",
            "trends"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
    assert_eq!(payload["from"], "2026-08-01");
    assert_eq!(payload["to"], "2026-08-03");
    assert_eq!(payload["timeZone"], "Asia/Bangkok");
    assert_eq!(payload["currency"], "THB");
    assert_eq!(payload["provider"], "STRIPE");
    assert!(payload["generatedAt"].as_str().is_some());
    assert_eq!(
        payload["summary"],
        json!({
            "grossRevenue":10000,"totalBookings":4,"ticketsIssued":4,"cancelledBookings":3,
            "cancellationRatePercent":75.0,"refundCount":1,"refundValue":1000,
            "pendingRefundCount":1,"pendingRefundValue":2000,"attentionRefundCount":1,
            "averageBookingValue":2500.0
        })
    );
    assert_eq!(
        payload["trends"],
        json!([
            {"date":"2026-08-01","bookings":1,"revenue":1000},
            {"date":"2026-08-02","bookings":1,"revenue":2000},
            {"date":"2026-08-03","bookings":2,"revenue":7000}
        ])
    );
    assert_eq!(
        payload["routes"],
        json!([
            {"route":"HND-BKK","bookings":2,"revenue":7000},
            {"route":"BKK-HND","bookings":2,"revenue":3000}
        ])
    );
    assert_eq!(
        payload["cabins"],
        json!([
            {"cabin":"business","bookings":2,"revenue":7000},
            {"cabin":"economy","bookings":2,"revenue":3000}
        ])
    );
    assert_eq!(
        payload["flights"][0],
        json!({"flightNumber":"DA901","route":"BKK-HND","departureDate":"2026-09-10","bookings":2,"revenue":3000})
    );
    assert_eq!(
        payload["revenueFlights"][0],
        json!({"flightNumber":"DA902","route":"HND-BKK","departureDate":"2026-09-12","bookings":1,"revenue":4000})
    );
    assert_eq!(
        payload["inventory"],
        json!({
            "bookedSeats":3,"sellableSeats":7,"occupancyPercent":42.86,
            "flights":[
                {"flightNumber":"DA901","route":"BKK-HND","departureDate":"2026-08-02","bookedSeats":0,"sellableSeats":2,"occupancyPercent":0.0},
                {"flightNumber":"DA901","route":"BKK-HND","departureDate":"2026-08-01","bookedSeats":2,"sellableSeats":4,"occupancyPercent":50.0},
                {"flightNumber":"DA902","route":"HND-BKK","departureDate":"2026-08-03","bookedSeats":1,"sellableSeats":1,"occupancyPercent":100.0}
            ]
        })
    );
    assert!(payload["availableRoutes"]
        .as_array()
        .unwrap()
        .contains(&json!("BKK-HND")));
    assert!(payload["availableRoutes"]
        .as_array()
        .unwrap()
        .contains(&json!("HND-BKK")));
    let serialized = payload.to_string();
    for forbidden in [
        "passenger",
        "bookingReference",
        "ticketNumber",
        "providerReference",
        "secret",
    ] {
        assert!(!serialized.contains(forbidden));
    }

    let filtered = get(&router, "/api/v1/admin/dashboard?from=2026-08-01&to=2026-08-03&route=BKK-HND&cabin=economy&provider=STRIPE", Some(&cookie)).await;
    assert_eq!(filtered.status(), StatusCode::OK);
    let filtered = body(filtered).await;
    assert_eq!(filtered["summary"]["grossRevenue"], 3000);
    assert_eq!(filtered["summary"]["totalBookings"], 2);
    assert_eq!(
        filtered["trends"],
        json!([
            {"date":"2026-08-01","bookings":1,"revenue":1000},
            {"date":"2026-08-02","bookings":1,"revenue":2000},
            {"date":"2026-08-03","bookings":0,"revenue":0}
        ])
    );
    assert_eq!(filtered["inventory"]["bookedSeats"], 2);
    assert_eq!(filtered["inventory"]["sellableSeats"], 6);
    clean_dashboard_fixtures(&pool).await;
}

#[tokio::test]
async fn dashboard_empty_range_uses_nullable_rates_and_accepts_each_provider() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    insert_service(&pool, "DA903", "BKK", "LHR").await;
    let cookie = session_cookie(
        &pool,
        "executive_empty",
        &["dashboard:read", "analytics:read", "reports:read"],
    )
    .await;
    let router = app(pool.clone());
    for provider in ["STRIPE", "MOCK_BITCOIN"] {
        let response = get(
            &router,
            &format!("/api/v1/admin/dashboard?from=2026-08-01&to=2026-08-01&provider={provider}"),
            Some(&cookie),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let payload = body(response).await;
        assert_eq!(payload["provider"], provider);
        assert_eq!(payload["summary"]["cancellationRatePercent"], Value::Null);
        assert_eq!(payload["summary"]["averageBookingValue"], Value::Null);
        assert_eq!(payload["inventory"]["occupancyPercent"], Value::Null);
        assert!(payload["availableRoutes"]
            .as_array()
            .unwrap()
            .contains(&json!("BKK-LHR")));
    }
    clean_dashboard_fixtures(&pool).await;
}
