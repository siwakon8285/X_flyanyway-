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
    application::use_cases::PaymentApplication,
    infrastructure::{
        database::{prepare_database, SqlxSeatHoldRepository},
        http::build_router,
        payment::{MockBitcoinPaymentGateway, MockCardPaymentGateway},
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
        build_router(
            AppState::new(
                repository.clone(),
                repository.clone(),
                repository.clone(),
                repository.clone(),
                Duration::from_secs(600),
                false,
                "http://localhost:3000".to_owned(),
            )
            .with_payments(PaymentApplication::new(
                repository,
                Arc::new(MockCardPaymentGateway),
                Arc::new(MockBitcoinPaymentGateway),
            )),
        ),
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

async fn complete_review(app: &axum::Router) -> (String, String) {
    let departure = departure_date();
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
    let cookie = created.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let hold_id = body(created).await["id"].as_str().unwrap().to_owned();
    let passenger = json!({
        "ordinal": 1,
        "passengerType": "ADULT",
        "title": "MS",
        "givenName": "Nara",
        "middleName": null,
        "familyName": "Payment",
        "dateOfBirth": "1990-01-01",
        "gender": "FEMALE",
        "nationalityCode": "TH",
        "passportNumber": format!("PH{:08X}", uuid::Uuid::new_v4().as_u128() as u32),
        "passportIssuingCountryCode": "TH",
        "email": "payment-private@example.com",
        "phoneCountryCode": "+66",
        "phoneNumber": "812345678",
        "emergencyContact": null
    });
    for (method, uri, payload) in [
        (
            "PUT",
            format!("/api/v1/seat-holds/{hold_id}/passengers"),
            json!({ "passengers": [passenger] }),
        ),
        (
            "PUT",
            format!("/api/v1/seat-holds/{hold_id}/extras"),
            json!({ "selections": [] }),
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, &cookie)
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    let review = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{hold_id}/review"))
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(review.status(), StatusCode::OK);
    (hold_id, cookie)
}

async fn create_attempt(
    app: &axum::Router,
    hold_id: &str,
    cookie: &str,
    request_id: uuid::Uuid,
    method: &str,
    scenario: Option<&str>,
) -> axum::response::Response {
    let mut payload = json!({ "requestId": request_id, "method": method });
    if let Some(scenario) = scenario {
        payload["scenario"] = json!(scenario);
    }
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/seat-holds/{hold_id}/payment-attempts"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, cookie)
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn loads_ready_payment_context_from_the_existing_review_snapshot() {
    let (app, _) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{hold_id}/payment"))
                .header(header::COOKIE, &cookie)
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
    assert_eq!(payload["pricing"]["grandTotal"]["amount"], 23_400);
    assert_eq!(payload["pricing"]["currencyCode"], "THB");
    assert_eq!(payload["methods"], json!(["CARD", "BITCOIN"]));
    assert_eq!(payload["readyForPayment"], true);
}

#[tokio::test]
async fn successful_mock_card_payment_finalizes_inventory_without_accepting_client_amounts() {
    let (app, pool) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;
    let rejected = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/seat-holds/{hold_id}/payment-attempts"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({
                        "requestId": uuid::Uuid::new_v4(),
                        "method": "CARD",
                        "scenario": "SUCCESS",
                        "amount": 1,
                        "currencyCode": "USD"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let request_id = uuid::Uuid::new_v4();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/seat-holds/{hold_id}/payment-attempts"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({
                        "requestId": request_id,
                        "method": "CARD",
                        "scenario": "SUCCESS"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let payload = body(response).await;
    assert_eq!(payload["status"], "SUCCEEDED");
    assert_eq!(payload["amount"]["amount"], 23_400);
    let finalized: (bool, String, Option<uuid::Uuid>) = sqlx::query_as(
        "SELECT hold.consumed_at IS NOT NULL, seat.booking_status, seat.hold_id
         FROM seat_holds AS hold
         JOIN flight_seats AS seat ON seat.flight_instance_id = hold.flight_instance_id AND seat.seat_number = '20A'
         WHERE hold.id = $1",
    )
    .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(finalized, (true, "BOOKED".to_owned(), None));

    let replay = create_attempt(&app, &hold_id, &cookie, request_id, "CARD", Some("SUCCESS")).await;
    assert_eq!(replay.status(), StatusCode::CREATED);
    assert_eq!(body(replay).await["id"], payload["id"]);

    let reloaded = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{hold_id}/payment"))
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reloaded.status(), StatusCode::OK);
    let reloaded = body(reloaded).await;
    assert_eq!(reloaded["readyForPayment"], false);
    assert_eq!(reloaded["attempts"][0]["status"], "SUCCEEDED");
    assert_eq!(reloaded["hold"]["seats"], json!(["20A"]));
}

#[tokio::test]
async fn persists_declined_and_processing_error_card_attempts_and_allows_retry() {
    let (app, pool) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;

    for (scenario, failure_code) in [
        ("DECLINED", "MOCK_CARD_DECLINED"),
        ("PROCESSING_ERROR", "MOCK_CARD_PROCESSING_ERROR"),
    ] {
        let response = create_attempt(
            &app,
            &hold_id,
            &cookie,
            uuid::Uuid::new_v4(),
            "CARD",
            Some(scenario),
        )
        .await;
        assert_eq!(response.status(), StatusCode::CREATED);
        let payload = body(response).await;
        assert_eq!(payload["status"], "FAILED");
        assert_eq!(payload["failure"]["code"], failure_code);
    }

    let hold: (Option<chrono::DateTime<chrono::Utc>>, i64) = sqlx::query_as(
        "SELECT consumed_at, (SELECT COUNT(*) FROM flight_seats WHERE hold_id = $1)
         FROM seat_holds WHERE id = $1",
    )
    .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(hold, (None, 1));
}

#[tokio::test]
async fn creates_and_simulates_a_demo_bitcoin_invoice() {
    let (app, pool) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;
    let created = create_attempt(
        &app,
        &hold_id,
        &cookie,
        uuid::Uuid::new_v4(),
        "BITCOIN",
        None,
    )
    .await;
    assert_eq!(created.status(), StatusCode::CREATED);
    let invoice = body(created).await;
    assert_eq!(invoice["status"], "AWAITING_PAYMENT");
    assert_eq!(invoice["demoBitcoinInvoice"]["rateThbPerBtc"], 2_000_000);
    assert!(invoice["demoBitcoinInvoice"]["demoAddress"]
        .as_str()
        .unwrap()
        .starts_with("DEMO-ONLY-NOT-A-BITCOIN-ADDRESS-"));
    let attempt_id = invoice["id"].as_str().unwrap();

    let received = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/seat-holds/{hold_id}/payment-attempts/{attempt_id}/simulate"
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, cookie)
                .body(Body::from(json!({ "outcome": "RECEIVED" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(received.status(), StatusCode::OK);
    assert_eq!(body(received).await["status"], "SUCCEEDED");
    let finalized: bool =
        sqlx::query_scalar("SELECT consumed_at IS NOT NULL FROM seat_holds WHERE id = $1")
            .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(finalized);
}

#[tokio::test]
async fn bitcoin_received_after_expiry_persists_hold_expired_failure() {
    let (app, pool) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;
    let created = create_attempt(
        &app,
        &hold_id,
        &cookie,
        uuid::Uuid::new_v4(),
        "BITCOIN",
        None,
    )
    .await;
    let attempt_id = body(created).await["id"].as_str().unwrap().to_owned();
    sqlx::query("UPDATE seat_holds SET expires_at = NOW() WHERE id = $1")
        .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    let received = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/seat-holds/{hold_id}/payment-attempts/{attempt_id}/simulate"
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, cookie)
                .body(Body::from(json!({ "outcome": "RECEIVED" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(received.status(), StatusCode::GONE);
    assert_eq!(body(received).await["error"]["code"], "HOLD_EXPIRED");
    let persisted: (String, Option<String>) =
        sqlx::query_as("SELECT status, failure_code FROM payment_attempts WHERE id = $1")
            .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        persisted,
        ("FAILED".to_owned(), Some("HOLD_EXPIRED".to_owned()))
    );
}

#[tokio::test]
async fn bitcoin_failure_and_cancellation_preserve_the_active_hold() {
    for outcome in ["FAILED", "CANCELLED"] {
        let (app, pool) = app().await;
        let (hold_id, cookie) = complete_review(&app).await;
        let created = create_attempt(
            &app,
            &hold_id,
            &cookie,
            uuid::Uuid::new_v4(),
            "BITCOIN",
            None,
        )
        .await;
        let attempt_id = body(created).await["id"].as_str().unwrap().to_owned();
        let simulated = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/seat-holds/{hold_id}/payment-attempts/{attempt_id}/simulate"
                    ))
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, cookie)
                    .body(Body::from(json!({ "outcome": outcome }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(simulated.status(), StatusCode::OK);
        assert_eq!(body(simulated).await["status"], outcome);
        let unchanged: (Option<chrono::DateTime<chrono::Utc>>, i64) = sqlx::query_as(
            "SELECT consumed_at, (SELECT COUNT(*) FROM flight_seats WHERE hold_id = $1)
             FROM seat_holds WHERE id = $1",
        )
        .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(unchanged, (None, 1));
    }
}

#[tokio::test]
async fn rejects_missing_review_unauthorized_and_cross_hold_attempt_access() {
    let (app, pool) = app().await;
    let (first_id, first_cookie) = complete_review(&app).await;
    let (second_id, second_cookie) = complete_review(&app).await;
    let unauthorized = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{first_id}/payment"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    sqlx::query("DELETE FROM hold_review_pricing WHERE seat_hold_id = $1")
        .bind(uuid::Uuid::parse_str(&second_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let missing_review = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{second_id}/payment"))
                .header(header::COOKIE, &second_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_review.status(), StatusCode::CONFLICT);
    assert_eq!(
        body(missing_review).await["error"]["code"],
        "REVIEW_NOT_READY"
    );
    let restored_review = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{second_id}/review"))
                .header(header::COOKIE, &second_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(restored_review.status(), StatusCode::OK);

    let bitcoin = create_attempt(
        &app,
        &first_id,
        &first_cookie,
        uuid::Uuid::new_v4(),
        "BITCOIN",
        None,
    )
    .await;
    let attempt_id = body(bitcoin).await["id"].as_str().unwrap().to_owned();
    let cross = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/seat-holds/{second_id}/payment-attempts/{attempt_id}/simulate"
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, second_cookie)
                .body(Body::from(json!({ "outcome": "RECEIVED" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cross.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn idempotent_declined_request_returns_one_persisted_attempt() {
    let (app, pool) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;
    let request_id = uuid::Uuid::new_v4();
    let first = create_attempt(
        &app,
        &hold_id,
        &cookie,
        request_id,
        "CARD",
        Some("DECLINED"),
    )
    .await;
    let first_id = body(first).await["id"].as_str().unwrap().to_owned();
    let replay = create_attempt(
        &app,
        &hold_id,
        &cookie,
        request_id,
        "CARD",
        Some("DECLINED"),
    )
    .await;
    assert_eq!(body(replay).await["id"], first_id);
    let mismatched =
        create_attempt(&app, &hold_id, &cookie, request_id, "CARD", Some("SUCCESS")).await;
    assert_eq!(mismatched.status(), StatusCode::CONFLICT);
    assert_eq!(
        body(mismatched).await["error"]["code"],
        "IDEMPOTENCY_KEY_REUSED"
    );
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM payment_attempts WHERE seat_hold_id = $1 AND request_id = $2",
    )
    .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
    .bind(request_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
}
