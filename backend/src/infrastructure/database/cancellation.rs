use super::SqlxSeatHoldRepository;
use crate::domain::{
    booking_management::BookingDetail,
    cancellation::{
        Cancellation, Clock, ProviderRefund, ProviderRefundStatus, RefundFailure, RefundJob,
        RefundStatus, StaffCancellationActor, StripeRefundEvent, MAX_REFUND_ATTEMPTS,
    },
    extras::Money,
    payment::PaymentProvider,
    repositories::{CancellationRepository, CancellationRepositoryError},
};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
struct CancelRow {
    booking_reference: String,
    payment_attempt_id: Uuid,
    ticket_status: String,
    payment_status: String,
    provider: String,
    provider_reference: Option<String>,
    amount: i64,
    currency_code: String,
    consumed_at: Option<DateTime<Utc>>,
    departure_at: Option<DateTime<Utc>>,
}
#[derive(FromRow)]
struct CancellationRow {
    id: Uuid,
    refund_status: String,
    refund_amount: i64,
    currency_code: String,
    cancelled_at: DateTime<Utc>,
}
impl CancellationRow {
    fn domain(self) -> Cancellation {
        Cancellation {
            id: self.id,
            refund_status: RefundStatus::parse_database(&self.refund_status)
                .expect("valid refund status"),
            refund_amount: Money {
                amount: self.refund_amount,
                currency_code: self.currency_code,
            },
            cancelled_at: self.cancelled_at,
        }
    }
}

#[async_trait]
impl CancellationRepository for SqlxSeatHoldRepository {
    async fn cancel_booking(
        &self,
        ticket_id: Uuid,
        clock: &dyn Clock,
        staff_actor: Option<&StaffCancellationActor>,
    ) -> Result<Cancellation, CancellationRepositoryError> {
        cancel_booking_inner(self, ticket_id, clock, staff_actor, false)
            .await
            .map(|result| result.cancellation)
    }

    async fn cancel_booking_for_staff(
        &self,
        ticket_id: Uuid,
        clock: &dyn Clock,
        staff_actor: &StaffCancellationActor,
    ) -> Result<BookingDetail, CancellationRepositoryError> {
        let result = cancel_booking_inner(self, ticket_id, clock, Some(staff_actor), true).await?;
        result
            .detail
            .ok_or(CancellationRepositoryError::InconsistentState)
    }
    async fn claim_due_refund(&self, now: DateTime<Utc>) -> Result<Option<RefundJob>, sqlx::Error> {
        let mut tx = self.pool().begin().await?;
        sqlx::query(
            "WITH exhausted AS (
                SELECT id
                FROM booking_cancellations
                WHERE refund_status='IN_FLIGHT'
                  AND lease_until <= $1
                  AND attempt_count >= $2
                ORDER BY lease_until,created_at
                FOR UPDATE SKIP LOCKED
                LIMIT 1
             )
             UPDATE booking_cancellations cancellation
             SET refund_status='REQUIRES_ATTENTION',
                 lease_until=NULL,
                 lease_token=NULL,
                 last_error_code=COALESCE(last_error_code,'REFUND_LEASE_EXPIRED_AT_MAX'),
                 updated_at=$1
             FROM exhausted
             WHERE cancellation.id=exhausted.id",
        )
        .bind(now)
        .bind(i16::from(MAX_REFUND_ATTEMPTS))
        .execute(&mut *tx)
        .await?;
        let row=sqlx::query_as::<_,RefundJobRow>("WITH candidate AS (SELECT id FROM booking_cancellations WHERE ((refund_status IN ('PENDING','PROCESSING') AND next_attempt_at<=$1) OR (refund_status='IN_FLIGHT' AND lease_until<=$1)) AND attempt_count<$2 ORDER BY next_attempt_at,created_at FOR UPDATE SKIP LOCKED LIMIT 1) UPDATE booking_cancellations c SET refund_status='IN_FLIGHT',attempt_count=attempt_count+1,lease_until=$1+INTERVAL '2 minutes',lease_token=gen_random_uuid(),updated_at=$1 FROM candidate WHERE c.id=candidate.id RETURNING c.id,c.payment_attempt_id,c.refund_provider,c.provider_refund_id,c.refund_amount,c.currency_code,c.attempt_count,c.lease_token,(SELECT provider_reference FROM payment_attempts WHERE id=c.payment_attempt_id) provider_payment_id").bind(now).bind(i16::from(MAX_REFUND_ATTEMPTS)).fetch_optional(&mut *tx).await?;
        tx.commit().await?;
        Ok(row.and_then(RefundJobRow::domain))
    }
    async fn mark_refund_result(
        &self,
        job: &RefundJob,
        r: &ProviderRefund,
        now: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        let status = match r.status {
            ProviderRefundStatus::Processing => "PROCESSING",
            ProviderRefundStatus::Succeeded => "SUCCEEDED",
            ProviderRefundStatus::RequiresAttention => "REQUIRES_ATTENTION",
        };
        sqlx::query("UPDATE booking_cancellations SET refund_status=$3,provider_refund_id=$4,refunded_at=CASE WHEN $3='SUCCEEDED' THEN $5 ELSE NULL END,next_attempt_at=CASE WHEN $3='PROCESSING' THEN $5+INTERVAL '5 minutes' ELSE next_attempt_at END,lease_until=NULL,lease_token=NULL,last_error_code=NULL,updated_at=$5 WHERE id=$1 AND refund_status='IN_FLIGHT' AND lease_token=$2").bind(job.id).bind(job.lease_token).bind(status).bind(&r.id).bind(now).execute(self.pool()).await?;
        Ok(())
    }
    async fn mark_refund_retry(
        &self,
        job: &RefundJob,
        next: DateTime<Utc>,
        f: &RefundFailure,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE booking_cancellations SET refund_status='PENDING',next_attempt_at=$3,lease_until=NULL,lease_token=NULL,last_error_code=$4,updated_at=NOW() WHERE id=$1 AND refund_status='IN_FLIGHT' AND lease_token=$2").bind(job.id).bind(job.lease_token).bind(next).bind(code(f)).execute(self.pool()).await?;
        Ok(())
    }
    async fn mark_refund_attention(
        &self,
        job: &RefundJob,
        f: &RefundFailure,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE booking_cancellations SET refund_status='REQUIRES_ATTENTION',lease_until=NULL,lease_token=NULL,last_error_code=$3,updated_at=NOW() WHERE id=$1 AND refund_status='IN_FLIGHT' AND lease_token=$2").bind(job.id).bind(job.lease_token).bind(code(f)).execute(self.pool()).await?;
        Ok(())
    }
    async fn process_stripe_refund_event(
        &self,
        event: StripeRefundEvent,
    ) -> Result<(), CancellationRepositoryError> {
        let mut tx = self
            .pool()
            .begin()
            .await
            .map_err(CancellationRepositoryError::Infrastructure)?;
        let row:Option<(String,i64,String,Option<String>,String)>=sqlx::query_as("SELECT p.provider_reference,c.refund_amount,c.currency_code,c.provider_refund_id,c.refund_status FROM booking_cancellations c JOIN payment_attempts p ON p.id=c.payment_attempt_id WHERE c.id=$1 AND c.refund_provider='STRIPE' FOR UPDATE OF c").bind(event.refund.cancellation_id).fetch_optional(&mut *tx).await.map_err(CancellationRepositoryError::Infrastructure)?;
        let Some((payment_id, amount, currency, existing, status)) = row else {
            return Err(CancellationRepositoryError::NotFound);
        };
        if payment_id != event.refund.payment_id
            || amount.checked_mul(100) != Some(event.refund.amount_minor)
            || !currency.eq_ignore_ascii_case(&event.refund.currency)
            || existing.as_deref().is_some_and(|id| id != event.refund.id)
        {
            return Err(CancellationRepositoryError::InconsistentState);
        }
        let inserted=sqlx::query("INSERT INTO stripe_refund_events(stripe_event_id,cancellation_id,refund_id,event_type) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING").bind(&event.event_id).bind(event.refund.cancellation_id).bind(&event.refund.id).bind(&event.event_type).execute(&mut *tx).await.map_err(CancellationRepositoryError::Infrastructure)?.rows_affected();
        if inserted == 1 && status != "SUCCEEDED" {
            let next = match event.refund.status {
                ProviderRefundStatus::Processing => "PROCESSING",
                ProviderRefundStatus::Succeeded => "SUCCEEDED",
                ProviderRefundStatus::RequiresAttention => "REQUIRES_ATTENTION",
            };
            sqlx::query("UPDATE booking_cancellations SET refund_status=$2,provider_refund_id=$3,refunded_at=CASE WHEN $2='SUCCEEDED' THEN NOW() ELSE NULL END,lease_until=NULL,lease_token=NULL,last_error_code=NULL,updated_at=NOW() WHERE id=$1 AND refund_status<>'SUCCEEDED'").bind(event.refund.cancellation_id).bind(next).bind(&event.refund.id).execute(&mut *tx).await.map_err(CancellationRepositoryError::Infrastructure)?;
        }
        tx.commit()
            .await
            .map_err(CancellationRepositoryError::Infrastructure)?;
        Ok(())
    }
}

struct CancellationOutcome {
    cancellation: Cancellation,
    detail: Option<BookingDetail>,
}

async fn cancel_booking_inner(
    repository: &SqlxSeatHoldRepository,
    ticket_id: Uuid,
    clock: &dyn Clock,
    staff_actor: Option<&StaffCancellationActor>,
    include_detail: bool,
) -> Result<CancellationOutcome, CancellationRepositoryError> {
    let mut tx = repository
        .pool()
        .begin()
        .await
        .map_err(CancellationRepositoryError::Infrastructure)?;
    let row = sqlx::query_as::<_, CancelRow>(
        "SELECT t.booking_reference,p.id payment_attempt_id,t.status ticket_status,
                p.status payment_status,p.provider,p.provider_reference,p.amount,p.currency_code,
                h.consumed_at,
                CASE WHEN s.departure_time IS NULL OR s.origin_time_zone IS NULL THEN NULL
                     ELSE (i.departure_date+s.departure_time) AT TIME ZONE s.origin_time_zone END departure_at
         FROM tickets t
         JOIN payment_attempts p ON p.id=t.payment_attempt_id
         JOIN seat_holds h ON h.id=p.seat_hold_id
         JOIN flight_instances i ON i.id=h.flight_instance_id
         JOIN flight_services s ON s.id=i.flight_service_id
         WHERE t.id=$1 FOR UPDATE OF t,p,h,i,s",
    )
    .bind(ticket_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(CancellationRepositoryError::Infrastructure)?
    .ok_or(CancellationRepositoryError::NotFound)?;

    if let Some(existing) = sqlx::query_as::<_, CancellationRow>(
        "SELECT id,refund_status,refund_amount,currency_code,cancelled_at
         FROM booking_cancellations WHERE ticket_id=$1",
    )
    .bind(ticket_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(CancellationRepositoryError::Infrastructure)?
    {
        let cancellation = existing.domain();
        let detail = if include_detail {
            super::booking_management::load_booking_detail(
                &mut tx,
                &row.booking_reference,
                clock.now(),
            )
            .await
            .map_err(map_booking_detail_error)?
        } else {
            None
        };
        tx.commit()
            .await
            .map_err(CancellationRepositoryError::Infrastructure)?;
        return Ok(CancellationOutcome {
            cancellation,
            detail,
        });
    }

    let seat_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT ps.flight_seat_id
         FROM payment_attempt_seats ps
         JOIN flight_seats fs ON fs.id=ps.flight_seat_id
         WHERE ps.payment_attempt_id=$1 AND ps.released_at IS NULL
         ORDER BY ps.flight_seat_id FOR UPDATE OF ps,fs",
    )
    .bind(row.payment_attempt_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(CancellationRepositoryError::Infrastructure)?;
    let now = clock.now();
    let departure = row
        .departure_at
        .ok_or(CancellationRepositoryError::InconsistentState)?;
    if row.ticket_status != "ISSUED"
        || row.payment_status != "SUCCEEDED"
        || row.consumed_at.is_none()
        || row.provider_reference.is_none()
        || seat_ids.is_empty()
        || now > departure - Duration::hours(24)
    {
        return Err(CancellationRepositoryError::Ineligible);
    }
    let provider = PaymentProvider::parse_database(&row.provider)
        .ok_or(CancellationRepositoryError::InconsistentState)?;
    let cancellation = sqlx::query_as::<_, CancellationRow>(
        "INSERT INTO booking_cancellations(
            ticket_id,payment_attempt_id,refund_provider,refund_amount,currency_code,
            requested_at,cancelled_at
         ) VALUES($1,$2,$3,$4,$5,$6,$6)
         RETURNING id,refund_status,refund_amount,currency_code,cancelled_at",
    )
    .bind(ticket_id)
    .bind(row.payment_attempt_id)
    .bind(provider.as_str())
    .bind(row.amount)
    .bind(&row.currency_code)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(CancellationRepositoryError::Infrastructure)?
    .domain();
    sqlx::query(
        "UPDATE tickets SET status='CANCELLED',cancelled_at=$2
         WHERE id=$1 AND status='ISSUED'",
    )
    .bind(ticket_id)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(CancellationRepositoryError::Infrastructure)?;
    sqlx::query(
        "UPDATE payment_attempt_seats SET released_at=$2
         WHERE payment_attempt_id=$1 AND released_at IS NULL",
    )
    .bind(row.payment_attempt_id)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(CancellationRepositoryError::Infrastructure)?;
    sqlx::query(
        "UPDATE flight_seats SET booking_status='AVAILABLE',booked_at=NULL
         WHERE id=ANY($1) AND booking_status='BOOKED'",
    )
    .bind(&seat_ids)
    .execute(&mut *tx)
    .await
    .map_err(CancellationRepositoryError::Infrastructure)?;
    if let Some(actor) = staff_actor {
        sqlx::query(
            "INSERT INTO booking_operations_audit(
                actor_staff_user_id,actor_email,ticket_id,booking_reference,cancellation_id,
                action,before_state,after_state,created_at
             ) VALUES($1,$2,$3,$4,$5,'STAFF_BOOKING_CANCELLED',
                jsonb_build_object('bookingStatus','CONFIRMED','ticketStatus','ISSUED'),
                jsonb_build_object('bookingStatus','CANCELLED','ticketStatus','CANCELLED','refundStatus','PENDING'),$6)",
        )
        .bind(actor.staff_user_id)
        .bind(&actor.email)
        .bind(ticket_id)
        .bind(&row.booking_reference)
        .bind(cancellation.id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(CancellationRepositoryError::Infrastructure)?;
    }
    let detail = if include_detail {
        super::booking_management::load_booking_detail(&mut tx, &row.booking_reference, now)
            .await
            .map_err(map_booking_detail_error)?
    } else {
        None
    };
    tx.commit()
        .await
        .map_err(CancellationRepositoryError::Infrastructure)?;
    Ok(CancellationOutcome {
        cancellation,
        detail,
    })
}

fn map_booking_detail_error(
    error: crate::domain::repositories::BookingManagementRepositoryError,
) -> CancellationRepositoryError {
    match error {
        crate::domain::repositories::BookingManagementRepositoryError::InvalidPagination => {
            CancellationRepositoryError::InconsistentState
        }
        crate::domain::repositories::BookingManagementRepositoryError::InconsistentState => {
            CancellationRepositoryError::InconsistentState
        }
        crate::domain::repositories::BookingManagementRepositoryError::Infrastructure(error) => {
            CancellationRepositoryError::Infrastructure(error)
        }
    }
}

fn code(f: &RefundFailure) -> &'static str {
    match f {
        RefundFailure::Transient(c) | RefundFailure::Permanent(c) => c,
    }
}
#[derive(FromRow)]
struct RefundJobRow {
    id: Uuid,
    payment_attempt_id: Uuid,
    refund_provider: String,
    provider_payment_id: Option<String>,
    provider_refund_id: Option<String>,
    refund_amount: i64,
    currency_code: String,
    attempt_count: i16,
    lease_token: Option<Uuid>,
}
impl RefundJobRow {
    fn domain(self) -> Option<RefundJob> {
        Some(RefundJob {
            id: self.id,
            payment_attempt_id: self.payment_attempt_id,
            provider: PaymentProvider::parse_database(&self.refund_provider)?,
            provider_payment_id: self.provider_payment_id?,
            provider_refund_id: self.provider_refund_id,
            amount: Money {
                amount: self.refund_amount,
                currency_code: self.currency_code,
            },
            attempt_count: self.attempt_count as u8,
            lease_token: self.lease_token?,
        })
    }
}
