mod common;

use std::{
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
    application::{flight::FlightManagement, staff_auth::StaffAuthService},
    infrastructure::{
        database::{
            prepare_database, SqlxFlightRepository, SqlxSeatHoldRepository, SqlxStaffAuthRepository,
        },
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
        .max_connections(5)
        .connect(&url)
        .await
        .unwrap();
    prepare_database(&pool).await.unwrap();
    sqlx::raw_sql(
        "DELETE FROM flight_management_audit WHERE flight_service_id IN (SELECT id FROM flight_services WHERE flight_number = 'XF 952')
             OR actor_staff_user_id IN (SELECT id FROM staff_users WHERE email LIKE '%@flight-http.test');
         DELETE FROM flight_service_seat_templates WHERE flight_service_id IN (SELECT id FROM flight_services WHERE flight_number = 'XF 952');
         DELETE FROM flight_instances WHERE flight_service_id IN (SELECT id FROM flight_services WHERE flight_number = 'XF 952');
         DELETE FROM flight_service_cabins WHERE flight_service_id IN (SELECT id FROM flight_services WHERE flight_number = 'XF 952');
         DELETE FROM flight_services WHERE flight_number = 'XF 952';
         DELETE FROM staff_sessions; DELETE FROM staff_user_roles; DELETE FROM staff_users WHERE email LIKE '%@flight-http.test';",
    ).execute(&pool).await.unwrap();
    pool
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
        .with_flights(FlightManagement::new(Arc::new(SqlxFlightRepository::new(
            pool,
        )))),
    )
}

async fn cookie(pool: &PgPool, role: &str, suffix: &str) -> String {
    let user: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash) VALUES ($1,'hash') RETURNING id",
    )
    .bind(format!("{suffix}@flight-http.test"))
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO staff_user_roles (staff_user_id,role_code) VALUES ($1,$2)")
        .bind(user)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
    let raw = Sha256::digest(format!("flight-http-{suffix}").as_bytes());
    let hash: [u8; 32] = Sha256::digest(raw).into();
    sqlx::query("INSERT INTO staff_sessions (staff_user_id,token_hash,expires_at) VALUES ($1,$2,NOW()+INTERVAL '1 hour')")
        .bind(user).bind(hash.as_slice()).execute(pool).await.unwrap();
    format!("x_fly_staff_session={}", hex::encode(raw))
}

async fn send(
    router: &axum::Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    payload: Option<Value>,
    trusted: bool,
) -> axum::response::Response {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    if payload.is_some() {
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
                .body(payload.map_or_else(Body::empty, |value| Body::from(value.to_string())))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn body(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    }
}

fn create_payload(number: &str) -> Value {
    json!({"flightNumber":number,"originCode":"BKK","destinationCode":"DXB","operatingDate":"2026-10-09",
        "departureTime":"09:20:00","arrivalTime":"13:05:00","arrivalDayOffset":0,"aircraftCode":"Boeing 787-9",
        "businessPriceAmount":46900,"firstPriceAmount":78900,"currencyCode":"THB","businessCapacity":16,"firstCapacity":4})
}

#[tokio::test]
async fn flights_use_effective_read_and_write_permissions_without_role_bypasses() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let router = app(pool.clone());
    assert_eq!(
        send(&router, "GET", "/api/v1/admin/flights", None, None, false)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );

    let baggage = cookie(&pool, "BAGGAGE_STAFF", "baggage").await;
    assert_eq!(
        send(
            &router,
            "GET",
            "/api/v1/admin/flights",
            Some(&baggage),
            None,
            false
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        send(
            &router,
            "POST",
            "/api/v1/admin/flights",
            Some(&baggage),
            Some(create_payload("XF 952")),
            true
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );

    for (role, suffix) in [("SYSTEM_ADMIN", "system"), ("EXECUTIVE", "executive")] {
        let denied = cookie(&pool, role, suffix).await;
        assert_eq!(
            send(
                &router,
                "POST",
                "/api/v1/admin/flights",
                Some(&denied),
                Some(create_payload("XF 952")),
                true
            )
            .await
            .status(),
            StatusCode::FORBIDDEN
        );
    }

    let manager = cookie(&pool, "FLIGHT_MANAGER", "manager").await;
    assert_eq!(
        send(
            &router,
            "POST",
            "/api/v1/admin/flights",
            Some(&manager),
            Some(create_payload("XF 952")),
            false
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let created_response = send(
        &router,
        "POST",
        "/api/v1/admin/flights",
        Some(&manager),
        Some(create_payload("XF 952")),
        true,
    )
    .await;
    assert_eq!(created_response.status(), StatusCode::CREATED);
    let created = body(created_response).await;
    let id = created["id"].as_str().unwrap();
    assert_eq!(created["business"]["priceAmount"], 46900);
    let detail_path = format!("/api/v1/admin/flights/{id}");
    let cancel_path = format!("/api/v1/admin/flights/{id}/cancel");
    let mut update = create_payload("XF 952");
    update["version"] = json!(1);

    for (method, uri, payload) in [
        (
            "POST",
            "/api/v1/admin/flights".to_owned(),
            Some(create_payload("XF 953")),
        ),
        ("GET", detail_path.clone(), None),
        ("PUT", detail_path.clone(), Some(update.clone())),
        ("POST", cancel_path.clone(), Some(json!({"version":1}))),
    ] {
        assert_eq!(
            send(&router, method, &uri, None, payload, true)
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }

    assert_eq!(
        send(&router, "GET", &detail_path, Some(&baggage), None, false)
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        send(
            &router,
            "PUT",
            &detail_path,
            Some(&baggage),
            Some(update.clone()),
            true
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        send(
            &router,
            "POST",
            &cancel_path,
            Some(&baggage),
            Some(json!({"version":1})),
            true
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let updated = send(
        &router,
        "PUT",
        &detail_path,
        Some(&manager),
        Some(update),
        true,
    )
    .await;
    assert_eq!(updated.status(), StatusCode::OK);
    assert_eq!(body(updated).await["version"], 2);
    let cancelled = send(
        &router,
        "POST",
        &cancel_path,
        Some(&manager),
        Some(json!({"version":2})),
        true,
    )
    .await;
    assert_eq!(cancelled.status(), StatusCode::OK);
    assert_eq!(body(cancelled).await["status"], "CANCELLED");
}

#[tokio::test]
async fn flight_filters_and_validation_are_bounded_and_structured() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let manager = cookie(&pool, "FLIGHT_MANAGER", "validation").await;
    let router = app(pool);
    for uri in [
        "/api/v1/admin/flights?limit=1000",
        "/api/v1/admin/flights?status=DELAYED",
        "/api/v1/admin/flights?date=not-a-date",
    ] {
        let response = send(&router, "GET", uri, Some(&manager), None, false).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY, "{uri}");
        assert_eq!(
            body(response).await["error"]["code"],
            "FLIGHT_VALIDATION_FAILED"
        );
    }
    let invalid = send(
        &router,
        "POST",
        "/api/v1/admin/flights",
        Some(&manager),
        Some(create_payload("BAD")),
        true,
    )
    .await;
    assert_eq!(invalid.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        body(invalid).await["error"]["code"],
        "FLIGHT_VALIDATION_FAILED"
    );
}
