mod common;

use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    body::{Body, Bytes},
    http::{header, HeaderMap, Request, StatusCode},
    response::Response,
    Router,
};
use chrono::{NaiveDate, NaiveTime};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower::ServiceExt;
use uuid::Uuid;

use x_fly_api::{
    application::{
        external_analytics::ExternalAnalyticsService,
        external_auth::{
            ApiClientCredentialService, ExternalAuthService, ExternalCredentialCrypto,
        },
    },
    domain::{
        api_client::ApiClientScope,
        external_api::{ExternalApiCredentialPepper, PlaintextClientSecret},
    },
    infrastructure::{
        database::{
            migrate_database, verify_database_ready, SqlxExternalAnalyticsRepository,
            SqlxExternalAuthRepository, SqlxSeatHoldRepository,
        },
        external_auth_crypto::HmacExternalCredentialCrypto,
        http::build_router,
    },
    state::AppState,
};

const PEPPER: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const FIXTURE_REFERENCE_DATE: NaiveDate = NaiveDate::from_ymd_opt(2040, 1, 1).unwrap();

#[derive(Clone)]
struct Fixture {
    actor_id: Uuid,
    client_pk: Uuid,
    client_id: String,
    client_secret: String,
    namespace_date: NaiveDate,
    service_ids: Arc<Mutex<Vec<Uuid>>>,
}

async fn pools() -> (PgPool, PgPool) {
    let setup = PgPoolOptions::new()
        .max_connections(8)
        .connect(&common::test_database_url())
        .await
        .expect("connect to TEST setup database");
    migrate_database(&setup)
        .await
        .expect("TEST migration chain is current");
    let runtime = PgPoolOptions::new()
        .max_connections(8)
        .connect(&common::test_runtime_database_url())
        .await
        .expect("connect to TEST runtime database");
    verify_database_ready(&runtime)
        .await
        .expect("runtime sees ready TEST schema");
    (setup, runtime)
}

fn crypto() -> Arc<HmacExternalCredentialCrypto> {
    Arc::new(HmacExternalCredentialCrypto::from_pepper(
        ExternalApiCredentialPepper::parse_hex(PEPPER).expect("test pepper"),
    ))
}

fn unique_client_id() -> String {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let suffix: String = Uuid::new_v4()
        .as_bytes()
        .iter()
        .map(|byte| ALPHABET[(*byte & 31) as usize] as char)
        .collect();
    format!("XFC{suffix}")
}

async fn fixture(setup: &PgPool, scopes: &[ApiClientScope]) -> Fixture {
    let mut transaction = setup
        .begin()
        .await
        .expect("begin external analytics fixture");
    let actor_id: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash) VALUES ($1,'external-analytics-http') RETURNING id",
    )
    .bind(format!("external-analytics-{}@test.invalid", Uuid::new_v4()))
    .fetch_one(&mut *transaction)
    .await
    .expect("fixture actor");
    let client_id = unique_client_id();
    let client_pk: Uuid = sqlx::query_scalar(
        "INSERT INTO api_clients (
             client_id,display_name,description,status,
             created_by_staff_user_id,updated_by_staff_user_id
         ) VALUES ($1,$2,NULL,'ACTIVE',$3,$3) RETURNING id",
    )
    .bind(&client_id)
    .bind(format!("External analytics {}", Uuid::new_v4()))
    .bind(actor_id)
    .fetch_one(&mut *transaction)
    .await
    .expect("fixture client");
    for scope in scopes {
        sqlx::query(
            "INSERT INTO api_client_allowed_scopes
             (api_client_id,scope_code,assigned_by_staff_user_id)
             VALUES ($1,$2,$3)",
        )
        .bind(client_pk)
        .bind(scope.as_str())
        .bind(actor_id)
        .execute(&mut *transaction)
        .await
        .expect("fixture scope");
    }
    let secret = crypto().generate_client_secret();
    let client_secret = hex::encode(secret.as_bytes());
    let digest = crypto().credential_digest(
        &client_id,
        &PlaintextClientSecret::parse_hex(&client_secret).expect("fixture secret"),
    );
    sqlx::query(
        "INSERT INTO api_client_credentials
         (api_client_id,secret_digest,digest_version,issued_at,issued_by_staff_user_id)
         VALUES ($1,$2,1,clock_timestamp(),$3)",
    )
    .bind(client_pk)
    .bind(digest.as_bytes().as_slice())
    .bind(actor_id)
    .execute(&mut *transaction)
    .await
    .expect("fixture credential");
    transaction
        .commit()
        .await
        .expect("commit external analytics fixture");
    let namespace_date = common::allocate_test_departure_date(
        "xf-201",
        NaiveDate::from_ymd_opt(2500, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2600, 1, 1).unwrap(),
    )
    .await
    .expect("allocate analytics HTTP fixture date");
    Fixture {
        actor_id,
        client_pk,
        client_id,
        client_secret,
        namespace_date,
        service_ids: Arc::new(Mutex::new(Vec::new())),
    }
}

fn fixture_date(fixture: &Fixture, requested: NaiveDate) -> NaiveDate {
    fixture.namespace_date + requested.signed_duration_since(FIXTURE_REFERENCE_DATE)
}

async fn add_service(
    setup: &PgPool,
    fixture: &mut Fixture,
    departure: NaiveDate,
    departure_time: NaiveTime,
) -> Uuid {
    let departure = fixture_date(fixture, departure);
    let mut transaction = setup.begin().await.expect("begin analytics HTTP flight");
    let service_id: Uuid = sqlx::query_scalar(
        "INSERT INTO flight_services (
             public_id,flight_number,origin_code,destination_code,aircraft_code,
             origin_time_zone,departure_time,arrival_time,arrival_day_offset,
             duration_minutes,stops,status,operating_date
         ) VALUES ($1,$2,'BKK','DXB','A320','Asia/Bangkok',$3,'12:00',0,120,
             'DIRECT','SCHEDULED',$4) RETURNING id",
    )
    .bind(format!(
        "external-analytics-http-{}",
        Uuid::new_v4().simple()
    ))
    .bind(format!("XH-{}", Uuid::new_v4().simple()))
    .bind(departure_time)
    .bind(departure)
    .fetch_one(&mut *transaction)
    .await
    .expect("fixture flight service");
    sqlx::query("INSERT INTO flight_instances (flight_service_id,departure_date) VALUES ($1,$2)")
        .bind(service_id)
        .bind(departure)
        .execute(&mut *transaction)
        .await
        .expect("fixture flight instance");
    transaction
        .commit()
        .await
        .expect("commit analytics HTTP flight");
    fixture
        .service_ids
        .lock()
        .expect("service tracker lock")
        .push(service_id);
    service_id
}

async fn cleanup(setup: &PgPool, fixture: &Fixture) -> Result<(), sqlx::Error> {
    let mut tx = setup.begin().await?;
    sqlx::query(
        "DELETE FROM external_access_tokens
         WHERE api_client_credential_id IN
           (SELECT id FROM api_client_credentials WHERE api_client_id=$1)",
    )
    .bind(fixture.client_pk)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM api_client_management_audit WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM api_client_credentials WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(&mut *tx)
        .await?;
    let service_ids = fixture
        .service_ids
        .lock()
        .map_err(|_| sqlx::Error::Protocol("service tracker poisoned".to_owned()))?
        .clone();
    for service_id in &service_ids {
        sqlx::query("DELETE FROM flight_instances WHERE flight_service_id=$1")
            .bind(service_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM flight_services WHERE id=$1")
            .bind(service_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM api_clients WHERE id=$1")
        .bind(fixture.client_pk)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "DELETE FROM flight_instances AS instance
         USING flight_services AS service
         WHERE instance.flight_service_id=service.id
           AND service.public_id='xf-201'
           AND instance.departure_date=$1",
    )
    .bind(fixture.namespace_date)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM staff_users WHERE id=$1")
        .bind(fixture.actor_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

async fn run_fixture_test<F, Fut>(scopes: Vec<ApiClientScope>, body: F)
where
    F: FnOnce(PgPool, PgPool, Fixture) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &scopes).await;
    let cleanup_setup = setup.clone();
    let cleanup_target = fixture.clone();
    common::run_fixture_body_with_cleanup(
        move || body(setup, runtime, fixture),
        move || async move { cleanup(&cleanup_setup, &cleanup_target).await },
    )
    .await;
}

#[tokio::test]
async fn dynamically_added_service_is_cleaned_when_body_mutates_fixture() {
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, _runtime) = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::AnalyticsRead]).await;
    let body_fixture = fixture.clone();
    let cleanup_fixture = fixture.clone();
    let cleanup_setup = setup.clone();
    let body_setup = setup.clone();
    let created_service = Arc::new(Mutex::new(None));
    let created_service_for_body = Arc::clone(&created_service);

    common::run_fixture_body_with_cleanup(
        move || async move {
            let mut fixture = body_fixture;
            let service_id = add_service(
                &body_setup,
                &mut fixture,
                NaiveDate::from_ymd_opt(2040, 1, 10).expect("fixture date"),
                NaiveTime::from_hms_opt(12, 0, 0).expect("fixture time"),
            )
            .await;
            *created_service_for_body
                .lock()
                .expect("service tracker lock") = Some(service_id);
        },
        move || async move { cleanup(&cleanup_setup, &cleanup_fixture).await },
    )
    .await;

    let service_id = created_service
        .lock()
        .expect("service tracker lock")
        .expect("body created a service");
    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM flight_services WHERE id=$1")
        .bind(service_id)
        .fetch_one(&setup)
        .await
        .expect("inspect dynamically-created analytics service");
    assert_eq!(remaining, 0, "body-created service must be finalized");
}

fn state(runtime: PgPool) -> AppState {
    let booking = Arc::new(SqlxSeatHoldRepository::new(runtime.clone()));
    let auth_repository = Arc::new(SqlxExternalAuthRepository::new(runtime.clone()));
    let auth_crypto = crypto();
    let analytics_repository = Arc::new(SqlxExternalAnalyticsRepository::new(runtime));
    AppState::new(
        booking.clone(),
        booking.clone(),
        booking.clone(),
        booking,
        Duration::from_secs(600),
        false,
        "http://localhost:3000".to_owned(),
    )
    .with_external_auth(
        ApiClientCredentialService::new(auth_repository.clone(), auth_crypto.clone()),
        ExternalAuthService::new(auth_repository, auth_crypto),
    )
    .with_external_analytics(ExternalAnalyticsService::new(analytics_repository))
}

fn state_without_auth(runtime: PgPool) -> AppState {
    let booking = Arc::new(SqlxSeatHoldRepository::new(runtime));
    AppState::new(
        booking.clone(),
        booking.clone(),
        booking.clone(),
        booking,
        Duration::from_secs(600),
        false,
        "http://localhost:3000".to_owned(),
    )
}

async fn response_body(response: Response) -> (StatusCode, HeaderMap, Value) {
    let status = response.status();
    let headers = response.headers().clone();
    let bytes: Bytes = response
        .into_body()
        .collect()
        .await
        .expect("analytics HTTP body")
        .to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, body)
}

async fn exchange(router: &Router, fixture: &Fixture) -> String {
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/external/token")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "clientId": fixture.client_id,
                "clientSecret": fixture.client_secret
            })
            .to_string(),
        ))
        .expect("analytics token request");
    let response = router
        .clone()
        .oneshot(request)
        .await
        .expect("analytics token response");
    let (status, _, body) = response_body(response).await;
    assert_eq!(status, StatusCode::OK);
    body["accessToken"]
        .as_str()
        .expect("analytics access token")
        .to_owned()
}

async fn get(
    router: &Router,
    uri: &str,
    token: Option<&str>,
    origin: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method("GET").uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    if let Some(origin) = origin {
        builder = builder.header(header::ORIGIN, origin);
    }
    let response = router
        .clone()
        .oneshot(builder.body(Body::empty()).expect("analytics request"))
        .await
        .expect("analytics response");
    response_body(response).await
}

fn assert_error(body: &Value, code: &str) {
    assert_eq!(body["error"]["code"], code);
    assert!(body["error"]["message"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert!(body["error"]["requestId"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
}

#[tokio::test]
async fn analytics_requires_analytics_read() {
    run_fixture_test(vec![], move |_setup, runtime, fixture| async move {
        let router = build_router(state(runtime));
        let (status, headers, body) = get(
            &router,
            "/api/v1/external/analytics/summary?from=2040-01-01&to=2040-01-01",
            None,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
        assert_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
        let token = exchange(&router, &fixture).await;
        let (status, headers, body) = get(
            &router,
            "/api/v1/external/analytics/summary?from=2040-01-01&to=2040-01-01",
            Some(&token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
        assert_error(&body, "EXTERNAL_SCOPE_DENIED");
    })
    .await;
}

#[tokio::test]
async fn invalid_bearer_remains_authentication_failure() {
    let (_setup, runtime) = pools().await;
    let router = build_router(state(runtime));
    let (status, headers, body) = get(
        &router,
        "/api/v1/external/analytics/summary?from=2040-01-01&to=2040-01-01",
        Some("xfa_v1_0000000000000000000000000000000000000000000000000000000000000000"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
    assert_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
}

#[tokio::test]
async fn flights_scope_cannot_read_analytics() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        move |_setup, runtime, fixture| async move {
            let router = build_router(state(runtime));
            let token = exchange(&router, &fixture).await;
            let (status, headers, body) = get(
                &router,
                "/api/v1/external/analytics/summary?from=2040-01-01&to=2040-01-01",
                Some(&token),
                None,
            )
            .await;
            assert_eq!(status, StatusCode::FORBIDDEN);
            assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
            assert_error(&body, "EXTERNAL_SCOPE_DENIED");
        },
    )
    .await;
}

#[tokio::test]
async fn zero_scope_principal_is_authenticated_but_denied_by_scope_guard() {
    run_fixture_test(vec![], move |_setup, runtime, fixture| async move {
        let router = build_router(state(runtime));
        let token = exchange(&router, &fixture).await;
        let (status, _, body) = get(
            &router,
            "/api/v1/external/analytics/summary?from=2040-01-01&to=2040-01-01",
            Some(&token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_error(&body, "EXTERNAL_SCOPE_DENIED");
    })
    .await;
}

#[tokio::test]
async fn analytics_scope_returns_exact_minimal_summary_and_no_browser_cors() {
    run_fixture_test(
        vec![ApiClientScope::AnalyticsRead],
        move |setup, runtime, fixture| async move {
            let mut fixture = fixture;
            add_service(
                &setup,
                &mut fixture,
                NaiveDate::from_ymd_opt(2040, 1, 10).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )
            .await;
            let router = build_router(state(runtime));
            let token = exchange(&router, &fixture).await;
            let (status, headers, body) = get(
                &router,
                "/api/v1/external/analytics/summary?from=2040-01-10&to=2040-01-10",
                Some(&token),
                Some("https://browser.example"),
            )
            .await;
            assert_eq!(status, StatusCode::OK);
            assert!(headers.get("access-control-allow-origin").is_none());
            assert!(headers.get("access-control-allow-credentials").is_none());
            assert_eq!(
                body.as_object()
                    .unwrap()
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>(),
                vec![
                    "bookedSeats",
                    "cancelledBookings",
                    "generatedAt",
                    "occupancyPercent",
                    "period",
                    "sellableSeats",
                    "ticketsIssued",
                    "totalBookings",
                ]
            );
            assert_eq!(
                body["period"],
                json!({"from":"2040-01-10","to":"2040-01-10"})
            );
            assert_eq!(body["totalBookings"], 0);
            assert_eq!(body["ticketsIssued"], 0);
            assert_eq!(body["cancelledBookings"], 0);
            assert_eq!(body["bookedSeats"], 0);
            assert_eq!(body["sellableSeats"], 0);
            assert_eq!(body["occupancyPercent"], 0.0);
            for forbidden in [
                "revenue",
                "bookingReference",
                "ticketNumber",
                "providerReference",
                "passenger",
                "payment",
                "refund",
                "internalId",
                "uuid",
            ] {
                assert!(
                    !body.to_string().contains(forbidden),
                    "forbidden field {forbidden}"
                );
            }
        },
    )
    .await;
}

#[tokio::test]
async fn both_scopes_allow_analytics_then_current_scope_removal_denies_next_request() {
    run_fixture_test(
        vec![ApiClientScope::AnalyticsRead, ApiClientScope::FlightsRead],
        move |setup, runtime, fixture| async move {
            let router = build_router(state(runtime));
            let token = exchange(&router, &fixture).await;
            let uri = "/api/v1/external/analytics/summary?from=2040-01-01&to=2040-01-01";

            let (status, _, body) = get(&router, uri, Some(&token), None).await;
            assert_eq!(status, StatusCode::OK);
            assert!(body.get("totalBookings").is_some());

            sqlx::query(
                "DELETE FROM api_client_allowed_scopes
                 WHERE api_client_id=$1 AND scope_code=$2",
            )
            .bind(fixture.client_pk)
            .bind(ApiClientScope::AnalyticsRead.as_str())
            .execute(&setup)
            .await
            .expect("remove analytics scope");

            let (status, headers, body) = get(&router, uri, Some(&token), None).await;
            assert_eq!(status, StatusCode::FORBIDDEN);
            assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
            assert_error(&body, "EXTERNAL_SCOPE_DENIED");
        },
    )
    .await;
}

#[tokio::test]
async fn invalid_analytics_query_is_rejected_after_authentication() {
    run_fixture_test(
        vec![ApiClientScope::AnalyticsRead],
        move |_setup, runtime, fixture| async move {
            let router = build_router(state(runtime));
            let token = exchange(&router, &fixture).await;
            let (status, headers, body) = get(
                &router,
                "/api/v1/external/analytics/summary?from=not-a-date&to=2040-01-01",
                Some(&token),
                None,
            )
            .await;
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
            assert_error(&body, "EXTERNAL_REQUEST_INVALID");
        },
    )
    .await;
}

#[tokio::test]
async fn analytics_auth_repository_unavailable_fails_closed() {
    run_fixture_test(vec![], move |_setup, runtime, _fixture| async move {
        let router = build_router(state_without_auth(runtime));
        let (status, headers, body) = get(
            &router,
            "/api/v1/external/analytics/summary?from=2040-01-01&to=2040-01-01",
            Some("xfa_v1_0000000000000000000000000000000000000000000000000000000000000000"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
        assert_error(&body, "EXTERNAL_AUTH_UNAVAILABLE");
    })
    .await;
}
