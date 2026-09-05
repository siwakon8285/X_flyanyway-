use std::{
    env,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
    time::Duration,
};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::{Duration as ChronoDuration, NaiveDate, Utc};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

use x_fly_api::{
    infrastructure::{
        database::{prepare_database, SqlxSeatHoldRepository},
        http::build_router,
    },
    state::AppState,
};

async fn app() -> (axum::Router, PgPool) {
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
    let repository = Arc::new(SqlxSeatHoldRepository::new(pool.clone()));
    (
        build_router(AppState::new(
            repository.clone(),
            repository.clone(),
            repository.clone(),
            repository,
            Duration::from_secs(600),
            false,
            "http://localhost:3000".to_owned(),
        )),
        pool,
    )
}

fn departure_date() -> NaiveDate {
    static NEXT_OFFSET: AtomicI64 = AtomicI64::new(0);
    Utc::now().date_naive()
        + ChronoDuration::days(200 + NEXT_OFFSET.fetch_add(1, Ordering::Relaxed))
}

async fn create_hold(
    app: &axum::Router,
    departure: NaiveDate,
    passengers: Value,
    seats: Value,
) -> (String, String) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/seat-holds")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "flightId": "xf-201",
                        "departureDate": departure,
                        "cabin": "economy",
                        "passengers": passengers,
                        "seats": seats
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    (body["id"].as_str().unwrap().to_owned(), cookie)
}

fn passenger(ordinal: u8, passenger_type: &str, departure: NaiveDate) -> Value {
    let dob = match passenger_type {
        "ADULT" => NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
        "CHILD" => departure - ChronoDuration::days(8 * 365),
        "INFANT" => Utc::now().date_naive(),
        _ => NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
    };
    json!({
        "ordinal": ordinal,
        "passengerType": passenger_type,
        "title": "MS",
        "givenName": "  Nara  ",
        "middleName": null,
        "familyName": "Suri",
        "dateOfBirth": dob,
        "gender": "FEMALE",
        "nationalityCode": "th",
        "passportNumber": format!("TH-{ordinal}234567"),
        "passportIssuingCountryCode": "th",
        "email": format!("nara{ordinal}@example.com"),
        "phoneCountryCode": "+66",
        "phoneNumber": format!("81234567{ordinal}"),
        "emergencyContact": null
    })
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

#[tokio::test]
async fn saves_and_reloads_the_authorized_passenger_resource() {
    let (app, _) = app().await;
    let departure = departure_date();
    let (hold_id, cookie) = create_hold(
        &app,
        departure,
        json!({ "adults": 1, "children": 1, "infants": 1 }),
        json!(["20A", "20B"]),
    )
    .await;
    let passengers = vec![
        passenger(1, "ADULT", departure),
        passenger(2, "CHILD", departure),
        passenger(3, "INFANT", departure),
    ];
    let booking_contact = json!({
        "email": "booking-contact@example.com",
        "preferredLocale": "TH"
    });
    let saved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/seat-holds/{hold_id}/passengers"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({
                        "passengers": passengers,
                        "bookingContact": booking_contact
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(saved.status(), StatusCode::OK);
    assert_eq!(
        saved.headers().get(header::CACHE_CONTROL).unwrap(),
        "no-store, private"
    );
    let saved = json_body(saved).await;
    assert_eq!(saved["readyToContinue"], true);
    assert_eq!(saved["passengers"][0]["givenName"], "Nara");
    assert_eq!(saved["expectedPassengers"][2]["passengerType"], "INFANT");
    assert_eq!(
        saved["bookingContact"]["email"],
        "booking-contact@example.com"
    );
    assert_eq!(saved["bookingContact"]["preferredLocale"], "TH");

    let loaded = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{hold_id}/passengers"))
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(loaded.status(), StatusCode::OK);
    assert_eq!(
        loaded.headers().get(header::CACHE_CONTROL).unwrap(),
        "no-store, private"
    );
    let loaded = json_body(loaded).await;
    assert_eq!(loaded["passengers"].as_array().unwrap().len(), 3);
    assert_eq!(
        loaded["bookingContact"]["email"],
        "booking-contact@example.com"
    );
    assert_eq!(loaded["bookingContact"]["preferredLocale"], "TH");
}

#[tokio::test]
async fn rejects_count_and_type_tampering_with_safe_error_payloads() {
    let (app, _) = app().await;
    let departure = departure_date();
    let (hold_id, cookie) = create_hold(
        &app,
        departure,
        json!({ "adults": 1, "children": 1, "infants": 0 }),
        json!(["20A", "20B"]),
    )
    .await;

    let wrong_count = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/seat-holds/{hold_id}/passengers"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({ "passengers": [passenger(1, "ADULT", departure)] }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(wrong_count.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json_body(wrong_count).await["error"]["code"],
        "PASSENGER_COUNT_MISMATCH"
    );

    let private_email = "private-person@example.com";
    let private_passport = "SECRET999";
    let mut wrong_type = passenger(1, "CHILD", departure);
    wrong_type["email"] = private_email.into();
    wrong_type["passportNumber"] = private_passport.into();
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/seat-holds/{hold_id}/passengers"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, cookie)
                .body(Body::from(
                    json!({
                        "passengers": [wrong_type, passenger(2, "ADULT", departure)]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let text = String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(text.contains("PASSENGER_TYPE_MISMATCH"));
    assert!(!text.contains(private_email));
    assert!(!text.contains(private_passport));
}

#[tokio::test]
async fn expired_released_and_other_hold_cookies_cannot_read_passenger_data() {
    let (app, pool) = app().await;
    let departure = departure_date();
    let (first_id, first_cookie) = create_hold(
        &app,
        departure,
        json!({ "adults": 1, "children": 0, "infants": 0 }),
        json!(["20C"]),
    )
    .await;
    let (second_id, second_cookie) = create_hold(
        &app,
        departure + ChronoDuration::days(1),
        json!({ "adults": 1, "children": 0, "infants": 0 }),
        json!(["20C"]),
    )
    .await;

    let cross_access = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{second_id}/passengers"))
                .header(header::COOKIE, &first_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cross_access.status(), StatusCode::UNAUTHORIZED);

    sqlx::query("UPDATE seat_holds SET expires_at = NOW() WHERE id = $1")
        .bind(uuid::Uuid::parse_str(&first_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let expired = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{first_id}/passengers"))
                .header(header::COOKIE, first_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(expired.status(), StatusCode::GONE);
    assert_eq!(json_body(expired).await["error"]["code"], "HOLD_EXPIRED");

    let released = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/seat-holds/{second_id}"))
                .header(header::COOKIE, &second_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(released.status(), StatusCode::NO_CONTENT);
    let released_get = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{second_id}/passengers"))
                .header(header::COOKIE, second_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(released_get.status(), StatusCode::GONE);
    assert_eq!(
        json_body(released_get).await["error"]["code"],
        "HOLD_RELEASED"
    );
}
