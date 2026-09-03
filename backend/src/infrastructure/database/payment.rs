use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::{
    extras::Money,
    passengers::expected_passenger_slots,
    payment::{
        PaymentAttempt, PaymentAttemptTransition, PaymentContext, PaymentFailure, PaymentMethod,
        PaymentPricing, PaymentProvider, PaymentRepositoryCommand, PaymentStatus,
    },
    repositories::{PaymentRepository, PaymentRepositoryError, SeatHoldRepositoryError},
    review::{ReviewJourney, StopType},
    value_objects::PassengerCounts,
};

use super::{passengers::load_passengers, HoldRow, SqlxSeatHoldRepository};

#[async_trait]
impl PaymentRepository for SqlxSeatHoldRepository {
    async fn get_payment(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<PaymentContext, PaymentRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(PaymentRepositoryError::Infrastructure)?;
        let (hold_row, server_time) =
            locked_payment_hold(&mut transaction, hold_id, token_hash).await?;
        let has_success: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM payment_attempts WHERE seat_hold_id = $1 AND status = 'SUCCEEDED')",
        )
        .bind(hold_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(PaymentRepositoryError::Infrastructure)?;
        if hold_row.released_at.is_some() {
            return Err(PaymentRepositoryError::HoldReleased);
        }
        if hold_row.consumed_at.is_some() && !has_success {
            return Err(PaymentRepositoryError::HoldConsumed);
        }
        let attempts = load_attempts(&mut transaction, hold_id).await?;
        let has_protected_open_stripe_attempt = attempts.iter().any(|attempt| {
            attempt.provider == PaymentProvider::Stripe
                && attempt.payment_method == PaymentMethod::Card
                && matches!(
                    attempt.status,
                    PaymentStatus::Created
                        | PaymentStatus::Processing
                        | PaymentStatus::AwaitingPayment
                )
                && attempt.payment_finalization_deadline.is_some()
        });
        if !has_success {
            if hold_row.expires_at <= server_time && !has_protected_open_stripe_attempt {
                return Err(PaymentRepositoryError::HoldExpired);
            }
            ensure_ready(&mut transaction, &hold_row).await?;
        }
        let snapshot = load_snapshot(&mut transaction, hold_id).await?;
        let journey = load_journey(&mut transaction, hold_id).await?;
        let mut hold = Self::hold_entity(&mut transaction, hold_row)
            .await
            .map_err(map_hold_error)?;
        if has_success {
            hold.seats = load_finalized_seats(&mut transaction, hold_id).await?;
        }
        transaction
            .commit()
            .await
            .map_err(PaymentRepositoryError::Infrastructure)?;
        Ok(PaymentContext {
            hold,
            journey,
            pricing: PaymentPricing {
                currency_code: snapshot.currency_code.clone(),
                grand_total: Money {
                    amount: snapshot.amount,
                    currency_code: snapshot.currency_code,
                },
                priced_at: snapshot.priced_at,
            },
            methods: vec![PaymentMethod::Card, PaymentMethod::Bitcoin],
            attempts,
            ready_for_payment: !has_success,
        })
    }

    async fn get_payment_attempt(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        attempt_id: Uuid,
    ) -> Result<PaymentAttempt, PaymentRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(PaymentRepositoryError::Infrastructure)?;
        locked_payment_hold(&mut transaction, hold_id, token_hash).await?;
        let attempt = load_attempt_for_update(&mut transaction, hold_id, attempt_id).await?;
        transaction
            .commit()
            .await
            .map_err(PaymentRepositoryError::Infrastructure)?;
        Ok(attempt.into_domain())
    }

    async fn create_payment_attempt(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        command: PaymentRepositoryCommand,
    ) -> Result<PaymentAttempt, PaymentRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(PaymentRepositoryError::Infrastructure)?;
        let (hold, server_time) =
            locked_payment_hold(&mut transaction, hold_id, token_hash).await?;

        if let Some(existing) =
            load_by_request(&mut transaction, hold_id, command.request_id).await?
        {
            if existing.request_fingerprint.as_slice() != command.request_fingerprint {
                return Err(PaymentRepositoryError::IdempotencyKeyReused);
            }
            transaction
                .commit()
                .await
                .map_err(PaymentRepositoryError::Infrastructure)?;
            return Ok(existing.into_domain());
        }

        let succeeded: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM payment_attempts WHERE seat_hold_id = $1 AND status = 'SUCCEEDED')",
        )
        .bind(hold_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(PaymentRepositoryError::Infrastructure)?;
        if succeeded {
            return Err(PaymentRepositoryError::AlreadySucceeded);
        }
        if hold.consumed_at.is_some() {
            return Err(PaymentRepositoryError::HoldConsumed);
        }
        if hold.released_at.is_some() {
            return Err(PaymentRepositoryError::HoldReleased);
        }
        if hold.expires_at <= server_time {
            return Err(PaymentRepositoryError::HoldExpired);
        }
        ensure_ready(&mut transaction, &hold).await?;
        let open: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM payment_attempts WHERE seat_hold_id = $1 AND status IN ('CREATED', 'PROCESSING', 'AWAITING_PAYMENT'))",
        )
        .bind(hold_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(PaymentRepositoryError::Infrastructure)?;
        if open {
            return Err(PaymentRepositoryError::AttemptInProgress);
        }

        let snapshot = load_snapshot(&mut transaction, hold_id).await?;
        let payment_finalization_deadline = (command.provider == PaymentProvider::Stripe
            && command.method == PaymentMethod::Card)
            .then_some(hold.expires_at + chrono::Duration::minutes(5));
        let row = sqlx::query_as::<_, PaymentAttemptRow>(
            "INSERT INTO payment_attempts (
                seat_hold_id, request_id, request_fingerprint, provider, payment_method,
                status, amount, currency_code, review_priced_at, payment_finalization_deadline
             ) VALUES ($1, $2, $3, $4, $5, 'CREATED', $6, $7, $8, $9)
             RETURNING id, seat_hold_id, request_id, request_fingerprint, provider,
                payment_method, status, amount, currency_code, review_priced_at,
                provider_reference, failure_code, failure_message, created_at, updated_at,
                succeeded_at, payment_finalization_deadline",
        )
        .bind(hold_id)
        .bind(command.request_id)
        .bind(command.request_fingerprint.as_slice())
        .bind(command.provider.as_str())
        .bind(command.method.as_str())
        .bind(snapshot.amount)
        .bind(&snapshot.currency_code)
        .bind(snapshot.priced_at)
        .bind(payment_finalization_deadline)
        .fetch_one(&mut *transaction)
        .await
        .map_err(map_insert_error)?;
        transaction
            .commit()
            .await
            .map_err(PaymentRepositoryError::Infrastructure)?;
        Ok(row.into_domain())
    }

    async fn transition_payment_attempt(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        attempt_id: Uuid,
        transition: PaymentAttemptTransition,
    ) -> Result<PaymentAttempt, PaymentRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(PaymentRepositoryError::Infrastructure)?;
        let (hold, server_time) =
            locked_payment_hold(&mut transaction, hold_id, token_hash).await?;
        let current = load_attempt_for_update(&mut transaction, hold_id, attempt_id).await?;
        let current_status = PaymentStatus::parse_database(&current.status)
            .expect("database payment status constraint is valid");
        current_status
            .transition(transition.status)
            .map_err(|_| PaymentRepositoryError::InvalidTransition)?;

        if hold.expires_at <= server_time
            && !current.has_protected_stripe_card_finalization()
            && !matches!(
                transition.status,
                PaymentStatus::Failed | PaymentStatus::Cancelled
            )
        {
            sqlx::query(
                "UPDATE payment_attempts SET status = 'FAILED', failure_code = 'HOLD_EXPIRED',
                    failure_message = 'The seat hold is no longer active.', updated_at = NOW()
                 WHERE seat_hold_id = $1 AND id = $2",
            )
            .bind(hold_id)
            .bind(attempt_id)
            .execute(&mut *transaction)
            .await
            .map_err(PaymentRepositoryError::Infrastructure)?;
            transaction
                .commit()
                .await
                .map_err(PaymentRepositoryError::Infrastructure)?;
            return Err(PaymentRepositoryError::HoldExpired);
        }

        if transition.status == PaymentStatus::Succeeded {
            let lifecycle_error = if hold.consumed_at.is_some() {
                Some((PaymentRepositoryError::HoldConsumed, "HOLD_CONSUMED"))
            } else if hold.released_at.is_some() {
                Some((PaymentRepositoryError::HoldReleased, "HOLD_RELEASED"))
            } else {
                None
            };
            if let Some((error, code)) = lifecycle_error {
                sqlx::query(
                    "UPDATE payment_attempts SET status = 'FAILED', failure_code = $3,
                        failure_message = 'The seat hold is no longer active.', updated_at = NOW()
                     WHERE seat_hold_id = $1 AND id = $2",
                )
                .bind(hold_id)
                .bind(attempt_id)
                .bind(code)
                .execute(&mut *transaction)
                .await
                .map_err(PaymentRepositoryError::Infrastructure)?;
                transaction
                    .commit()
                    .await
                    .map_err(PaymentRepositoryError::Infrastructure)?;
                return Err(error);
            }
        } else if hold.consumed_at.is_some() {
            return Err(PaymentRepositoryError::HoldConsumed);
        } else if hold.released_at.is_some() {
            return Err(PaymentRepositoryError::HoldReleased);
        }

        if transition.status == PaymentStatus::Succeeded {
            finalize_seats_and_hold(
                &mut transaction,
                &hold,
                attempt_id,
                current.amount,
                &current.currency_code,
                current.review_priced_at,
            )
            .await?;
        }

        let failure_code = transition
            .failure
            .as_ref()
            .map(|failure| failure.code.as_str());
        let failure_message = transition
            .failure
            .as_ref()
            .map(|failure| failure.message.as_str());
        let row = sqlx::query_as::<_, PaymentAttemptRow>(
            "UPDATE payment_attempts SET
                status = $3,
                provider_reference = COALESCE($4, provider_reference),
                failure_code = $5,
                failure_message = $6,
                succeeded_at = CASE WHEN $3 = 'SUCCEEDED' THEN NOW() ELSE NULL END,
                updated_at = NOW()
             WHERE seat_hold_id = $1 AND id = $2
             RETURNING id, seat_hold_id, request_id, request_fingerprint, provider,
                payment_method, status, amount, currency_code, review_priced_at,
                provider_reference, failure_code, failure_message, created_at, updated_at,
                succeeded_at, payment_finalization_deadline",
        )
        .bind(hold_id)
        .bind(attempt_id)
        .bind(transition.status.as_str())
        .bind(&transition.provider_reference)
        .bind(failure_code)
        .bind(failure_message)
        .fetch_one(&mut *transaction)
        .await
        .map_err(PaymentRepositoryError::Infrastructure)?;
        transaction
            .commit()
            .await
            .map_err(PaymentRepositoryError::Infrastructure)?;
        Ok(row.into_domain())
    }

    async fn process_stripe_webhook(
        &self,
        command: crate::domain::payment::ProcessStripeWebhookCommand,
    ) -> Result<crate::domain::payment::StripeWebhookResult, PaymentRepositoryError> {
        if command.event_type != "payment_intent.succeeded"
            && command.event_type != "payment_intent.payment_failed"
        {
            return Ok(crate::domain::payment::StripeWebhookResult::Ignored);
        }

        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(PaymentRepositoryError::Infrastructure)?;

        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
            .bind(&command.event_id)
            .execute(&mut *transaction)
            .await
            .map_err(PaymentRepositoryError::Infrastructure)?;

        let already_processed: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM stripe_webhook_events WHERE stripe_event_id = $1)",
        )
        .bind(&command.event_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(PaymentRepositoryError::Infrastructure)?;

        if already_processed {
            transaction
                .commit()
                .await
                .map_err(PaymentRepositoryError::Infrastructure)?;
            return Ok(crate::domain::payment::StripeWebhookResult::AlreadyProcessed);
        }

        let attempt_row = sqlx::query_as::<_, PaymentAttemptRow>(
            "SELECT id, seat_hold_id, request_id, request_fingerprint, provider,
                payment_method, status, amount, currency_code, review_priced_at,
                provider_reference, failure_code, failure_message, created_at, updated_at,
                succeeded_at, payment_finalization_deadline
             FROM payment_attempts
             WHERE provider = 'STRIPE' AND provider_reference = $1
             FOR UPDATE",
        )
        .bind(&command.payment_intent_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(PaymentRepositoryError::Infrastructure)?;

        let Some(current) = attempt_row else {
            return Err(PaymentRepositoryError::AttemptNotFound);
        };

        if current.provider != "STRIPE" || current.payment_method != "CARD" {
            return Err(PaymentRepositoryError::InvalidRequest);
        }

        let (hold, _server_time) =
            locked_payment_hold_by_id(&mut transaction, current.seat_hold_id).await?;

        match command.event_type.as_str() {
            "payment_intent.succeeded" => {
                if current.status == "SUCCEEDED" {
                    sqlx::query(
                        "INSERT INTO stripe_webhook_events (stripe_event_id, event_type, payment_intent_id, processed_at)
                         VALUES ($1, $2, $3, NOW())
                         ON CONFLICT (stripe_event_id) DO NOTHING",
                    )
                    .bind(&command.event_id)
                    .bind(&command.event_type)
                    .bind(&command.payment_intent_id)
                    .execute(&mut *transaction)
                    .await
                    .map_err(PaymentRepositoryError::Infrastructure)?;

                    transaction
                        .commit()
                        .await
                        .map_err(PaymentRepositoryError::Infrastructure)?;
                    return Ok(crate::domain::payment::StripeWebhookResult::Processed);
                }

                let expected_stripe =
                    crate::infrastructure::payment::stripe::StripeAmount::from_xfly_money(&Money {
                        amount: current.amount,
                        currency_code: current.currency_code.clone(),
                    })
                    .map_err(|_| PaymentRepositoryError::InvalidRequest)?;

                let amount_matches = command
                    .amount
                    .is_some_and(|amount| amount == expected_stripe.amount());
                let currency_matches = command
                    .currency
                    .as_deref()
                    .is_some_and(|curr| curr.eq_ignore_ascii_case(expected_stripe.currency()));

                if !amount_matches || !currency_matches {
                    return Err(PaymentRepositoryError::AmountMismatch);
                }

                let current_status = PaymentStatus::parse_database(&current.status)
                    .expect("database payment status constraint is valid");
                current_status
                    .transition(PaymentStatus::Succeeded)
                    .map_err(|_| PaymentRepositoryError::InvalidTransition)?;

                let lifecycle_error = if hold.consumed_at.is_some() {
                    Some((PaymentRepositoryError::HoldConsumed, "HOLD_CONSUMED"))
                } else if hold.released_at.is_some() {
                    Some((PaymentRepositoryError::HoldReleased, "HOLD_RELEASED"))
                } else {
                    None
                };
                if let Some((error, code)) = lifecycle_error {
                    sqlx::query(
                        "UPDATE payment_attempts SET status = 'FAILED', failure_code = $3,
                            failure_message = 'The seat hold is no longer active.', updated_at = NOW()
                         WHERE seat_hold_id = $1 AND id = $2",
                    )
                    .bind(hold.id)
                    .bind(current.id)
                    .bind(code)
                    .execute(&mut *transaction)
                    .await
                    .map_err(PaymentRepositoryError::Infrastructure)?;
                    transaction
                        .commit()
                        .await
                        .map_err(PaymentRepositoryError::Infrastructure)?;
                    return Err(error);
                }

                finalize_seats_and_hold(
                    &mut transaction,
                    &hold,
                    current.id,
                    current.amount,
                    &current.currency_code,
                    current.review_priced_at,
                )
                .await?;

                sqlx::query(
                    "UPDATE payment_attempts SET
                        status = 'SUCCEEDED',
                        succeeded_at = NOW(),
                        updated_at = NOW()
                     WHERE id = $1",
                )
                .bind(current.id)
                .execute(&mut *transaction)
                .await
                .map_err(PaymentRepositoryError::Infrastructure)?;

                sqlx::query(
                    "INSERT INTO stripe_webhook_events (stripe_event_id, event_type, payment_intent_id, processed_at)
                     VALUES ($1, $2, $3, NOW())",
                )
                .bind(&command.event_id)
                .bind(&command.event_type)
                .bind(&command.payment_intent_id)
                .execute(&mut *transaction)
                .await
                .map_err(PaymentRepositoryError::Infrastructure)?;

                transaction
                    .commit()
                    .await
                    .map_err(PaymentRepositoryError::Infrastructure)?;
                Ok(crate::domain::payment::StripeWebhookResult::Processed)
            }
            "payment_intent.payment_failed" => {
                if current.status == "FAILED" || current.status == "CANCELLED" {
                    sqlx::query(
                        "INSERT INTO stripe_webhook_events (stripe_event_id, event_type, payment_intent_id, processed_at)
                         VALUES ($1, $2, $3, NOW())
                         ON CONFLICT (stripe_event_id) DO NOTHING",
                    )
                    .bind(&command.event_id)
                    .bind(&command.event_type)
                    .bind(&command.payment_intent_id)
                    .execute(&mut *transaction)
                    .await
                    .map_err(PaymentRepositoryError::Infrastructure)?;

                    transaction
                        .commit()
                        .await
                        .map_err(PaymentRepositoryError::Infrastructure)?;
                    return Ok(crate::domain::payment::StripeWebhookResult::Processed);
                }

                if let Some(cmd_amount) = command.amount {
                    if let Ok(expected_stripe) =
                        crate::infrastructure::payment::stripe::StripeAmount::from_xfly_money(
                            &Money {
                                amount: current.amount,
                                currency_code: current.currency_code.clone(),
                            },
                        )
                    {
                        if cmd_amount != expected_stripe.amount() {
                            return Err(PaymentRepositoryError::AmountMismatch);
                        }
                    }
                }
                if let Some(ref cmd_currency) = command.currency {
                    if !cmd_currency.eq_ignore_ascii_case("thb") {
                        return Err(PaymentRepositoryError::AmountMismatch);
                    }
                }

                let current_status = PaymentStatus::parse_database(&current.status)
                    .expect("database payment status constraint is valid");
                current_status
                    .transition(PaymentStatus::Failed)
                    .map_err(|_| PaymentRepositoryError::InvalidTransition)?;

                let failure_code = command
                    .failure_code
                    .unwrap_or_else(|| "PROCESSING_ERROR".to_owned());
                let failure_message = command
                    .failure_message
                    .unwrap_or_else(|| "The payment could not be processed.".to_owned());

                sqlx::query(
                    "UPDATE payment_attempts SET
                        status = 'FAILED',
                        failure_code = $2,
                        failure_message = $3,
                        updated_at = NOW()
                     WHERE id = $1",
                )
                .bind(current.id)
                .bind(&failure_code)
                .bind(&failure_message)
                .execute(&mut *transaction)
                .await
                .map_err(PaymentRepositoryError::Infrastructure)?;

                sqlx::query(
                    "INSERT INTO stripe_webhook_events (stripe_event_id, event_type, payment_intent_id, processed_at)
                     VALUES ($1, $2, $3, NOW())",
                )
                .bind(&command.event_id)
                .bind(&command.event_type)
                .bind(&command.payment_intent_id)
                .execute(&mut *transaction)
                .await
                .map_err(PaymentRepositoryError::Infrastructure)?;

                transaction
                    .commit()
                    .await
                    .map_err(PaymentRepositoryError::Infrastructure)?;
                Ok(crate::domain::payment::StripeWebhookResult::Processed)
            }
            _ => {
                transaction
                    .commit()
                    .await
                    .map_err(PaymentRepositoryError::Infrastructure)?;
                Ok(crate::domain::payment::StripeWebhookResult::Ignored)
            }
        }
    }
}

async fn load_attempts(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
) -> Result<Vec<PaymentAttempt>, PaymentRepositoryError> {
    let rows = sqlx::query_as::<_, PaymentAttemptRow>(
        "SELECT id, seat_hold_id, request_id, request_fingerprint, provider,
            payment_method, status, amount, currency_code, review_priced_at,
            provider_reference, failure_code, failure_message, created_at, updated_at,
            succeeded_at, payment_finalization_deadline
         FROM payment_attempts WHERE seat_hold_id = $1 ORDER BY created_at DESC",
    )
    .bind(hold_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(PaymentRepositoryError::Infrastructure)?;
    Ok(rows
        .into_iter()
        .map(PaymentAttemptRow::into_domain)
        .collect())
}

async fn load_finalized_seats(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
) -> Result<Vec<crate::domain::value_objects::SeatNumber>, PaymentRepositoryError> {
    let values: Vec<String> = sqlx::query_scalar(
        "SELECT seat.seat_number
         FROM payment_attempts AS attempt
         JOIN payment_attempt_seats AS finalized ON finalized.payment_attempt_id = attempt.id
         JOIN flight_seats AS seat ON seat.id = finalized.flight_seat_id
         WHERE attempt.seat_hold_id = $1 AND attempt.status = 'SUCCEEDED'
         ORDER BY seat.seat_number",
    )
    .bind(hold_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(PaymentRepositoryError::Infrastructure)?;
    Ok(values
        .iter()
        .map(|value| {
            crate::domain::value_objects::SeatNumber::parse(value)
                .expect("database seat constraint is valid")
        })
        .collect())
}

async fn load_journey(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
) -> Result<ReviewJourney, PaymentRepositoryError> {
    let row = sqlx::query_as::<_, PaymentJourneyRow>(
        "SELECT service.flight_number, service.origin_code, service.destination_code,
            service.aircraft_code, service.departure_time, service.arrival_time,
            service.arrival_day_offset, service.duration_minutes, service.stops
         FROM seat_holds AS hold
         JOIN flight_instances AS instance ON instance.id = hold.flight_instance_id
         JOIN flight_services AS service ON service.id = instance.flight_service_id
         WHERE hold.id = $1",
    )
    .bind(hold_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(PaymentRepositoryError::Infrastructure)?;
    Ok(ReviewJourney {
        flight_number: row.flight_number,
        origin_code: row.origin_code,
        destination_code: row.destination_code,
        aircraft_code: row.aircraft_code,
        departure_time: row
            .departure_time
            .ok_or(PaymentRepositoryError::ReviewNotReady)?
            .format("%H:%M")
            .to_string(),
        arrival_time: row
            .arrival_time
            .ok_or(PaymentRepositoryError::ReviewNotReady)?
            .format("%H:%M")
            .to_string(),
        arrival_day_offset: row
            .arrival_day_offset
            .and_then(|value| u8::try_from(value).ok())
            .ok_or(PaymentRepositoryError::ReviewNotReady)?,
        duration_minutes: row
            .duration_minutes
            .and_then(|value| u16::try_from(value).ok())
            .ok_or(PaymentRepositoryError::ReviewNotReady)?,
        stops: match row.stops.as_deref() {
            Some("DIRECT") => StopType::Direct,
            Some("ONE_STOP") => StopType::OneStop,
            _ => return Err(PaymentRepositoryError::ReviewNotReady),
        },
    })
}

async fn locked_payment_hold_by_id(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
) -> Result<(HoldRow, DateTime<Utc>), PaymentRepositoryError> {
    let hold = sqlx::query_as::<_, HoldRow>(
        "SELECT hold.id, service.public_id AS flight_id, instance.departure_date,
            hold.flight_instance_id, hold.cabin, hold.adults, hold.children, hold.infants,
            hold.access_token_hash, hold.expires_at, hold.released_at, hold.consumed_at
         FROM seat_holds AS hold
         JOIN flight_instances AS instance ON instance.id = hold.flight_instance_id
         JOIN flight_services AS service ON service.id = instance.flight_service_id
         WHERE hold.id = $1 FOR UPDATE OF hold",
    )
    .bind(hold_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(PaymentRepositoryError::Infrastructure)?
    .ok_or(PaymentRepositoryError::HoldNotFound)?;
    let server_time: DateTime<Utc> = sqlx::query_scalar("SELECT NOW()")
        .fetch_one(&mut **transaction)
        .await
        .map_err(PaymentRepositoryError::Infrastructure)?;
    Ok((hold, server_time))
}

async fn locked_payment_hold(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
    token_hash: [u8; 32],
) -> Result<(HoldRow, DateTime<Utc>), PaymentRepositoryError> {
    let (hold, server_time) = locked_payment_hold_by_id(transaction, hold_id).await?;
    if hold.access_token_hash.as_slice() != token_hash {
        return Err(PaymentRepositoryError::Unauthorized);
    }
    Ok((hold, server_time))
}

async fn finalize_seats_and_hold(
    transaction: &mut Transaction<'_, Postgres>,
    hold: &HoldRow,
    current_attempt_id: Uuid,
    expected_amount: i64,
    expected_currency_code: &str,
    expected_priced_at: DateTime<Utc>,
) -> Result<(), PaymentRepositoryError> {
    ensure_ready(transaction, hold).await?;
    let snapshot = load_snapshot(transaction, hold.id).await?;
    if snapshot.amount != expected_amount
        || snapshot.currency_code != expected_currency_code
        || snapshot.priced_at != expected_priced_at
    {
        return Err(PaymentRepositoryError::ReviewNotReady);
    }
    let prior_success: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM payment_attempts WHERE seat_hold_id = $1 AND status = 'SUCCEEDED' AND id <> $2)",
    )
    .bind(hold.id)
    .bind(current_attempt_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(PaymentRepositoryError::Infrastructure)?;
    if prior_success {
        return Err(PaymentRepositoryError::AlreadySucceeded);
    }

    let required = required_seats(hold);
    let linked = sqlx::query(
        "INSERT INTO payment_attempt_seats (payment_attempt_id, flight_seat_id)
         SELECT $2, id FROM flight_seats
         WHERE hold_id = $1 AND sellable = TRUE AND booking_status = 'AVAILABLE'",
    )
    .bind(hold.id)
    .bind(current_attempt_id)
    .execute(&mut **transaction)
    .await
    .map_err(PaymentRepositoryError::Infrastructure)?;
    if linked.rows_affected() != required as u64 {
        return Err(PaymentRepositoryError::SeatsNotReady);
    }
    let booked = sqlx::query(
        "UPDATE flight_seats
         SET booking_status = 'BOOKED', booked_at = NOW(), hold_id = NULL, updated_at = NOW()
         WHERE hold_id = $1 AND sellable = TRUE AND booking_status = 'AVAILABLE'",
    )
    .bind(hold.id)
    .execute(&mut **transaction)
    .await
    .map_err(PaymentRepositoryError::Infrastructure)?;
    if booked.rows_affected() != required as u64 {
        return Err(PaymentRepositoryError::SeatsNotReady);
    }
    sqlx::query("UPDATE seat_holds SET consumed_at = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(hold.id)
        .execute(&mut **transaction)
        .await
        .map_err(PaymentRepositoryError::Infrastructure)?;
    Ok(())
}

async fn ensure_ready(
    transaction: &mut Transaction<'_, Postgres>,
    hold: &HoldRow,
) -> Result<(), PaymentRepositoryError> {
    let counts = PassengerCounts::new(hold.adults as u8, hold.children as u8, hold.infants as u8)
        .expect("database passenger constraints are valid");
    let valid_seats: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM flight_seats
         WHERE hold_id = $1 AND sellable = TRUE AND booking_status = 'AVAILABLE'",
    )
    .bind(hold.id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(PaymentRepositoryError::Infrastructure)?;
    if valid_seats as usize != counts.required_seats() {
        return Err(PaymentRepositoryError::SeatsNotReady);
    }

    let passengers = load_passengers(transaction, hold.id)
        .await
        .map_err(|error| match error {
            crate::domain::repositories::PassengerRepositoryError::Infrastructure(error) => {
                PaymentRepositoryError::Infrastructure(error)
            }
            _ => PaymentRepositoryError::PassengersNotReady,
        })?;
    let expected = expected_passenger_slots(counts);
    if passengers.len() != expected.len()
        || passengers.iter().zip(expected).any(|(actual, expected)| {
            actual.ordinal != expected.ordinal || actual.passenger_type != expected.passenger_type
        })
    {
        return Err(PaymentRepositoryError::PassengersNotReady);
    }
    let extras_saved: bool =
        sqlx::query_scalar("SELECT extras_saved_at IS NOT NULL FROM seat_holds WHERE id = $1")
            .bind(hold.id)
            .fetch_one(&mut **transaction)
            .await
            .map_err(PaymentRepositoryError::Infrastructure)?;
    if !extras_saved {
        return Err(PaymentRepositoryError::ExtrasNotReady);
    }
    load_snapshot(transaction, hold.id).await?;
    Ok(())
}

fn required_seats(hold: &HoldRow) -> usize {
    usize::from(hold.adults as u16 + hold.children as u16)
}

async fn load_snapshot(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
) -> Result<PaymentSnapshotRow, PaymentRepositoryError> {
    sqlx::query_as::<_, PaymentSnapshotRow>(
        "SELECT pricing.grand_total_amount AS amount, pricing.currency_code, pricing.priced_at
         FROM hold_review_pricing AS pricing
         JOIN seat_holds AS hold ON hold.id = pricing.seat_hold_id
         WHERE pricing.seat_hold_id = $1
           AND pricing.source_extras_saved_at = hold.extras_saved_at",
    )
    .bind(hold_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(PaymentRepositoryError::Infrastructure)?
    .ok_or(PaymentRepositoryError::ReviewNotReady)
}

async fn load_by_request(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
    request_id: Uuid,
) -> Result<Option<PaymentAttemptRow>, PaymentRepositoryError> {
    sqlx::query_as::<_, PaymentAttemptRow>(
        "SELECT id, seat_hold_id, request_id, request_fingerprint, provider,
            payment_method, status, amount, currency_code, review_priced_at,
            provider_reference, failure_code, failure_message, created_at, updated_at,
            succeeded_at, payment_finalization_deadline
         FROM payment_attempts WHERE seat_hold_id = $1 AND request_id = $2",
    )
    .bind(hold_id)
    .bind(request_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(PaymentRepositoryError::Infrastructure)
}

async fn load_attempt_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
    attempt_id: Uuid,
) -> Result<PaymentAttemptRow, PaymentRepositoryError> {
    sqlx::query_as::<_, PaymentAttemptRow>(
        "SELECT id, seat_hold_id, request_id, request_fingerprint, provider,
            payment_method, status, amount, currency_code, review_priced_at,
            provider_reference, failure_code, failure_message, created_at, updated_at,
            succeeded_at, payment_finalization_deadline
         FROM payment_attempts WHERE seat_hold_id = $1 AND id = $2 FOR UPDATE",
    )
    .bind(hold_id)
    .bind(attempt_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(PaymentRepositoryError::Infrastructure)?
    .ok_or(PaymentRepositoryError::AttemptNotFound)
}

fn map_hold_error(error: SeatHoldRepositoryError) -> PaymentRepositoryError {
    match error {
        SeatHoldRepositoryError::HoldNotFound => PaymentRepositoryError::HoldNotFound,
        SeatHoldRepositoryError::Unauthorized => PaymentRepositoryError::Unauthorized,
        SeatHoldRepositoryError::HoldExpired => PaymentRepositoryError::HoldExpired,
        SeatHoldRepositoryError::HoldReleased => PaymentRepositoryError::HoldReleased,
        SeatHoldRepositoryError::HoldConsumed => PaymentRepositoryError::HoldConsumed,
        SeatHoldRepositoryError::Infrastructure(error) => {
            PaymentRepositoryError::Infrastructure(error)
        }
        _ => PaymentRepositoryError::SeatsNotReady,
    }
}

fn map_insert_error(error: sqlx::Error) -> PaymentRepositoryError {
    if let sqlx::Error::Database(database) = &error {
        if database.constraint() == Some("idx_payment_attempts_one_open_per_hold") {
            return PaymentRepositoryError::AttemptInProgress;
        }
        if database.constraint() == Some("idx_payment_attempts_one_success_per_hold") {
            return PaymentRepositoryError::AlreadySucceeded;
        }
    }
    PaymentRepositoryError::Infrastructure(error)
}

#[derive(FromRow)]
struct PaymentSnapshotRow {
    amount: i64,
    currency_code: String,
    priced_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct PaymentJourneyRow {
    flight_number: String,
    origin_code: String,
    destination_code: String,
    aircraft_code: String,
    departure_time: Option<chrono::NaiveTime>,
    arrival_time: Option<chrono::NaiveTime>,
    arrival_day_offset: Option<i16>,
    duration_minutes: Option<i16>,
    stops: Option<String>,
}

#[derive(FromRow)]
struct PaymentAttemptRow {
    id: Uuid,
    #[allow(dead_code)]
    seat_hold_id: Uuid,
    #[allow(dead_code)]
    request_id: Uuid,
    request_fingerprint: Vec<u8>,
    provider: String,
    payment_method: String,
    status: String,
    amount: i64,
    currency_code: String,
    review_priced_at: DateTime<Utc>,
    provider_reference: Option<String>,
    failure_code: Option<String>,
    failure_message: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    succeeded_at: Option<DateTime<Utc>>,
    payment_finalization_deadline: Option<DateTime<Utc>>,
}

impl PaymentAttemptRow {
    fn has_protected_stripe_card_finalization(&self) -> bool {
        self.provider == "STRIPE"
            && self.payment_method == "CARD"
            && matches!(
                self.status.as_str(),
                "CREATED" | "PROCESSING" | "AWAITING_PAYMENT"
            )
            && self.payment_finalization_deadline.is_some()
    }

    fn into_domain(self) -> PaymentAttempt {
        PaymentAttempt {
            id: self.id,
            provider: PaymentProvider::parse_database(&self.provider)
                .expect("database payment provider constraint is valid"),
            payment_method: PaymentMethod::parse_database(&self.payment_method)
                .expect("database payment method constraint is valid"),
            status: PaymentStatus::parse_database(&self.status)
                .expect("database payment status constraint is valid"),
            amount: Money {
                amount: self.amount,
                currency_code: self.currency_code,
            },
            provider_reference: self.provider_reference,
            failure: self
                .failure_code
                .zip(self.failure_message)
                .map(|(code, message)| PaymentFailure { code, message }),
            created_at: self.created_at,
            updated_at: self.updated_at,
            succeeded_at: self.succeeded_at,
            payment_finalization_deadline: self.payment_finalization_deadline,
            client_payment_session: None,
            demo_bitcoin_invoice: None,
        }
    }
}
