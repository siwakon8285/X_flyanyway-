mod common;

use std::{sync::Arc, time::Duration};

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
        external_auth::{
            ApiClientCredentialService, ExternalAuthService, ExternalCredentialCrypto,
        },
        external_flights::ExternalFlightService,
        flight::FlightManagement,
    },
    domain::{
        api_client::ApiClientScope,
        external_api::{ExternalApiCredentialPepper, PlaintextClientSecret},
    },
    infrastructure::{
        database::{
            migrate_database, verify_database_ready, SqlxExternalAuthRepository,
            SqlxFlightRepository, SqlxSeatHoldRepository,
        },
        external_auth_crypto::HmacExternalCredentialCrypto,
        http::build_router,
    },
    state::AppState,
};

const PEPPER: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

struct FlightFixture {
    id: Uuid,
    public_id: String,
    departure: NaiveDate,
}

struct Fixture {
    actor_id: Uuid,
    client_pk: Uuid,
    client_id: String,
    client_secret: String,
    flights: Vec<FlightFixture>,
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
    let actor_id: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash) VALUES ($1,'external-flights-http') RETURNING id",
    )
    .bind(format!("external-flights-{}@test.invalid", Uuid::new_v4()))
    .fetch_one(setup)
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
    .bind(format!("External flights {}", Uuid::new_v4()))
    .bind(actor_id)
    .fetch_one(setup)
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
        .execute(setup)
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
    .execute(setup)
    .await
    .expect("fixture credential");

    Fixture {
        actor_id,
        client_pk,
        client_id,
        client_secret,
        flights: Vec::new(),
    }
}

async fn add_flight(
    setup: &PgPool,
    fixture: &mut Fixture,
    departure_time: NaiveTime,
    flight_number_suffix: &str,
) {
    let public_id = format!("xf-ext-{}", Uuid::new_v4().simple());
    let flight_number = format!("XF-EXT-{flight_number_suffix}-{}", Uuid::new_v4());
    let departure = NaiveDate::from_ymd_opt(2099, 12, 30).expect("fixture date");
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO flight_services (
             public_id,flight_number,origin_code,destination_code,aircraft_code,
             origin_time_zone,departure_time,arrival_time,arrival_day_offset,
             duration_minutes,stops,status,operating_date
         ) VALUES ($1,$2,'BKK','DXB','Boeing 787-9','Asia/Bangkok',$3,'13:05',0,285,'DIRECT','SCHEDULED',$4)
         RETURNING id",
    )
    .bind(&public_id)
    .bind(&flight_number)
    .bind(departure_time)
    .bind(departure)
    .fetch_one(setup)
    .await
    .expect("fixture flight");

    for (cabin, fare) in [
        ("economy", 9_900_i64),
        ("business", 46_900),
        ("first", 78_900),
    ] {
        sqlx::query(
            "INSERT INTO flight_service_cabins
             (flight_service_id,cabin,base_fare_amount,currency_code)
             VALUES ($1,$2,$3,'THB')",
        )
        .bind(id)
        .bind(cabin)
        .bind(fare)
        .execute(setup)
        .await
        .expect("fixture cabin");
    }

    fixture.flights.push(FlightFixture {
        id,
        public_id,
        departure,
    });
}

async fn cleanup(setup: &PgPool, fixture: &Fixture) {
    let mut transaction = setup.begin().await.expect("begin fixture cleanup");
    sqlx::query(
        "DELETE FROM external_access_tokens
         WHERE api_client_credential_id IN (
             SELECT id FROM api_client_credentials WHERE api_client_id=$1
         )",
    )
    .bind(fixture.client_pk)
    .execute(&mut *transaction)
    .await
    .expect("clean fixture tokens");
    sqlx::query("DELETE FROM api_client_management_audit WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(&mut *transaction)
        .await
        .expect("clean fixture audit");
    sqlx::query("DELETE FROM api_client_credentials WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(&mut *transaction)
        .await
        .expect("clean fixture credentials");
    sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(&mut *transaction)
        .await
        .expect("clean fixture scopes");
    for flight in &fixture.flights {
        sqlx::query("DELETE FROM flight_service_cabins WHERE flight_service_id=$1")
            .bind(flight.id)
            .execute(&mut *transaction)
            .await
            .expect("clean fixture cabins");
        sqlx::query("DELETE FROM flight_services WHERE id=$1")
            .bind(flight.id)
            .execute(&mut *transaction)
            .await
            .expect("clean fixture flight");
    }
    sqlx::query("DELETE FROM api_clients WHERE id=$1")
        .bind(fixture.client_pk)
        .execute(&mut *transaction)
        .await
        .expect("clean fixture client");
    sqlx::query("DELETE FROM staff_users WHERE id=$1")
        .bind(fixture.actor_id)
        .execute(&mut *transaction)
        .await
        .expect("clean fixture actor");
    transaction.commit().await.expect("commit fixture cleanup");
}

fn state(setup_runtime: PgPool) -> AppState {
    let booking = Arc::new(SqlxSeatHoldRepository::new(setup_runtime.clone()));
    let flights = FlightManagement::new(Arc::new(SqlxFlightRepository::new(setup_runtime.clone())));
    let external_flights = ExternalFlightService::new(flights.clone());
    let repository = Arc::new(SqlxExternalAuthRepository::new(setup_runtime));
    let crypto = crypto();
    AppState::new(
        booking.clone(),
        booking.clone(),
        booking.clone(),
        booking,
        Duration::from_secs(600),
        false,
        "http://localhost:3000".to_owned(),
    )
    .with_flights(flights)
    .with_external_flights(external_flights)
    .with_external_auth(
        ApiClientCredentialService::new(repository.clone(), crypto.clone()),
        ExternalAuthService::new(repository, crypto),
    )
}

async fn response_body(response: Response) -> (StatusCode, HeaderMap, Value) {
    let status = response.status();
    let headers = response.headers().clone();
    let bytes: Bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, body)
}

async fn request(
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
        .oneshot(builder.body(Body::empty()).expect("flight request"))
        .await
        .expect("flight response");
    response_body(response).await
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
        .expect("token request");
    let response = router
        .clone()
        .oneshot(request)
        .await
        .expect("token response");
    let (status, _, body) = response_body(response).await;
    assert_eq!(status, StatusCode::OK);
    body["accessToken"]
        .as_str()
        .expect("access token")
        .to_owned()
}

fn assert_error(body: &Value, code: &str) {
    assert_eq!(body["error"]["code"].as_str(), Some(code));
    assert!(body["error"]["message"].as_str().is_some());
    assert!(body["error"]["requestId"].as_str().is_some());
}

fn assert_external_fields(body: &Value) {
    let object = body.as_object().expect("external flight object");
    let mut keys = object.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "aircraftCode",
            "arrivalDayOffset",
            "arrivalTime",
            "cabinPrices",
            "departureTime",
            "destinationCode",
            "durationMinutes",
            "flightNumber",
            "flightPublicId",
            "originCode",
            "status",
            "stops",
        ]
    );
    assert!(!object.contains_key("id"));
    assert!(!object.contains_key("version"));
    assert!(!object.contains_key("originTimeZone"));
    assert!(!object.contains_key("destinationTimeZone"));
    assert!(!object.contains_key("business"));
    assert!(!object.contains_key("first"));
    assert!(!object.contains_key("audit"));
    assert!(!object.contains_key("inventory"));
}

#[tokio::test]
async fn flights_search_requires_flights_read() {
    let (setup, runtime) = pools().await;
    let mut fixture = fixture(&setup, &[]).await;
    add_flight(
        &setup,
        &mut fixture,
        NaiveTime::from_hms_opt(9, 20, 0).unwrap(),
        "A",
    )
    .await;
    let router = build_router(state(runtime));
    let uri = format!(
        "/api/v1/external/flights?origin=BKK&destination=DXB&departure={}&cabin=business",
        fixture.flights[0].departure
    );
    let (status, headers, body) = request(&router, &uri, None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
    assert_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn flights_search_returns_only_external_fields() {
    let (setup, runtime) = pools().await;
    let mut fixture = fixture(&setup, &[ApiClientScope::FlightsRead]).await;
    add_flight(
        &setup,
        &mut fixture,
        NaiveTime::from_hms_opt(9, 20, 0).unwrap(),
        "A",
    )
    .await;
    add_flight(
        &setup,
        &mut fixture,
        NaiveTime::from_hms_opt(8, 10, 0).unwrap(),
        "B",
    )
    .await;
    let router = build_router(state(runtime));
    let token = exchange(&router, &fixture).await;
    let uri = format!(
        "/api/v1/external/flights?origin=BKK&destination=DXB&departure={}&cabin=business",
        fixture.flights[0].departure
    );
    let (status, headers, body) =
        request(&router, &uri, Some(&token), Some("https://example.test")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(headers.get("access-control-allow-origin").is_none());
    assert!(headers.get("access-control-allow-credentials").is_none());
    assert_eq!(
        body.as_object()
            .map(|object| object.keys().cloned().collect::<Vec<_>>()),
        Some(vec!["items".to_owned()])
    );
    let items = body["items"].as_array().expect("items");
    let fixture_items = items
        .iter()
        .filter(|item| {
            item["flightPublicId"] == fixture.flights[0].public_id
                || item["flightPublicId"] == fixture.flights[1].public_id
        })
        .collect::<Vec<_>>();
    assert_eq!(fixture_items.len(), 2);
    assert_eq!(
        fixture_items[0]["flightPublicId"],
        fixture.flights[1].public_id
    );
    assert_eq!(
        fixture_items[1]["flightPublicId"],
        fixture.flights[0].public_id
    );
    for item in fixture_items {
        assert_external_fields(item);
        assert_eq!(item["originCode"], "BKK");
        assert_eq!(item["destinationCode"], "DXB");
        assert_eq!(item["status"], "scheduled");
        assert_eq!(item["stops"], "direct");
        assert_eq!(item["aircraftCode"], "Boeing 787-9");
        assert_eq!(item["cabinPrices"].as_array().unwrap().len(), 2);
        assert_eq!(
            item["cabinPrices"][0],
            json!({"amountThb": 46900, "cabin": "business"})
        );
        assert_eq!(
            item["cabinPrices"][1],
            json!({"amountThb": 78900, "cabin": "first"})
        );
    }
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn empty_search_returns_an_empty_items_page() {
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead]).await;
    let router = build_router(state(runtime));
    let token = exchange(&router, &fixture).await;
    let (status, _, body) = request(
        &router,
        "/api/v1/external/flights?origin=HND&destination=JFK&departure=2099-12-31&cabin=business",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"items": []}));
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn flight_detail_requires_flights_read() {
    let (setup, runtime) = pools().await;
    let mut fixture = fixture(&setup, &[]).await;
    add_flight(
        &setup,
        &mut fixture,
        NaiveTime::from_hms_opt(9, 20, 0).unwrap(),
        "A",
    )
    .await;
    let router = build_router(state(runtime));
    let uri = format!(
        "/api/v1/external/flights/{}?departure={}&cabin=business",
        fixture.flights[0].public_id, fixture.flights[0].departure
    );
    let (status, headers, body) = request(&router, &uri, None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
    assert_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn flight_detail_uses_public_id_not_internal_uuid() {
    let (setup, runtime) = pools().await;
    let mut fixture = fixture(&setup, &[ApiClientScope::FlightsRead]).await;
    add_flight(
        &setup,
        &mut fixture,
        NaiveTime::from_hms_opt(9, 20, 0).unwrap(),
        "A",
    )
    .await;
    let router = build_router(state(runtime));
    let token = exchange(&router, &fixture).await;
    let valid_uri = format!(
        "/api/v1/external/flights/{}?departure={}&cabin=business",
        fixture.flights[0].public_id, fixture.flights[0].departure
    );
    let (status, _, body) = request(&router, &valid_uri, Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_external_fields(&body);
    let internal_uri = format!(
        "/api/v1/external/flights/{}?departure={}&cabin=business",
        fixture.flights[0].id, fixture.flights[0].departure
    );
    let (status, _, body) = request(&router, &internal_uri, Some(&token), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&body, "EXTERNAL_RESOURCE_NOT_FOUND");
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn flight_dto_exposes_aircraft_code_only() {
    let (setup, runtime) = pools().await;
    let mut fixture = fixture(&setup, &[ApiClientScope::FlightsRead]).await;
    add_flight(
        &setup,
        &mut fixture,
        NaiveTime::from_hms_opt(9, 20, 0).unwrap(),
        "A",
    )
    .await;
    let router = build_router(state(runtime));
    let token = exchange(&router, &fixture).await;
    let uri = format!(
        "/api/v1/external/flights/{}?departure={}&cabin=business",
        fixture.flights[0].public_id, fixture.flights[0].departure
    );
    let (_, _, body) = request(&router, &uri, Some(&token), None).await;
    assert_external_fields(&body);
    assert_eq!(body["aircraftCode"], "Boeing 787-9");
    assert!(body.get("aircraftDescription").is_none());
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn invalid_flight_filters_are_400() {
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead]).await;
    let router = build_router(state(runtime));
    let token = exchange(&router, &fixture).await;
    let (status, _, body) = request(
        &router,
        "/api/v1/external/flights?origin=BA&destination=DXB&departure=2099-12-30&cabin=business",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "EXTERNAL_REQUEST_INVALID");
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn missing_flight_is_404() {
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead]).await;
    let router = build_router(state(runtime));
    let token = exchange(&router, &fixture).await;
    let (status, _, body) = request(
        &router,
        "/api/v1/external/flights/xf-999-20991230?departure=2099-12-30&cabin=business",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&body, "EXTERNAL_RESOURCE_NOT_FOUND");
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn analytics_scope_cannot_read_flights() {
    let (setup, runtime) = pools().await;
    let mut fixture = fixture(&setup, &[ApiClientScope::AnalyticsRead]).await;
    add_flight(
        &setup,
        &mut fixture,
        NaiveTime::from_hms_opt(9, 20, 0).unwrap(),
        "A",
    )
    .await;
    let router = build_router(state(runtime));
    let token = exchange(&router, &fixture).await;
    let uri = format!(
        "/api/v1/external/flights?origin=BKK&destination=DXB&departure={}&cabin=business",
        fixture.flights[0].departure
    );
    let (status, headers, body) = request(&router, &uri, Some(&token), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(headers.get(header::WWW_AUTHENTICATE).is_none());
    assert_error(&body, "EXTERNAL_SCOPE_DENIED");
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn scope_removal_is_seen_by_external_flights_next_request() {
    let (setup, runtime) = pools().await;
    let mut fixture = fixture(&setup, &[ApiClientScope::FlightsRead]).await;
    add_flight(
        &setup,
        &mut fixture,
        NaiveTime::from_hms_opt(9, 20, 0).unwrap(),
        "A",
    )
    .await;
    let router = build_router(state(runtime));
    let token = exchange(&router, &fixture).await;
    let uri = format!(
        "/api/v1/external/flights?origin=BKK&destination=DXB&departure={}&cabin=business",
        fixture.flights[0].departure
    );
    let (status, _, _) = request(&router, &uri, Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1 AND scope_code='flights:read'")
        .bind(fixture.client_pk)
        .execute(&setup)
        .await
        .expect("remove fixture scope");
    let (status, _, body) = request(&router, &uri, Some(&token), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&body, "EXTERNAL_SCOPE_DENIED");
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn invalid_bearer_remains_generic_401() {
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead]).await;
    let router = build_router(state(runtime));
    let (status, headers, body) = request(
        &router,
        "/api/v1/external/flights?origin=BKK&destination=DXB&departure=2099-12-30&cabin=business",
        Some("not-a-token"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
    assert_error(&body, "EXTERNAL_AUTHENTICATION_FAILED");
    cleanup(&setup, &fixture).await;
}
