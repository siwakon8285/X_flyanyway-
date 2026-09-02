use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::{
    extras::{
        catalog_for_cabin, price_extra_selections, total_for_selections, ExtraCategory,
        ExtraContext, ExtraSelectionInput, Money, PricedExtraSelection,
    },
    passengers::{expected_passenger_slots, PassengerSlot, PassengerType},
    repositories::{ExtraRepository, ExtraRepositoryError, SeatHoldRepositoryError},
    value_objects::PassengerCounts,
};

use super::{HoldRow, SqlxSeatHoldRepository};

#[async_trait]
impl ExtraRepository for SqlxSeatHoldRepository {
    async fn get_extras(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<ExtraContext, ExtraRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(ExtraRepositoryError::Infrastructure)?;
        let hold_row = Self::locked_hold(&mut transaction, hold_id, token_hash)
            .await
            .map_err(map_hold_error)?;
        let context = load_context(&mut transaction, hold_row).await?;
        transaction
            .commit()
            .await
            .map_err(ExtraRepositoryError::Infrastructure)?;
        Ok(context)
    }

    async fn save_extras(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
        inputs: Vec<ExtraSelectionInput>,
    ) -> Result<ExtraContext, ExtraRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(ExtraRepositoryError::Infrastructure)?;
        let hold_row = Self::locked_hold(&mut transaction, hold_id, token_hash)
            .await
            .map_err(map_hold_error)?;
        let passengers = load_ready_passengers(&mut transaction, &hold_row).await?;
        let priced = price_extra_selections(&inputs, &passengers)?;

        sqlx::query("DELETE FROM hold_extras WHERE seat_hold_id = $1")
            .bind(hold_id)
            .execute(&mut *transaction)
            .await
            .map_err(ExtraRepositoryError::Infrastructure)?;
        for selection in &priced {
            sqlx::query(
                "INSERT INTO hold_extras (
                    seat_hold_id, passenger_ordinal, product_code, category, quantity,
                    unit_price_amount, currency_code
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(hold_id)
            .bind(i16::from(selection.passenger_ordinal))
            .bind(&selection.product_code)
            .bind(selection.category.as_str())
            .bind(i16::from(selection.quantity))
            .bind(selection.unit_price.amount)
            .bind(&selection.unit_price.currency_code)
            .execute(&mut *transaction)
            .await
            .map_err(ExtraRepositoryError::Infrastructure)?;
        }
        // Workflow marker only: this records explicit review/save of Travel Extras.
        // It is not evidence of payment, booking confirmation, or ticket issuance.
        sqlx::query(
            "UPDATE seat_holds SET extras_saved_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(hold_id)
        .execute(&mut *transaction)
        .await
        .map_err(ExtraRepositoryError::Infrastructure)?;
        sqlx::query("DELETE FROM hold_review_pricing WHERE seat_hold_id = $1")
            .bind(hold_id)
            .execute(&mut *transaction)
            .await
            .map_err(ExtraRepositoryError::Infrastructure)?;

        let context = load_context_with_passengers(&mut transaction, hold_row, passengers).await?;
        transaction
            .commit()
            .await
            .map_err(ExtraRepositoryError::Infrastructure)?;
        Ok(context)
    }
}

async fn load_context(
    transaction: &mut Transaction<'_, Postgres>,
    hold_row: HoldRow,
) -> Result<ExtraContext, ExtraRepositoryError> {
    let passengers = load_ready_passengers(transaction, &hold_row).await?;
    load_context_with_passengers(transaction, hold_row, passengers).await
}

async fn load_context_with_passengers(
    transaction: &mut Transaction<'_, Postgres>,
    hold_row: HoldRow,
    passengers: Vec<PassengerSlot>,
) -> Result<ExtraContext, ExtraRepositoryError> {
    let hold_id = hold_row.id;
    let saved_at: Option<DateTime<Utc>> =
        sqlx::query_scalar("SELECT extras_saved_at FROM seat_holds WHERE id = $1")
            .bind(hold_id)
            .fetch_one(&mut **transaction)
            .await
            .map_err(ExtraRepositoryError::Infrastructure)?;
    let selections = load_selections(transaction, hold_id).await?;
    let hold = SqlxSeatHoldRepository::hold_entity(transaction, hold_row)
        .await
        .map_err(map_hold_error)?;
    let catalog = catalog_for_cabin(hold.cabin);
    let total = total_for_selections(&selections);

    Ok(ExtraContext {
        hold,
        passengers,
        catalog,
        selections,
        total,
        ready_to_continue: saved_at.is_some(),
        saved_at,
    })
}

async fn load_ready_passengers(
    transaction: &mut Transaction<'_, Postgres>,
    hold_row: &HoldRow,
) -> Result<Vec<PassengerSlot>, ExtraRepositoryError> {
    let counts = PassengerCounts::new(
        hold_row.adults as u8,
        hold_row.children as u8,
        hold_row.infants as u8,
    )
    .expect("database passenger constraints are valid");
    let held_seat_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM flight_seats WHERE hold_id = $1")
            .bind(hold_row.id)
            .fetch_one(&mut **transaction)
            .await
            .map_err(ExtraRepositoryError::Infrastructure)?;
    if held_seat_count as usize != counts.required_seats() {
        return Err(ExtraRepositoryError::SeatCountMismatch);
    }

    let rows = sqlx::query_as::<_, PassengerSlotRow>(
        "SELECT ordinal, passenger_type
         FROM hold_passengers
         WHERE seat_hold_id = $1
         ORDER BY ordinal",
    )
    .bind(hold_row.id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(ExtraRepositoryError::Infrastructure)?;
    let passengers: Vec<PassengerSlot> = rows
        .into_iter()
        .map(|row| PassengerSlot {
            ordinal: row.ordinal as u8,
            passenger_type: PassengerType::parse_database(&row.passenger_type)
                .expect("database passenger type constraint is valid"),
        })
        .collect();
    let expected = expected_passenger_slots(counts);
    if passengers.len() != expected.len()
        || passengers.iter().zip(&expected).any(|(actual, expected)| {
            actual.ordinal != expected.ordinal || actual.passenger_type != expected.passenger_type
        })
    {
        return Err(ExtraRepositoryError::PassengersNotReady);
    }
    Ok(passengers)
}

pub(super) async fn load_selections(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
) -> Result<Vec<PricedExtraSelection>, ExtraRepositoryError> {
    let rows = sqlx::query_as::<_, ExtraRow>(
        "SELECT passenger_ordinal, product_code, category, quantity,
                unit_price_amount, currency_code
         FROM hold_extras
         WHERE seat_hold_id = $1
         ORDER BY passenger_ordinal, category, product_code",
    )
    .bind(hold_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(ExtraRepositoryError::Infrastructure)?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let unit_price = Money {
                amount: row.unit_price_amount,
                currency_code: row.currency_code,
            };
            PricedExtraSelection {
                passenger_ordinal: row.passenger_ordinal as u8,
                product_code: row.product_code,
                category: ExtraCategory::parse_database(&row.category)
                    .expect("database extra category constraint is valid"),
                quantity: row.quantity as u8,
                line_total: Money {
                    amount: unit_price.amount * i64::from(row.quantity),
                    currency_code: unit_price.currency_code.clone(),
                },
                unit_price,
            }
        })
        .collect())
}

fn map_hold_error(error: SeatHoldRepositoryError) -> ExtraRepositoryError {
    match error {
        SeatHoldRepositoryError::HoldNotFound => ExtraRepositoryError::HoldNotFound,
        SeatHoldRepositoryError::Unauthorized => ExtraRepositoryError::Unauthorized,
        SeatHoldRepositoryError::HoldExpired => ExtraRepositoryError::HoldExpired,
        SeatHoldRepositoryError::HoldReleased => ExtraRepositoryError::HoldReleased,
        SeatHoldRepositoryError::HoldConsumed => ExtraRepositoryError::HoldConsumed,
        SeatHoldRepositoryError::SeatCountMismatch => ExtraRepositoryError::SeatCountMismatch,
        SeatHoldRepositoryError::Infrastructure(error) => {
            ExtraRepositoryError::Infrastructure(error)
        }
        SeatHoldRepositoryError::FlightNotFound
        | SeatHoldRepositoryError::CabinUnavailable
        | SeatHoldRepositoryError::SeatNotFound(_)
        | SeatHoldRepositoryError::SeatConflict(_) => ExtraRepositoryError::SeatCountMismatch,
    }
}

#[derive(FromRow)]
struct PassengerSlotRow {
    ordinal: i16,
    passenger_type: String,
}

#[derive(FromRow)]
struct ExtraRow {
    passenger_ordinal: i16,
    product_code: String,
    category: String,
    quantity: i16,
    unit_price_amount: i64,
    currency_code: String,
}
