use crate::domain::{
    booking_confirmation::{BookingConfirmationEmail, DeliveryFailure, RetryDisposition},
    repositories::{BookingConfirmationRepository, EmailDeliveryGateway, TicketRepository},
    ticket::Ticket,
};
use chrono::{DateTime, NaiveTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct BookingConfirmationEmailService {
    confirmations: Arc<dyn BookingConfirmationRepository>,
    tickets: Arc<dyn TicketRepository>,
    gateway: Arc<dyn EmailDeliveryGateway>,
    public_site_origin: String,
}

impl BookingConfirmationEmailService {
    pub fn new(
        confirmations: Arc<dyn BookingConfirmationRepository>,
        tickets: Arc<dyn TicketRepository>,
        gateway: Arc<dyn EmailDeliveryGateway>,
        public_site_origin: String,
    ) -> Self {
        Self {
            confirmations,
            tickets,
            gateway,
            public_site_origin,
        }
    }

    pub async fn dispatch_once(&self, now: DateTime<Utc>) -> Result<bool, String> {
        let Some(intent) = self
            .confirmations
            .claim_due_booking_confirmation(now)
            .await
            .map_err(|_| "claim failed".to_owned())?
        else {
            return Ok(false);
        };
        let ticket = match self
            .tickets
            .ensure_ticket_for_payment_attempt(intent.payment_attempt_id)
            .await
        {
            Ok(ticket) => ticket,
            Err(_) => {
                self.retry(
                    intent.id,
                    intent.attempt_count,
                    now,
                    DeliveryFailure::Connectivity,
                )
                .await?;
                return Ok(true);
            }
        };
        let email = build_email(
            &ticket,
            intent.recipient_email,
            intent.locale,
            &self.public_site_origin,
        );
        let key = format!("booking-confirmation/{}", intent.id);
        match self
            .gateway
            .send(&email.recipient, &key, &email.render())
            .await
        {
            Ok(message_id) => self
                .confirmations
                .mark_booking_confirmation_sent(intent.id, &message_id)
                .await
                .map_err(|_| "mark sent failed".to_owned())?,
            Err(failure) => match failure.retry_disposition(intent.attempt_count) {
                RetryDisposition::RetryAfter(_) => {
                    self.retry(intent.id, intent.attempt_count, now, failure)
                        .await?
                }
                RetryDisposition::Permanent => self
                    .confirmations
                    .mark_booking_confirmation_permanent(intent.id, failure)
                    .await
                    .map_err(|_| "mark permanent failed".to_owned())?,
            },
        }
        Ok(true)
    }

    async fn retry(
        &self,
        id: Uuid,
        attempt: u8,
        now: DateTime<Utc>,
        failure: DeliveryFailure,
    ) -> Result<(), String> {
        let delay = match failure.retry_disposition(attempt) {
            RetryDisposition::RetryAfter(delay) => delay,
            RetryDisposition::Permanent => {
                return self
                    .confirmations
                    .mark_booking_confirmation_permanent(id, failure)
                    .await
                    .map_err(|_| "mark permanent failed".to_owned())
            }
        };
        self.confirmations
            .mark_booking_confirmation_retry(id, now + delay, failure)
            .await
            .map_err(|_| "mark retry failed".to_owned())
    }
}

fn build_email(
    ticket: &Ticket,
    recipient: String,
    locale: crate::domain::booking_confirmation::BookingConfirmationLocale,
    origin: &str,
) -> BookingConfirmationEmail {
    BookingConfirmationEmail {
        recipient,
        locale,
        flight_number: ticket.journey.flight_number.clone(),
        origin_code: ticket.journey.origin_code.clone(),
        destination_code: ticket.journey.destination_code.clone(),
        departure_date: ticket.journey.departure_date,
        departure_time: ticket
            .journey
            .departure_time
            .as_deref()
            .and_then(|value| NaiveTime::parse_from_str(value, "%H:%M").ok()),
        departure_time_zone: None,
        cabin: ticket.journey.cabin.clone(),
        booking_reference: ticket.booking_reference.clone(),
        ticket_number: Some(ticket.ticket_number.clone()),
        manage_booking_url: format!("{}/manage-booking", origin.trim_end_matches('/')),
    }
}
