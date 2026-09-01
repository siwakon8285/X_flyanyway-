use std::{env, sync::Arc, time::Duration};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
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

async fn app() -> axum::Router {
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
    let repository = Arc::new(SqlxSeatHoldRepository::new(pool));
    build_router(AppState::new(
        repository.clone(),
        repository.clone(),
        repository,
        Duration::from_secs(600),
        false,
        "http://localhost:3000".to_owned(),
    ))
}

fn create_request(seat: &str, date: chrono::NaiveDate) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/seat-holds")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "flightId": "xf-201",
                "departureDate": date,
                "cabin": "economy",
                "passengers": { "adults": 1, "children": 0, "infants": 0 },
                "seats": [seat]
            })
            .to_string(),
        ))
        .unwrap()
}

#[tokio::test]
async fn create_conflict_and_scoped_http_only_authorization_contract() {
    let app = app().await;
    let date = chrono::NaiveDate::from_ymd_opt(2100, 1, 1).unwrap()
        + chrono::Duration::days((uuid::Uuid::new_v4().as_u128() % 100_000) as i64);
    let first_response = app
        .clone()
        .oneshot(create_request("20F", date))
        .await
        .unwrap();
    assert_eq!(first_response.status(), StatusCode::CREATED);
    let cookie = first_response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Lax"));
    assert!(cookie.starts_with("x_fly_hold_"));

    let body: Value = serde_json::from_slice(
        &first_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes(),
    )
    .unwrap();
    assert!(body.get("accessToken").is_none());
    let hold_id = body["id"].as_str().unwrap();

    let conflict_response = app
        .clone()
        .oneshot(create_request("20F", date))
        .await
        .unwrap();
    assert_eq!(conflict_response.status(), StatusCode::CONFLICT);
    let conflict: Value = serde_json::from_slice(
        &conflict_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes(),
    )
    .unwrap();
    assert_eq!(conflict["error"]["code"], "SEAT_UNAVAILABLE");
    assert_eq!(conflict["error"]["conflictingSeats"], json!(["20F"]));

    let unauthorized = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{hold_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    let authorized = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{hold_id}"))
                .header(header::COOKIE, cookie.split(';').next().unwrap())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(authorized.status(), StatusCode::OK);
}

#[tokio::test]
async fn continue_validation_rejects_a_partial_hold() {
    let app = app().await;
    let date = chrono::NaiveDate::from_ymd_opt(2100, 1, 1).unwrap()
        + chrono::Duration::days((uuid::Uuid::new_v4().as_u128() % 100_000) as i64);
    let create = Request::builder()
        .method("POST")
        .uri("/api/v1/seat-holds")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "flightId": "xf-201",
                "departureDate": date,
                "cabin": "economy",
                "passengers": { "adults": 2, "children": 0, "infants": 0 },
                "seats": ["20E"]
            })
            .to_string(),
        ))
        .unwrap();
    let created = app.clone().oneshot(create).await.unwrap();
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
    let body: Value =
        serde_json::from_slice(&created.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let hold_id = body["id"].as_str().unwrap();

    let validation = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/seat-holds/{hold_id}/validation"))
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(validation.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let error: Value =
        serde_json::from_slice(&validation.into_body().collect().await.unwrap().to_bytes())
            .unwrap();
    assert_eq!(error["error"]["code"], "SEAT_COUNT_MISMATCH");
}
