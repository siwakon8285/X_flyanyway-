mod common;

use std::{sync::Arc, time::Duration};

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
        database::{prepare_test_database, SqlxSeatHoldRepository},
        http::build_router,
    },
    state::AppState,
};

async fn app() -> (axum::Router, PgPool) {
    let database_url = common::test_database_url();
    let pool = PgPool::connect(&database_url).await.unwrap();
    prepare_test_database(&pool).await.unwrap();
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

#[tokio::test]
async fn removed_extras_endpoint_cannot_read_or_add_optional_products() {
    let (app, pool) = app().await;
    let departure = common::allocate_test_departure_date(
        "xf-201",
        chrono::NaiveDate::from_ymd_opt(2100, 1, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2104, 12, 31).unwrap(),
    )
    .await
    .unwrap();
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
                        "cabin": "business",
                        "passengers": { "adults": 1, "children": 0, "infants": 0 },
                        "seats": ["3A"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let cookie = created.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let payload: Value =
        serde_json::from_slice(&created.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let hold_id = payload["id"].as_str().unwrap();

    let get = app
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
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    let put = app
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
                            "quantity": 1
                        }]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(put.status(), StatusCode::NOT_FOUND);

    let extras: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM hold_extras WHERE seat_hold_id = $1")
            .bind(uuid::Uuid::parse_str(hold_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(extras, 0);
}
