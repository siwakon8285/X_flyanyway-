use std::{env, sync::Arc, time::Duration};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::{Duration as ChronoDuration, NaiveDate};
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
    let database_url = env::var("TEST_DATABASE_URL").unwrap();
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
    NaiveDate::from_ymd_opt(2100, 1, 1).unwrap()
        + ChronoDuration::days((uuid::Uuid::new_v4().as_u128() % 100_000) as i64)
}

async fn body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

async fn create_hold(app: &axum::Router) -> (String, String, NaiveDate) {
    let departure = departure_date();
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
                        "passengers": { "adults": 1, "children": 0, "infants": 0 },
                        "seats": ["20A"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let cookie = response.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let hold_id = body(response).await["id"].as_str().unwrap().to_owned();
    (hold_id, cookie, departure)
}

async fn complete_hold(app: &axum::Router) -> (String, String) {
    let (hold_id, cookie, departure) = create_hold(app).await;
    let passenger = json!({
        "ordinal": 1,
        "passengerType": "ADULT",
        "title": "MS",
        "givenName": "Nara",
        "middleName": null,
        "familyName": "Suri",
        "dateOfBirth": "1990-01-01",
        "gender": "FEMALE",
        "nationalityCode": "TH",
        "passportNumber": format!("RH{:08X}", uuid::Uuid::new_v4().as_u128() as u32),
        "passportIssuingCountryCode": "TH",
        "email": "review-private@example.com",
        "phoneCountryCode": "+66",
        "phoneNumber": "812345678",
        "emergencyContact": null
    });
    let saved_passenger = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/seat-holds/{hold_id}/passengers"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(json!({ "passengers": [passenger] }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(saved_passenger.status(), StatusCode::OK, "{departure}");
    let saved_extras = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/seat-holds/{hold_id}/extras"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({ "selections": [{
                        "passengerOrdinal": 1,
                        "productCode": "BAG_30KG",
                        "quantity": 1
                    }] })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(saved_extras.status(), StatusCode::OK);
    (hold_id, cookie)
}

#[tokio::test]
async fn returns_authorized_review_with_server_pricing_and_minimal_pii() {
    let (app, _) = app().await;
    let (hold_id, cookie) = complete_hold(&app).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/seat-holds/{hold_id}/review?grandTotal=1&currencyCode=USD"
                ))
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store, private"
    );
    let payload = body(response).await;
    assert_eq!(payload["readyForPayment"], true);
    assert_eq!(payload["pricing"]["currencyCode"], "THB");
    assert_eq!(payload["pricing"]["grandTotal"]["amount"], 27_300);
    assert_eq!(payload["journey"]["flightNumber"], "XF 201");
    assert_eq!(payload["passengers"][0]["displayName"], "MS Nara Suri");
    let serialized = payload.to_string();
    assert!(!serialized.contains("review-private@example.com"));
    assert!(!serialized.contains("812345678"));
    assert!(!serialized.contains("passportNumber"));
}

#[tokio::test]
async fn reports_the_exact_missing_review_prerequisite() {
    let (app, _) = app().await;
    let (hold_id, cookie, _) = create_hold(&app).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{hold_id}/review"))
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store, private"
    );
    assert_eq!(
        body(response).await["error"]["code"],
        "PASSENGERS_NOT_READY"
    );
}

#[tokio::test]
async fn rejects_missing_and_cross_hold_authorization() {
    let (app, _) = app().await;
    let (first_id, first_cookie) = complete_hold(&app).await;
    let (second_id, _) = complete_hold(&app).await;
    let missing = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{first_id}/review"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        missing.headers()[header::CACHE_CONTROL],
        "no-store, private"
    );

    let cross = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{second_id}/review"))
                .header(header::COOKIE, first_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cross.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rejects_expired_released_and_incomplete_seat_holds() {
    let (app, pool) = app().await;
    let (expired_id, expired_cookie) = complete_hold(&app).await;
    sqlx::query("UPDATE seat_holds SET expires_at = NOW() WHERE id = $1")
        .bind(uuid::Uuid::parse_str(&expired_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let expired = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{expired_id}/review"))
                .header(header::COOKIE, expired_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(expired.status(), StatusCode::GONE);
    assert_eq!(body(expired).await["error"]["code"], "HOLD_EXPIRED");

    let (released_id, released_cookie) = complete_hold(&app).await;
    let released = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/seat-holds/{released_id}"))
                .header(header::COOKIE, &released_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(released.status(), StatusCode::NO_CONTENT);
    let released_review = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{released_id}/review"))
                .header(header::COOKIE, released_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(released_review.status(), StatusCode::GONE);
    assert_eq!(
        body(released_review).await["error"]["code"],
        "HOLD_RELEASED"
    );

    let (incomplete_id, incomplete_cookie) = complete_hold(&app).await;
    sqlx::query("UPDATE flight_seats SET hold_id = NULL WHERE hold_id = $1")
        .bind(uuid::Uuid::parse_str(&incomplete_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let incomplete = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{incomplete_id}/review"))
                .header(header::COOKIE, incomplete_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(incomplete.status(), StatusCode::CONFLICT);
    assert_eq!(body(incomplete).await["error"]["code"], "SEATS_NOT_READY");
}
