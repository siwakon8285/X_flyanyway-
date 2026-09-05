use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use super::SqlxSeatHoldRepository;
use crate::domain::{
    booking_confirmation::{BookingConfirmationLocale, DeliveryFailure},
    repositories::{BookingConfirmationIntent, BookingConfirmationRepository},
};

#[async_trait]
impl BookingConfirmationRepository for SqlxSeatHoldRepository {
    async fn claim_due_booking_confirmation(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Option<BookingConfirmationIntent>, sqlx::Error> {
        let mut tx = self.pool().begin().await?;
        let row = sqlx::query_as::<_, IntentRow>(
            "SELECT id, payment_attempt_id, recipient_email, locale, attempt_count
             FROM booking_confirmation_email_outbox
             WHERE (status = 'PENDING' AND next_attempt_at <= $1)
                OR (status = 'IN_FLIGHT' AND lease_until <= $1)
             ORDER BY next_attempt_at, created_at
             FOR UPDATE SKIP LOCKED LIMIT 1",
        )
        .bind(now)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(row) = row else {
            tx.commit().await?;
            return Ok(None);
        };
        let attempt_count = u8::try_from(row.attempt_count.saturating_add(1)).unwrap_or(6);
        sqlx::query("UPDATE booking_confirmation_email_outbox SET status = 'IN_FLIGHT', attempt_count = $2, lease_until = $3, updated_at = $1 WHERE id = $4")
            .bind(now).bind(i16::from(attempt_count)).bind(now + Duration::minutes(2)).bind(row.id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(Some(BookingConfirmationIntent {
            id: row.id,
            payment_attempt_id: row.payment_attempt_id,
            recipient_email: row.recipient_email,
            locale: BookingConfirmationLocale::parse_database(&row.locale)
                .expect("locale constraint is valid"),
            attempt_count,
        }))
    }

    async fn mark_booking_confirmation_sent(
        &self,
        id: Uuid,
        provider_message_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE booking_confirmation_email_outbox SET status = 'SENT', provider_message_id = $2, sent_at = NOW(), lease_until = NULL, updated_at = NOW() WHERE id = $1")
            .bind(id).bind(provider_message_id).execute(self.pool()).await?;
        Ok(())
    }

    async fn mark_booking_confirmation_retry(
        &self,
        id: Uuid,
        next_attempt_at: DateTime<Utc>,
        failure: DeliveryFailure,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE booking_confirmation_email_outbox SET status = 'PENDING', next_attempt_at = $2, lease_until = NULL, last_error_code = $3, updated_at = NOW() WHERE id = $1")
            .bind(id).bind(next_attempt_at).bind(failure_code(failure)).execute(self.pool()).await?;
        Ok(())
    }

    async fn mark_booking_confirmation_permanent(
        &self,
        id: Uuid,
        failure: DeliveryFailure,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE booking_confirmation_email_outbox SET status = 'PERMANENTLY_FAILED', lease_until = NULL, last_error_code = $2, updated_at = NOW() WHERE id = $1")
            .bind(id).bind(failure_code(failure)).execute(self.pool()).await?;
        Ok(())
    }
}

fn failure_code(failure: DeliveryFailure) -> &'static str {
    match failure {
        DeliveryFailure::Timeout => "TIMEOUT",
        DeliveryFailure::Connectivity => "CONNECTIVITY",
        DeliveryFailure::ProviderStatus(429) => "HTTP_429",
        DeliveryFailure::ProviderStatus(500..=599) => "HTTP_5XX",
        DeliveryFailure::ProviderStatus(_) => "HTTP_PERMANENT",
    }
}

#[derive(FromRow)]
struct IntentRow {
    id: Uuid,
    payment_attempt_id: Uuid,
    recipient_email: String,
    locale: String,
    attempt_count: i16,
}
