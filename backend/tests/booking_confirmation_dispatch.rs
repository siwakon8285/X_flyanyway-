use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use x_fly_api::application::booking_confirmation::BookingConfirmationEmailService;
use x_fly_api::domain::{
    booking_confirmation::{
        BookingConfirmationLocale, DeliveryFailure, RenderedBookingConfirmationEmail,
    },
    repositories::{
        BookingConfirmationIntent, BookingConfirmationRepository, EmailDeliveryGateway,
        TicketRepository, TicketRepositoryError,
    },
    ticket::{Ticket, TicketJourney, TicketPassenger, TicketStatus, TicketVerification},
};

#[derive(Clone)]
struct FakeOutbox {
    intent: Arc<Mutex<Option<BookingConfirmationIntent>>>,
    sent: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl BookingConfirmationRepository for FakeOutbox {
    async fn claim_due_booking_confirmation(
        &self,
        _now: chrono::DateTime<Utc>,
    ) -> Result<Option<BookingConfirmationIntent>, sqlx::Error> {
        let mut intent = self.intent.lock().unwrap();
        let value = intent.take();
        Ok(value.map(|mut value| {
            value.attempt_count += 1;
            value
        }))
    }
    async fn mark_booking_confirmation_sent(
        &self,
        _id: Uuid,
        provider_message_id: &str,
    ) -> Result<(), sqlx::Error> {
        self.sent
            .lock()
            .unwrap()
            .push(provider_message_id.to_owned());
        Ok(())
    }
    async fn mark_booking_confirmation_retry(
        &self,
        _id: Uuid,
        _next: chrono::DateTime<Utc>,
        _failure: DeliveryFailure,
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }
    async fn mark_booking_confirmation_permanent(
        &self,
        _id: Uuid,
        _failure: DeliveryFailure,
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }
}

#[derive(Clone)]
struct FakeTickets;

#[async_trait]
impl TicketRepository for FakeTickets {
    async fn issue_ticket(
        &self,
        _hold_id: Uuid,
        _token_hash: [u8; 32],
        _payment_attempt_id: Uuid,
    ) -> Result<Ticket, TicketRepositoryError> {
        Ok(ticket())
    }
    async fn verify_ticket(
        &self,
        _ticket_id: Uuid,
    ) -> Result<Option<TicketVerification>, TicketRepositoryError> {
        Ok(None)
    }
    async fn ensure_ticket_for_payment_attempt(
        &self,
        _payment_attempt_id: Uuid,
    ) -> Result<Ticket, TicketRepositoryError> {
        Ok(ticket())
    }
}

#[derive(Clone)]
struct RecordingGateway {
    calls: Arc<Mutex<Vec<(String, String, String)>>>,
}

#[async_trait]
impl EmailDeliveryGateway for RecordingGateway {
    async fn send(
        &self,
        recipient: &str,
        key: &str,
        email: &RenderedBookingConfirmationEmail,
    ) -> Result<String, DeliveryFailure> {
        self.calls
            .lock()
            .unwrap()
            .push((recipient.to_owned(), key.to_owned(), email.html.clone()));
        Ok("resend-message-id".to_owned())
    }
}

fn ticket() -> Ticket {
    Ticket {
        id: Uuid::new_v4(),
        booking_reference: "XFABCDEFGH".to_owned(),
        ticket_number: "XFTABCDEFGHJKLM".to_owned(),
        status: TicketStatus::Issued,
        issued_at: Utc::now(),
        payment_status: "SUCCEEDED".to_owned(),
        amount: 23400,
        currency_code: "THB".to_owned(),
        journey: TicketJourney {
            flight_number: "XF 701".to_owned(),
            origin_code: "BKK".to_owned(),
            destination_code: "DXB".to_owned(),
            departure_date: NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(),
            departure_time: Some("09:15".to_owned()),
            arrival_time: Some("15:45".to_owned()),
            arrival_day_offset: Some(0),
            cabin: "business".to_owned(),
        },
        passengers: vec![TicketPassenger {
            display_name: "Nara Test".to_owned(),
        }],
        seats: vec!["20A".to_owned()],
    }
}

#[tokio::test]
async fn dispatcher_ensures_ticket_then_sends_one_recorded_email_without_network() {
    let outbox = FakeOutbox {
        intent: Arc::new(Mutex::new(Some(BookingConfirmationIntent {
            id: Uuid::new_v4(),
            payment_attempt_id: Uuid::new_v4(),
            recipient_email: "contact@example.test".to_owned(),
            locale: BookingConfirmationLocale::Th,
            attempt_count: 0,
        }))),
        sent: Arc::new(Mutex::new(Vec::new())),
    };
    let gateway = RecordingGateway {
        calls: Arc::new(Mutex::new(Vec::new())),
    };
    let service = BookingConfirmationEmailService::new(
        Arc::new(outbox.clone()),
        Arc::new(FakeTickets),
        Arc::new(gateway.clone()),
        "https://x-fly.example.test".to_owned(),
    );
    assert!(service.dispatch_once(Utc::now()).await.unwrap());
    let calls = gateway.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "contact@example.test");
    assert!(calls[0].1.starts_with("booking-confirmation/"));
    assert!(calls[0].2.contains("/manage-booking"));
    assert!(calls[0].2.contains("ยืนยันการจองแล้ว"));
    assert_eq!(
        outbox.sent.lock().unwrap().as_slice(),
        &["resend-message-id"]
    );
}
