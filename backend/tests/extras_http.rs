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

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

async fn complete_hold(app: &axum::Router, departure: NaiveDate) -> (String, String) {
    let created = app
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
    assert_eq!(created.status(), StatusCode::CREATED);
    let cookie = created
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let hold_id = json_body(created).await["id"].as_str().unwrap().to_owned();

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
        "passportNumber": format!("EX{:08X}", uuid::Uuid::new_v4().as_u128() as u32),
        "passportIssuingCountryCode": "TH",
        "email": "extras@example.com",
        "phoneCountryCode": "+66",
        "phoneNumber": "812345678",
        "emergencyContact": null
    });
    let saved = app
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
    assert_eq!(saved.status(), StatusCode::OK);
    (hold_id, cookie)
}

#[tokio::test]
async fn loads_and_saves_the_authorized_extras_resource_with_server_prices() {
    let (app, _) = app().await;
    let (hold_id, cookie) = complete_hold(&app, departure_date()).await;

    let loaded = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{hold_id}/extras"))
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(loaded.status(), StatusCode::OK);
    assert_eq!(loaded.headers()[header::CACHE_CONTROL], "no-store, private");
    let loaded = json_body(loaded).await;
    assert_eq!(loaded["readyToContinue"], false);
    assert_eq!(loaded["catalog"]["allowances"]["checkedBaggageKg"], 20);
    assert_eq!(loaded["passengers"][0]["ordinal"], 1);

    let saved = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/seat-holds/{hold_id}/extras"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({
                        "selections": [{
                            "passengerOrdinal": 1,
                            "productCode": "BAG_30KG",
                            "quantity": 1
                        }]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(saved.status(), StatusCode::OK);
    let saved = json_body(saved).await;
    assert_eq!(saved["readyToContinue"], true);
    assert!(saved["savedAt"].is_string());
    assert_eq!(saved["total"]["amount"], 3_900);
    assert_eq!(saved["selections"][0]["unitPrice"]["amount"], 3_900);

    let restored = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{hold_id}/extras"))
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(json_body(restored).await["total"]["amount"], 3_900);
}

#[tokio::test]
async fn rejects_unknown_products_quantities_passengers_and_client_prices() {
    let (app, _) = app().await;
    let (hold_id, cookie) = complete_hold(&app, departure_date()).await;
    let cases = [
        (
            json!({ "passengerOrdinal": 1, "productCode": "UNKNOWN", "quantity": 1 }),
            "EXTRA_PRODUCT_UNKNOWN",
        ),
        (
            json!({ "passengerOrdinal": 1, "productCode": "BAG_10KG", "quantity": 2 }),
            "EXTRA_QUANTITY_INVALID",
        ),
        (
            json!({ "passengerOrdinal": 9, "productCode": "BAG_10KG", "quantity": 1 }),
            "EXTRA_PASSENGER_INVALID",
        ),
    ];
    for (selection, expected_code) in cases {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/seat-holds/{hold_id}/extras"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, &cookie)
                    .body(Body::from(json!({ "selections": [selection] }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json_body(response).await["error"]["code"], expected_code);
    }

    let client_price = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/seat-holds/{hold_id}/extras"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, cookie)
                .body(Body::from(
                    json!({
                        "selections": [{
                            "passengerOrdinal": 1,
                            "productCode": "BAG_30KG",
                            "quantity": 1,
                            "unitPrice": { "amount": 1, "currencyCode": "THB" }
                        }]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(client_price.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json_body(client_price).await["error"]["code"],
        "EXTRAS_VALIDATION_FAILED"
    );
}

#[tokio::test]
async fn another_hold_cookie_and_expired_hold_cannot_access_extras() {
    let (app, pool) = app().await;
    let (first_id, first_cookie) = complete_hold(&app, departure_date()).await;
    let (second_id, second_cookie) = complete_hold(&app, departure_date()).await;

    let cross_access = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{second_id}/extras"))
                .header(header::COOKIE, &first_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cross_access.status(), StatusCode::UNAUTHORIZED);

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
    let released_extras = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{second_id}/extras"))
                .header(header::COOKIE, second_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(released_extras.status(), StatusCode::GONE);
    assert_eq!(
        json_body(released_extras).await["error"]["code"],
        "HOLD_RELEASED"
    );

    sqlx::query("UPDATE seat_holds SET expires_at = NOW() WHERE id = $1")
        .bind(uuid::Uuid::parse_str(&first_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let expired = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{first_id}/extras"))
                .header(header::COOKIE, first_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(expired.status(), StatusCode::GONE);
    assert_eq!(json_body(expired).await["error"]["code"], "HOLD_EXPIRED");
}
