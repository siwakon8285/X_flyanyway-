mod common;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::{Duration as ChronoDuration, Utc};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};
use tower::ServiceExt;
use uuid::Uuid;
use x_fly_api::{
    application::staff_auth::StaffAuthService,
    domain::{repositories::TicketOperationsRepository, ticket_operations::TicketOperationsFilter},
    infrastructure::{
        database::{prepare_test_database, SqlxSeatHoldRepository, SqlxStaffAuthRepository},
        http::build_router,
        password::Argon2PasswordService,
    },
    state::AppState,
};

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}
async fn pool() -> PgPool {
    let p = PgPoolOptions::new()
        .max_connections(8)
        .connect(&common::test_database_url())
        .await
        .unwrap();
    prepare_test_database(&p).await.unwrap();
    cleanup(&p).await;
    p
}
async fn cleanup(p: &PgPool) {
    sqlx::raw_sql("DELETE FROM booking_cancellations WHERE ticket_id IN(SELECT id FROM tickets WHERE booking_reference LIKE 'XF23%'); DELETE FROM tickets WHERE booking_reference LIKE 'XF23%'; DELETE FROM seat_holds WHERE id IN(SELECT h.id FROM seat_holds h JOIN flight_instances i ON i.id=h.flight_instance_id JOIN flight_services s ON s.id=i.flight_service_id WHERE s.public_id LIKE 'ticket-http-%'); DELETE FROM flight_instances WHERE flight_service_id IN(SELECT id FROM flight_services WHERE public_id LIKE 'ticket-http-%'); DELETE FROM flight_management_audit WHERE flight_service_id IN(SELECT id FROM flight_services WHERE public_id LIKE 'ticket-http-%'); DELETE FROM flight_service_seat_templates WHERE flight_service_id IN(SELECT id FROM flight_services WHERE public_id LIKE 'ticket-http-%'); DELETE FROM flight_service_cabins WHERE flight_service_id IN(SELECT id FROM flight_services WHERE public_id LIKE 'ticket-http-%'); DELETE FROM flight_services WHERE public_id LIKE 'ticket-http-%'; DELETE FROM staff_sessions WHERE staff_user_id IN(SELECT id FROM staff_users WHERE email LIKE '%@ticket-http.test'); DELETE FROM staff_user_roles WHERE staff_user_id IN(SELECT id FROM staff_users WHERE email LIKE '%@ticket-http.test'); DELETE FROM staff_users WHERE email LIKE '%@ticket-http.test';").execute(p).await.unwrap();
}
fn app(p: PgPool) -> axum::Router {
    let repo = Arc::new(SqlxSeatHoldRepository::new(p.clone()));
    let auth = StaffAuthService::new(
        Arc::new(SqlxStaffAuthRepository::new(p)),
        Argon2PasswordService::default(),
        Duration::from_secs(3600),
    )
    .unwrap();
    build_router(
        AppState::new(
            repo.clone(),
            repo.clone(),
            repo.clone(),
            repo.clone(),
            Duration::from_secs(600),
            true,
            "http://localhost:3000".into(),
        )
        .with_staff_auth(auth)
        .with_tickets(
            repo.clone(),
            "synthetic-ticket-signing-secret-at-least-32-characters".into(),
        )
        .with_ticket_operations(repo),
    )
}
async fn cookie(p: &PgPool, role: &str, suffix: &str) -> String {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users(email,password_hash) VALUES($1,'synthetic-hash') RETURNING id",
    )
    .bind(format!("{suffix}@ticket-http.test"))
    .fetch_one(p)
    .await
    .unwrap();
    sqlx::query("INSERT INTO staff_user_roles(staff_user_id,role_code) VALUES($1,$2)")
        .bind(id)
        .bind(role)
        .execute(p)
        .await
        .unwrap();
    let token = Sha256::digest(format!("ticket-http-{suffix}").as_bytes());
    let hash: [u8; 32] = Sha256::digest(token).into();
    sqlx::query("INSERT INTO staff_sessions(staff_user_id,token_hash,expires_at) VALUES($1,$2,NOW()+INTERVAL '1 hour')").bind(id).bind(hash.as_slice()).execute(p).await.unwrap();
    format!("x_fly_staff_session={}", hex::encode(token))
}
async fn send(
    app: &axum::Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> axum::response::Response {
    let mut b = Request::builder().method(method).uri(uri);
    if let Some(c) = cookie {
        b = b.header(header::COOKIE, c)
    }
    if body.is_some() {
        b = b
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ORIGIN, "http://localhost:3000")
            .header("x-x-fly-csrf", "1")
    }
    app.clone()
        .oneshot(
            b.body(body.map_or_else(Body::empty, |v| Body::from(v.to_string())))
                .unwrap(),
        )
        .await
        .unwrap()
}
async fn body(r: axum::response::Response) -> Value {
    serde_json::from_slice(&r.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

struct Fixture {
    number: String,
    reference: String,
    ticket_id: Uuid,
}
async fn fixture(p: &PgPool, suffix: &str, cabin: &str, cancelled: bool) -> Fixture {
    let service = Uuid::new_v4();
    let date = Utc::now().date_naive() + ChronoDuration::days(30);
    let flight_number = format!("XF {}", 100 + suffix.as_bytes()[0] as u16);
    sqlx::query("INSERT INTO flight_services(id,public_id,flight_number,origin_code,destination_code,aircraft_code,origin_time_zone,departure_time,arrival_time,arrival_day_offset,duration_minutes,stops,status,operating_date) VALUES($1,$2,$3,'BKK','NRT','Airbus A350-1000','Asia/Bangkok','09:00','17:00',0,420,'DIRECT','SCHEDULED',$4)").bind(service).bind(format!("ticket-http-{suffix}")).bind(flight_number).bind(date).execute(p).await.unwrap();
    let instance: Uuid = sqlx::query_scalar(
        "INSERT INTO flight_instances(flight_service_id,departure_date) VALUES($1,$2) RETURNING id",
    )
    .bind(service)
    .bind(date)
    .fetch_one(p)
    .await
    .unwrap();
    let hold:Uuid=sqlx::query_scalar("INSERT INTO seat_holds(flight_instance_id,cabin,adults,children,infants,access_token_hash,expires_at,consumed_at,passenger_details_saved_at) VALUES($1,$2,2,0,0,$3,NOW()+INTERVAL '1 hour',NOW(),NOW()) RETURNING id").bind(instance).bind(cabin).bind([3_u8;32].as_slice()).fetch_one(p).await.unwrap();
    for (ordinal, name, gender) in [(1, "Arun", "MALE"), (2, "Mali", "UNSPECIFIED")] {
        sqlx::query("INSERT INTO hold_passengers(seat_hold_id,ordinal,passenger_type,title,given_name,family_name,date_of_birth,gender,nationality_code,passport_number,passport_issuing_country_code) VALUES($1,$2,'ADULT','MR',$3,'Synthetic','1990-01-01',$4,'TH',$5,'TH')").bind(hold).bind(ordinal).bind(name).bind(gender).bind(format!("SYN{suffix}{ordinal}")).execute(p).await.unwrap();
    }
    let attempt:Uuid=sqlx::query_scalar("INSERT INTO payment_attempts(seat_hold_id,request_id,request_fingerprint,provider,payment_method,status,amount,currency_code,review_priced_at,provider_reference,succeeded_at) VALUES($1,$2,$3,'MOCK_BITCOIN','BITCOIN','SUCCEEDED',99500,'THB',NOW(),$4,NOW()) RETURNING id").bind(hold).bind(Uuid::new_v4()).bind([4_u8;32].as_slice()).bind(format!("SYN-{suffix}")).fetch_one(p).await.unwrap();
    for (ordinal, seat, col) in [(1, "1A", "A"), (2, "1K", "K")] {
        let id:Uuid=sqlx::query_scalar("INSERT INTO flight_seats(flight_instance_id,seat_number,row_number,column_code,cabin,position,booking_status,booked_at) VALUES($1,$2,1,$3,$4,'window','BOOKED',NOW()) RETURNING id").bind(instance).bind(seat).bind(col).bind(cabin).fetch_one(p).await.unwrap();
        sqlx::query("INSERT INTO payment_attempt_seats(payment_attempt_id,flight_seat_id,passenger_ordinal) VALUES($1,$2,$3)").bind(attempt).bind(id).bind(ordinal).execute(p).await.unwrap();
    }
    let reference = format!("XF23{}", &suffix[..6]);
    let number = format!("XFT{}", &suffix[..12]);
    let ticket_id:Uuid=sqlx::query_scalar("INSERT INTO tickets(payment_attempt_id,booking_reference,ticket_number,status,cancelled_at) VALUES($1,$2,$3,$4,$5) RETURNING id").bind(attempt).bind(&reference).bind(&number).bind(if cancelled{"CANCELLED"}else{"ISSUED"}).bind(cancelled.then(Utc::now)).fetch_one(p).await.unwrap();
    Fixture {
        number,
        reference,
        ticket_id,
    }
}

#[tokio::test]
async fn authorization_is_permission_based_and_private() {
    let _g = guard().await;
    let p = pool().await;
    let router = app(p.clone());
    let response = send(&router, "GET", "/api/v1/admin/tickets", None, None).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store, private"
    );
    for (role, suffix) in [
        ("SYSTEM_ADMIN", "system"),
        ("FLIGHT_MANAGER", "flight"),
        ("BOOKING_OPERATIONS", "booking"),
        ("EXECUTIVE", "executive"),
    ] {
        let c = cookie(&p, role, suffix).await;
        assert_eq!(
            send(&router, "GET", "/api/v1/admin/tickets", Some(&c), None)
                .await
                .status(),
            StatusCode::FORBIDDEN
        )
    }
    let c = cookie(&p, "TICKET_PASSENGER_OPERATIONS", "operator").await;
    assert_eq!(
        send(&router, "GET", "/api/v1/admin/tickets", Some(&c), None)
            .await
            .status(),
        StatusCode::OK
    );
    let without_print = cookie(&p, "FLIGHT_MANAGER", "no-print").await;
    assert_eq!(
        send(
            &router,
            "GET",
            "/api/v1/admin/tickets/XFTABCDEFGHJKL/print",
            Some(&without_print),
            None
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    cleanup(&p).await
}

#[tokio::test]
async fn search_detail_preserve_multi_passenger_history_and_separate_states() {
    let _g = guard().await;
    let p = pool().await;
    let f = fixture(&p, "ABCDEFGHJKLM", "economy", false).await;
    let c = cookie(&p, "TICKET_PASSENGER_OPERATIONS", "search").await;
    let router = app(p.clone());
    sqlx::query("UPDATE flight_services SET status='CANCELLED',cancelled_at=NOW() WHERE id=(SELECT service.id FROM tickets ticket JOIN payment_attempts attempt ON attempt.id=ticket.payment_attempt_id JOIN seat_holds hold ON hold.id=attempt.seat_hold_id JOIN flight_instances instance ON instance.id=hold.flight_instance_id JOIN flight_services service ON service.id=instance.flight_service_id WHERE ticket.id=$1)").bind(f.ticket_id).execute(&p).await.unwrap();
    let result=body(send(&router,"POST","/api/v1/admin/tickets/search",Some(&c),Some(json!({"passengerName":"% Arun _ Synthetic ","cabin":"ECONOMY","limit":50,"offset":0}))).await).await;
    assert_eq!(result["items"].as_array().unwrap().len(), 0);
    let result=body(send(&router,"POST","/api/v1/admin/tickets/search",Some(&c),Some(json!({"passengerName":"Arun Synthetic","bookingReference":f.reference,"flightNumber":"xf165","origin":"bkk","destination":"nrt","ticketStatus":"ISSUED","cabin":"ECONOMY","limit":50,"offset":0}))).await).await;
    assert_eq!(result["items"][0]["ticketNumber"], f.number);
    let detail = body(
        send(
            &router,
            "GET",
            &format!("/api/v1/admin/tickets/{}", f.number),
            Some(&c),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(detail["journey"]["cabin"], "economy");
    assert_eq!(detail["paymentStatus"], "SUCCEEDED");
    assert_eq!(detail["bookingStatus"], "CONFIRMED");
    assert_eq!(detail["ticketStatus"], "ISSUED");
    assert_eq!(detail["journey"]["flightStatus"], "CANCELLED");
    assert_eq!(detail["passengers"][0]["seat"], "1A");
    assert_eq!(detail["passengers"][1]["seat"], "1K");
    assert_eq!(detail["passengers"][1]["gender"], "UNSPECIFIED");
    cleanup(&p).await
}

#[tokio::test]
async fn print_reuses_signed_ticket_and_never_mutates_authoritative_records() {
    let _g = guard().await;
    let p = pool().await;
    let issued = fixture(&p, "MNPQRSTUVWXY", "first", false).await;
    let cancelled = fixture(&p, "ZABCDEFGHJKL", "first", true).await;
    let c = cookie(&p, "TICKET_PASSENGER_OPERATIONS", "print").await;
    let router = app(p.clone());
    let page = body(
        send(
            &router,
            "GET",
            "/api/v1/admin/tickets?limit=1&offset=0",
            Some(&c),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert_eq!(page["nextOffset"], 1);
    assert_eq!(
        send(
            &router,
            "GET",
            "/api/v1/admin/tickets/not-a-ticket",
            Some(&c),
            None
        )
        .await
        .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        send(
            &router,
            "GET",
            "/api/v1/admin/tickets/XFT222222222222",
            Some(&c),
            None
        )
        .await
        .status(),
        StatusCode::NOT_FOUND
    );
    let before:(i64,i64,i64)=sqlx::query_as("SELECT (SELECT COUNT(*) FROM tickets),(SELECT COUNT(*) FROM payment_attempts),(SELECT COUNT(*) FROM booking_cancellations)").fetch_one(&p).await.unwrap();
    let printed = body(
        send(
            &router,
            "GET",
            &format!("/api/v1/admin/tickets/{}/print", issued.number),
            Some(&c),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(printed["ticketNumber"], issued.number);
    assert!(printed["qrToken"]
        .as_str()
        .unwrap()
        .starts_with(&format!("v1.{}.", issued.ticket_id)));
    assert!(printed.get("ticketId").is_none());
    assert_eq!(
        send(
            &router,
            "GET",
            &format!("/api/v1/admin/tickets/{}/print", cancelled.number),
            Some(&c),
            None
        )
        .await
        .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let after:(i64,i64,i64)=sqlx::query_as("SELECT (SELECT COUNT(*) FROM tickets),(SELECT COUNT(*) FROM payment_attempts),(SELECT COUNT(*) FROM booking_cancellations)").fetch_one(&p).await.unwrap();
    assert_eq!(before, after);
    cleanup(&p).await
}

#[tokio::test]
async fn ticket_pagination_rejects_offsets_that_cannot_advance() {
    let _g = guard().await;
    let p = pool().await;
    let c = cookie(&p, "TICKET_PASSENGER_OPERATIONS", "overflow").await;
    let router = app(p.clone());
    let response = send(
        &router,
        "GET",
        &format!("/api/v1/admin/tickets?limit=50&offset={}", i64::MAX),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let response = send(
        &router,
        "POST",
        "/api/v1/admin/tickets/search",
        Some(&c),
        Some(json!({"limit": 50, "offset": i64::MAX})),
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let repository = SqlxSeatHoldRepository::new(p.clone());
    let boundary = TicketOperationsFilter {
        ticket_number: None,
        booking_reference: None,
        passenger_name: None,
        flight_number: None,
        origin: None,
        destination: None,
        travel_date: None,
        ticket_status: None,
        cabin: None,
        limit: 50,
        offset: i64::MAX - 50,
    };
    assert!(repository.list_tickets(&boundary).await.is_ok());
    let overflow = TicketOperationsFilter {
        offset: i64::MAX - 49,
        ..boundary.clone()
    };
    assert!(matches!(
        repository.list_tickets(&overflow).await,
        Err(x_fly_api::domain::repositories::TicketOperationsRepositoryError::InvalidPagination)
    ));
    let invalid_limit = TicketOperationsFilter {
        limit: i64::MAX,
        offset: 0,
        ..boundary
    };
    assert!(matches!(
        repository.list_tickets(&invalid_limit).await,
        Err(x_fly_api::domain::repositories::TicketOperationsRepositoryError::InvalidPagination)
    ));
    cleanup(&p).await
}
