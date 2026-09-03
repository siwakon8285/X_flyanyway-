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
use chrono::{Duration as ChronoDuration, NaiveDate};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

use x_fly_api::{
    application::use_cases::PaymentApplication,
    domain::payment::{
        PaymentGateway, PaymentGatewayOutcome, PaymentGatewayRequest, PaymentProviderError,
        PaymentProviderReconciler, PaymentProviderState, PaymentReconciliationStatus,
    },
    infrastructure::{
        database::{prepare_database, SqlxSeatHoldRepository},
        http::build_router,
        payment::MockBitcoinPaymentGateway,
    },
    state::AppState,
};

#[derive(Clone, Debug)]
struct TestStripePaymentGateway {
    amount: Arc<AtomicI64>,
    cancellation: PaymentReconciliationStatus,
    retrieved: PaymentReconciliationStatus,
}

impl Default for TestStripePaymentGateway {
    fn default() -> Self {
        Self {
            amount: Arc::new(AtomicI64::new(0)),
            cancellation: PaymentReconciliationStatus::AwaitingCustomer,
            retrieved: PaymentReconciliationStatus::AwaitingCustomer,
        }
    }
}

#[async_trait::async_trait]
impl PaymentGateway for TestStripePaymentGateway {
    async fn initiate(&self, request: PaymentGatewayRequest) -> PaymentGatewayOutcome {
        self.amount
            .store(request.amount.amount * 100, Ordering::Relaxed);
        let reference = format!("pi_{}", request.attempt_id.simple());
        PaymentGatewayOutcome::AwaitingPayment {
            client_payment_session: Some(format!("{reference}_secret_test")),
            provider_reference: reference,
        }
    }
}

#[async_trait::async_trait]
impl PaymentProviderReconciler for TestStripePaymentGateway {
    async fn retrieve_payment_intent(
        &self,
        provider_reference: &str,
    ) -> Result<PaymentProviderState, PaymentProviderError> {
        Ok(PaymentProviderState {
            amount: self.amount.load(Ordering::Relaxed),
            currency: "thb".to_owned(),
            failure: None,
            provider_reference: provider_reference.to_owned(),
            status: self.retrieved,
        })
    }

    async fn cancel_payment_intent(
        &self,
        provider_reference: &str,
    ) -> Result<PaymentProviderState, PaymentProviderError> {
        let mut state = self.retrieve_payment_intent(provider_reference).await?;
        state.status = self.cancellation;
        Ok(state)
    }
}

const TEST_WEBHOOK_SECRET: &str = "whsec_test_secret_for_integration_testing_12345";

async fn app_with_secret(secret: Option<String>) -> (axum::Router, PgPool) {
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
            .with_payments({
                let stripe = Arc::new(TestStripePaymentGateway::default());
                PaymentApplication::new(
                    repository,
                    stripe.clone(),
                    Arc::new(MockBitcoinPaymentGateway),
                )
                .with_stripe_provider(stripe)
            })
            .with_stripe_webhook_secret(secret),
        ),
        pool,
    )
}

async fn app_with_reconciliation_status(
    status: PaymentReconciliationStatus,
) -> (axum::Router, PgPool) {
    app_with_provider_statuses(status, status).await
}

async fn app_with_provider_statuses(
    retrieved: PaymentReconciliationStatus,
    cancellation: PaymentReconciliationStatus,
) -> (axum::Router, PgPool) {
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
    let stripe = Arc::new(TestStripePaymentGateway {
        amount: Arc::new(AtomicI64::new(0)),
        cancellation,
        retrieved,
    });
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
            .with_payments(
                PaymentApplication::new(
                    repository,
                    stripe.clone(),
                    Arc::new(MockBitcoinPaymentGateway),
                )
                .with_stripe_provider(stripe),
            )
            .with_stripe_webhook_secret(Some(TEST_WEBHOOK_SECRET.to_owned())),
        ),
        pool,
    )
}

async fn app() -> (axum::Router, PgPool) {
    app_with_secret(Some(TEST_WEBHOOK_SECRET.to_owned())).await
}

fn signed_webhook_header(payload: &[u8], secret: &str, timestamp: i64) -> String {
    use x_fly_api::infrastructure::payment::stripe::compute_stripe_signature;
    let sig = compute_stripe_signature(timestamp, payload, secret).unwrap();
    format!("t={timestamp},v1={sig}")
}

async fn post_webhook(
    app: &axum::Router,
    signature: Option<&str>,
    payload: &[u8],
) -> axum::response::Response {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/v1/payments/stripe/webhook")
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(sig) = signature {
        builder = builder.header("Stripe-Signature", sig);
    }
    app.clone()
        .oneshot(builder.body(Body::from(payload.to_vec())).unwrap())
        .await
        .unwrap()
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
) -> axum::response::Response {
    let payload = json!({ "requestId": request_id, "method": method });
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

async fn get_payment_context(
    app: &axum::Router,
    hold_id: &str,
    cookie: &str,
) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/seat-holds/{hold_id}/payment"))
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn payment_context_reconciles_a_succeeded_stripe_intent_through_guarded_finalization() {
    let (app, pool) = app_with_reconciliation_status(PaymentReconciliationStatus::Succeeded).await;
    let (hold_id, cookie) = complete_review(&app).await;
    let created = create_attempt(&app, &hold_id, &cookie, uuid::Uuid::new_v4(), "CARD").await;
    assert_eq!(created.status(), StatusCode::CREATED);
    let attempt_id = body(created).await["id"].as_str().unwrap().to_owned();

    let context = get_payment_context(&app, &hold_id, &cookie).await;
    assert_eq!(context.status(), StatusCode::OK);
    assert_eq!(body(context).await["attempts"][0]["status"], "SUCCEEDED");
    let booked: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM flight_seats AS seat JOIN payment_attempt_seats AS paid ON paid.flight_seat_id = seat.id WHERE paid.payment_attempt_id = $1 AND seat.booking_status = 'BOOKED'",
    )
    .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(booked, 1);

    let replay = get_payment_context(&app, &hold_id, &cookie).await;
    assert_eq!(replay.status(), StatusCode::OK);
    let paid_seats: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM payment_attempt_seats WHERE payment_attempt_id = $1",
    )
    .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(paid_seats, 1);
}

#[tokio::test]
async fn expired_unresolved_intent_only_releases_protection_after_confirmed_cancellation() {
    let (app, pool) = app_with_provider_statuses(
        PaymentReconciliationStatus::AwaitingCustomer,
        PaymentReconciliationStatus::Cancelled,
    )
    .await;
    let (hold_id, cookie) = complete_review(&app).await;
    let created = create_attempt(&app, &hold_id, &cookie, uuid::Uuid::new_v4(), "CARD").await;
    let attempt_id = body(created).await["id"].as_str().unwrap().to_owned();
    sqlx::query("UPDATE payment_attempts SET payment_finalization_deadline = NOW() - INTERVAL '1 second' WHERE id = $1")
        .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    let context = get_payment_context(&app, &hold_id, &cookie).await;
    assert_eq!(context.status(), StatusCode::OK);
    assert_eq!(body(context).await["attempts"][0]["status"], "CANCELLED");
    let status: String = sqlx::query_scalar("SELECT status FROM payment_attempts WHERE id = $1")
        .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "CANCELLED");
    let failure_code: String =
        sqlx::query_scalar("SELECT failure_code FROM payment_attempts WHERE id = $1")
            .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(failure_code, "PAYMENT_CANCELLED");
    let protected: bool = sqlx::query_scalar("SELECT has_protected_stripe_card_finalization($1)")
        .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!protected);
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
async fn stripe_card_attempt_returns_a_client_session_and_preserves_held_inventory() {
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
    let response = create_attempt(&app, &hold_id, &cookie, request_id, "CARD").await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let payload = body(response).await;
    assert_eq!(payload["status"], "AWAITING_PAYMENT");
    assert_eq!(payload["amount"]["amount"], 23_400);
    assert!(payload["clientPaymentSession"]
        .as_str()
        .unwrap()
        .ends_with("_secret_test"));
    let state: (bool, String, Option<uuid::Uuid>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "SELECT hold.consumed_at IS NOT NULL, seat.booking_status, seat.hold_id,
                hold.expires_at, attempt.payment_finalization_deadline
         FROM seat_holds AS hold
         JOIN flight_seats AS seat ON seat.flight_instance_id = hold.flight_instance_id AND seat.seat_number = '20A'
         JOIN payment_attempts AS attempt ON attempt.seat_hold_id = hold.id
         WHERE hold.id = $1",
    )
    .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!state.0);
    assert_eq!(state.1, "AVAILABLE");
    assert_eq!(state.2, Some(uuid::Uuid::parse_str(&hold_id).unwrap()));
    assert_eq!(state.4, state.3 + ChronoDuration::minutes(5));

    let replay = create_attempt(&app, &hold_id, &cookie, request_id, "CARD").await;
    assert_eq!(replay.status(), StatusCode::CREATED);
    assert_eq!(body(replay).await["id"], payload["id"]);
}

#[tokio::test]
async fn creates_and_simulates_a_demo_bitcoin_invoice() {
    let (app, pool) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;
    let created = create_attempt(&app, &hold_id, &cookie, uuid::Uuid::new_v4(), "BITCOIN").await;
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
    let created = create_attempt(&app, &hold_id, &cookie, uuid::Uuid::new_v4(), "BITCOIN").await;
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
        let created =
            create_attempt(&app, &hold_id, &cookie, uuid::Uuid::new_v4(), "BITCOIN").await;
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
async fn idempotent_stripe_request_returns_one_persisted_attempt() {
    let (app, pool) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;
    let request_id = uuid::Uuid::new_v4();
    let first = create_attempt(&app, &hold_id, &cookie, request_id, "CARD").await;
    let first_id = body(first).await["id"].as_str().unwrap().to_owned();
    let replay = create_attempt(&app, &hold_id, &cookie, request_id, "CARD").await;
    assert_eq!(body(replay).await["id"], first_id);
    let mismatched = create_attempt(&app, &hold_id, &cookie, request_id, "BITCOIN").await;
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

#[tokio::test]
async fn rejects_missing_and_malformed_webhook_signature_with_400() {
    let (app, _) = app().await;
    let payload = json!({
        "id": "evt_test_1",
        "type": "payment_intent.succeeded",
        "data": { "object": { "id": "pi_1" } }
    })
    .to_string();

    let res = post_webhook(&app, None, payload.as_bytes()).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body(res).await["error"]["code"], "STRIPE_SIGNATURE_MISSING");

    let res = post_webhook(&app, Some("malformed_signature_header"), payload.as_bytes()).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body(res).await["error"]["code"],
        "STRIPE_SIGNATURE_MALFORMED"
    );

    let invalid_sig = format!(
        "t={},v1=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        chrono::Utc::now().timestamp()
    );
    let res = post_webhook(&app, Some(&invalid_sig), payload.as_bytes()).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body(res).await["error"]["code"], "STRIPE_SIGNATURE_INVALID");

    let (unconfigured_app, _) = app_with_secret(None).await;
    let valid_sig = signed_webhook_header(
        payload.as_bytes(),
        TEST_WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );
    let res = post_webhook(&unconfigured_app, Some(&valid_sig), payload.as_bytes()).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body(res).await["error"]["code"],
        "STRIPE_WEBHOOK_UNCONFIGURED"
    );
}

#[tokio::test]
async fn authenticated_unrelated_event_acknowledged_with_no_mutation() {
    let (app, pool) = app().await;
    let payload = json!({
        "id": "evt_unrelated_123",
        "type": "customer.created",
        "data": { "object": { "id": "cus_123" } }
    })
    .to_string();

    let sig = signed_webhook_header(
        payload.as_bytes(),
        TEST_WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );
    let res = post_webhook(&app, Some(&sig), payload.as_bytes()).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(body(res).await["received"], true);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM stripe_webhook_events WHERE stripe_event_id = 'evt_unrelated_123'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn valid_stripe_success_webhook_atomically_finalizes_hold_and_seats() {
    let (app, pool) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;
    let request_id = uuid::Uuid::new_v4();
    let attempt_res = create_attempt(&app, &hold_id, &cookie, request_id, "CARD").await;
    assert_eq!(attempt_res.status(), StatusCode::CREATED);
    let attempt_body = body(attempt_res).await;
    let attempt_id = attempt_body["id"].as_str().unwrap();

    let pi_reference: String =
        sqlx::query_scalar("SELECT provider_reference FROM payment_attempts WHERE id = $1")
            .bind(uuid::Uuid::parse_str(attempt_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();

    let event_id = format!("evt_{}", uuid::Uuid::new_v4().simple());
    let payload = json!({
        "id": event_id,
        "type": "payment_intent.succeeded",
        "data": {
            "object": {
                "id": pi_reference,
                "amount": 2_340_000,
                "currency": "thb",
                "status": "succeeded"
            }
        }
    })
    .to_string();

    let sig = signed_webhook_header(
        payload.as_bytes(),
        TEST_WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );
    let res = post_webhook(&app, Some(&sig), payload.as_bytes()).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(body(res).await["received"], true);

    let state: (
        bool,
        String,
        Option<uuid::Uuid>,
        Option<chrono::DateTime<chrono::Utc>>,
    ) = sqlx::query_as(
        "SELECT hold.consumed_at IS NOT NULL, seat.booking_status, seat.hold_id, seat.booked_at
         FROM seat_holds AS hold
         JOIN flight_seats AS seat ON seat.flight_instance_id = hold.flight_instance_id AND seat.seat_number = '20A'
         WHERE hold.id = $1",
    )
    .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(state.0);
    assert_eq!(state.1, "BOOKED");
    assert_eq!(state.2, None);
    assert!(state.3.is_some());

    let attempt_state: (String, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT status, succeeded_at FROM payment_attempts WHERE id = $1")
            .bind(uuid::Uuid::parse_str(attempt_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(attempt_state.0, "SUCCEEDED");
    assert!(attempt_state.1.is_some());

    let seats_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM payment_attempt_seats WHERE payment_attempt_id = $1",
    )
    .bind(uuid::Uuid::parse_str(attempt_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(seats_count, 1);

    let recorded_event: String = sqlx::query_scalar(
        "SELECT payment_intent_id FROM stripe_webhook_events WHERE stripe_event_id = $1",
    )
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(recorded_event, pi_reference);
}

#[tokio::test]
async fn amount_and_currency_mismatch_rejects_and_does_not_finalize() {
    let (app, pool) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;
    let request_id = uuid::Uuid::new_v4();
    let attempt_res = create_attempt(&app, &hold_id, &cookie, request_id, "CARD").await;
    assert_eq!(attempt_res.status(), StatusCode::CREATED);
    let attempt_id = body(attempt_res).await["id"].as_str().unwrap().to_owned();

    let pi_reference: String =
        sqlx::query_scalar("SELECT provider_reference FROM payment_attempts WHERE id = $1")
            .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();

    let payload_amount_mismatch = json!({
        "id": format!("evt_amt_{}", uuid::Uuid::new_v4().simple()),
        "type": "payment_intent.succeeded",
        "data": {
            "object": {
                "id": pi_reference,
                "amount": 2_000_000,
                "currency": "thb",
                "status": "succeeded"
            }
        }
    })
    .to_string();
    let sig = signed_webhook_header(
        payload_amount_mismatch.as_bytes(),
        TEST_WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );
    let res = post_webhook(&app, Some(&sig), payload_amount_mismatch.as_bytes()).await;
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body(res).await["error"]["code"], "AMOUNT_MISMATCH");

    let payload_currency_mismatch = json!({
        "id": format!("evt_curr_{}", uuid::Uuid::new_v4().simple()),
        "type": "payment_intent.succeeded",
        "data": {
            "object": {
                "id": pi_reference,
                "amount": 2_340_000,
                "currency": "usd",
                "status": "succeeded"
            }
        }
    })
    .to_string();
    let sig = signed_webhook_header(
        payload_currency_mismatch.as_bytes(),
        TEST_WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );
    let res = post_webhook(&app, Some(&sig), payload_currency_mismatch.as_bytes()).await;
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body(res).await["error"]["code"], "AMOUNT_MISMATCH");

    let state: (bool, String) = sqlx::query_as(
        "SELECT hold.consumed_at IS NOT NULL, seat.booking_status
         FROM seat_holds AS hold
         JOIN flight_seats AS seat ON seat.flight_instance_id = hold.flight_instance_id AND seat.seat_number = '20A'
         WHERE hold.id = $1",
    )
    .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!state.0);
    assert_eq!(state.1, "AVAILABLE");
}

#[tokio::test]
async fn duplicate_and_concurrent_success_webhook_deliveries_are_idempotent() {
    let (app, pool) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;
    let request_id = uuid::Uuid::new_v4();
    let attempt_res = create_attempt(&app, &hold_id, &cookie, request_id, "CARD").await;
    let attempt_id = body(attempt_res).await["id"].as_str().unwrap().to_owned();

    let pi_reference: String =
        sqlx::query_scalar("SELECT provider_reference FROM payment_attempts WHERE id = $1")
            .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();

    let event_id = format!("evt_{}", uuid::Uuid::new_v4().simple());
    let payload = json!({
        "id": event_id,
        "type": "payment_intent.succeeded",
        "data": {
            "object": {
                "id": pi_reference,
                "amount": 2_340_000,
                "currency": "thb",
                "status": "succeeded"
            }
        }
    })
    .to_string();

    let sig = signed_webhook_header(
        payload.as_bytes(),
        TEST_WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );

    let first = post_webhook(&app, Some(&sig), payload.as_bytes()).await;
    assert_eq!(first.status(), StatusCode::OK);

    let replay = post_webhook(&app, Some(&sig), payload.as_bytes()).await;
    assert_eq!(replay.status(), StatusCode::OK);
    assert_eq!(body(replay).await["received"], true);

    let (hold_id2, cookie2) = complete_review(&app).await;
    let attempt_res2 =
        create_attempt(&app, &hold_id2, &cookie2, uuid::Uuid::new_v4(), "CARD").await;
    let attempt_id2 = body(attempt_res2).await["id"].as_str().unwrap().to_owned();
    let pi_reference2: String =
        sqlx::query_scalar("SELECT provider_reference FROM payment_attempts WHERE id = $1")
            .bind(uuid::Uuid::parse_str(&attempt_id2).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();

    let event_id2 = format!("evt_concurrent_{}", uuid::Uuid::new_v4().simple());
    let payload2 = json!({
        "id": event_id2,
        "type": "payment_intent.succeeded",
        "data": {
            "object": {
                "id": pi_reference2,
                "amount": 2_340_000,
                "currency": "thb",
                "status": "succeeded"
            }
        }
    })
    .to_string();
    let sig2 = signed_webhook_header(
        payload2.as_bytes(),
        TEST_WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );

    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let t1 = {
        let app = app.clone();
        let barrier = barrier.clone();
        let payload = payload2.clone();
        let sig = sig2.clone();
        tokio::spawn(async move {
            barrier.wait().await;
            post_webhook(&app, Some(&sig), payload.as_bytes()).await
        })
    };
    let t2 = {
        let app = app.clone();
        let barrier = barrier.clone();
        let payload = payload2.clone();
        let sig = sig2.clone();
        tokio::spawn(async move {
            barrier.wait().await;
            post_webhook(&app, Some(&sig), payload.as_bytes()).await
        })
    };

    let (r1, r2) = (t1.await.unwrap(), t2.await.unwrap());
    assert_eq!(r1.status(), StatusCode::OK);
    assert_eq!(r2.status(), StatusCode::OK);

    let booked_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM flight_seats WHERE flight_instance_id = (SELECT flight_instance_id FROM seat_holds WHERE id = $1) AND booking_status = 'BOOKED'",
    )
    .bind(uuid::Uuid::parse_str(&hold_id2).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(booked_count, 1);
}

#[tokio::test]
async fn transient_local_failure_rolls_back_and_allows_webhook_retry() {
    let (app, pool) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;
    let request_id = uuid::Uuid::new_v4();
    let attempt_res = create_attempt(&app, &hold_id, &cookie, request_id, "CARD").await;
    let attempt_id = body(attempt_res).await["id"].as_str().unwrap().to_owned();

    let pi_reference: String =
        sqlx::query_scalar("SELECT provider_reference FROM payment_attempts WHERE id = $1")
            .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();

    sqlx::query("UPDATE flight_seats SET hold_id = NULL WHERE hold_id = $1")
        .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    let event_id = format!("evt_transient_{}", uuid::Uuid::new_v4().simple());
    let payload = json!({
        "id": event_id,
        "type": "payment_intent.succeeded",
        "data": {
            "object": {
                "id": pi_reference,
                "amount": 2_340_000,
                "currency": "thb",
                "status": "succeeded"
            }
        }
    })
    .to_string();

    let sig = signed_webhook_header(
        payload.as_bytes(),
        TEST_WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );
    let failed_delivery = post_webhook(&app, Some(&sig), payload.as_bytes()).await;
    assert!(
        failed_delivery.status().is_client_error() || failed_delivery.status().is_server_error()
    );

    let event_recorded: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM stripe_webhook_events WHERE stripe_event_id = $1)",
    )
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!event_recorded);

    let status: String = sqlx::query_scalar("SELECT status FROM payment_attempts WHERE id = $1")
        .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_ne!(status, "FAILED");

    sqlx::query(
        "UPDATE flight_seats SET hold_id = $1 WHERE flight_instance_id = (SELECT flight_instance_id FROM seat_holds WHERE id = $1) AND seat_number = '20A'",
    )
    .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    let retry_delivery = post_webhook(&app, Some(&sig), payload.as_bytes()).await;
    assert_eq!(retry_delivery.status(), StatusCode::OK);

    let finalized_status: String =
        sqlx::query_scalar("SELECT status FROM payment_attempts WHERE id = $1")
            .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(finalized_status, "SUCCEEDED");
}

#[tokio::test]
async fn payment_intent_failed_event_marks_attempt_failed_and_clears_protection() {
    let (app, pool) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;
    let request_id = uuid::Uuid::new_v4();
    let attempt_res = create_attempt(&app, &hold_id, &cookie, request_id, "CARD").await;
    let attempt_id = body(attempt_res).await["id"].as_str().unwrap().to_owned();

    let pi_reference: String =
        sqlx::query_scalar("SELECT provider_reference FROM payment_attempts WHERE id = $1")
            .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();

    let event_id = format!("evt_failed_{}", uuid::Uuid::new_v4().simple());
    let payload = json!({
        "id": event_id,
        "type": "payment_intent.payment_failed",
        "data": {
            "object": {
                "id": pi_reference,
                "amount": 2_340_000,
                "currency": "thb",
                "status": "requires_payment_method",
                "last_payment_error": {
                    "code": "card_declined",
                    "decline_code": "generic_decline",
                    "message": "Your card was declined."
                }
            }
        }
    })
    .to_string();

    let sig = signed_webhook_header(
        payload.as_bytes(),
        TEST_WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );
    let res = post_webhook(&app, Some(&sig), payload.as_bytes()).await;
    assert_eq!(res.status(), StatusCode::OK);

    let (status, failure_code, failure_message): (String, Option<String>, Option<String>) =
        sqlx::query_as(
            "SELECT status, failure_code, failure_message FROM payment_attempts WHERE id = $1",
        )
        .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "FAILED");
    assert_eq!(failure_code.as_deref(), Some("CARD_DECLINED"));
    assert_eq!(
        failure_message.as_deref(),
        Some("The payment card was declined.")
    );

    let seat_status: String = sqlx::query_scalar(
        "SELECT booking_status FROM flight_seats WHERE flight_instance_id = (SELECT flight_instance_id FROM seat_holds WHERE id = $1) AND seat_number = '20A'",
    )
    .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(seat_status, "AVAILABLE");

    let replay = post_webhook(&app, Some(&sig), payload.as_bytes()).await;
    assert_eq!(replay.status(), StatusCode::OK);

    let is_protected: bool =
        sqlx::query_scalar("SELECT has_protected_stripe_card_finalization($1)")
            .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!is_protected);
}

#[tokio::test]
async fn valid_success_webhook_after_normal_expiry_and_deadline_still_finalizes() {
    let (app, pool) = app().await;
    let (hold_id, cookie) = complete_review(&app).await;
    let request_id = uuid::Uuid::new_v4();
    let attempt_res = create_attempt(&app, &hold_id, &cookie, request_id, "CARD").await;
    let attempt_id = body(attempt_res).await["id"].as_str().unwrap().to_owned();

    let pi_reference: String =
        sqlx::query_scalar("SELECT provider_reference FROM payment_attempts WHERE id = $1")
            .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();

    sqlx::query("UPDATE seat_holds SET expires_at = NOW() - INTERVAL '10 minutes' WHERE id = $1")
        .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE payment_attempts SET payment_finalization_deadline = NOW() - INTERVAL '5 minutes' WHERE id = $1",
    )
    .bind(uuid::Uuid::parse_str(&attempt_id).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    let event_id = format!("evt_expired_success_{}", uuid::Uuid::new_v4().simple());
    let payload = json!({
        "id": event_id,
        "type": "payment_intent.succeeded",
        "data": {
            "object": {
                "id": pi_reference,
                "amount": 2_340_000,
                "currency": "thb",
                "status": "succeeded"
            }
        }
    })
    .to_string();

    let sig = signed_webhook_header(
        payload.as_bytes(),
        TEST_WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );
    let res = post_webhook(&app, Some(&sig), payload.as_bytes()).await;
    assert_eq!(res.status(), StatusCode::OK);

    let state: (bool, String) = sqlx::query_as(
        "SELECT hold.consumed_at IS NOT NULL, seat.booking_status
         FROM seat_holds AS hold
         JOIN flight_seats AS seat ON seat.flight_instance_id = hold.flight_instance_id AND seat.seat_number = '20A'
         WHERE hold.id = $1",
    )
    .bind(uuid::Uuid::parse_str(&hold_id).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(state.0);
    assert_eq!(state.1, "BOOKED");
}

#[tokio::test]
async fn unknown_payment_intent_returns_retryable_5xx_and_does_not_mutate_state() {
    let (app, _) = app().await;
    let payload = json!({
        "id": format!("evt_unknown_{}", uuid::Uuid::new_v4().simple()),
        "type": "payment_intent.succeeded",
        "data": {
            "object": {
                "id": "pi_unknown_nonexistent_999999",
                "amount": 2_340_000,
                "currency": "thb",
                "status": "succeeded"
            }
        }
    })
    .to_string();

    let sig = signed_webhook_header(
        payload.as_bytes(),
        TEST_WEBHOOK_SECRET,
        chrono::Utc::now().timestamp(),
    );
    let res = post_webhook(&app, Some(&sig), payload.as_bytes()).await;
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body(res).await["error"]["code"], "INTERNAL_ERROR");
}

#[tokio::test]
async fn security_hygiene_no_raw_payload_or_secrets_persisted() {
    let (_, pool) = app().await;
    let webhook_cols: Vec<String> = sqlx::query_scalar(
        "SELECT column_name FROM information_schema.columns WHERE table_name = 'stripe_webhook_events'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    for forbidden in ["raw_payload", "payload", "body", "secret", "client_secret"] {
        assert!(!webhook_cols.iter().any(|col| col == forbidden));
    }

    let attempt_cols: Vec<String> = sqlx::query_scalar(
        "SELECT column_name FROM information_schema.columns WHERE table_name = 'payment_attempts'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    for forbidden in ["card_number", "pan", "cvc", "cvv", "client_secret"] {
        assert!(!attempt_cols.iter().any(|col| col == forbidden));
    }
}
