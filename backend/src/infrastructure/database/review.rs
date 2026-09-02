use async_trait::async_trait;
use chrono::{DateTime, NaiveTime, Utc};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::{
    extras::{catalog_for_cabin, Money, PricedExtraSelection},
    passengers::{expected_passenger_slots, Passenger, PassengerType},
    pricing::{
        calculate_review_pricing, CalculatedReviewPricing, DEMO_AIRPORT_FEE_AMOUNT,
        DEMO_BOOKING_FEE_AMOUNT, DEMO_PASSENGER_TAX_AMOUNT, REVIEW_CURRENCY,
    },
    repositories::{
        ExtraRepositoryError, PassengerRepositoryError, ReviewRepository, ReviewRepositoryError,
        SeatHoldRepositoryError,
    },
    review::{
        DemoFareConditions, ReviewBaseFare, ReviewBaseFareLine, ReviewContext,
        ReviewIncludedBenefits, ReviewJourney, ReviewPassenger, ReviewPricing, ReviewPricingLine,
        ReviewSeat, StopType,
    },
    value_objects::PassengerCounts,
};

use super::{extras::load_selections, passengers::load_passengers, SqlxSeatHoldRepository};

#[async_trait]
impl ReviewRepository for SqlxSeatHoldRepository {
    async fn get_review(
        &self,
        hold_id: Uuid,
        token_hash: [u8; 32],
    ) -> Result<ReviewContext, ReviewRepositoryError> {
        let mut transaction = self
            .pool()
            .begin()
            .await
            .map_err(ReviewRepositoryError::Infrastructure)?;
        let hold_row = Self::locked_hold(&mut transaction, hold_id, token_hash)
            .await
            .map_err(map_hold_error)?;
        let fixture = load_fixture(&mut transaction, hold_id).await?;
        let counts = PassengerCounts::new(
            hold_row.adults as u8,
            hold_row.children as u8,
            hold_row.infants as u8,
        )
        .expect("database passenger constraints are valid");
        let hold = Self::hold_entity(&mut transaction, hold_row)
            .await
            .map_err(map_hold_error)?;
        ensure_seats_ready(&mut transaction, hold_id, counts, hold.seats.len()).await?;

        let passengers = load_passengers(&mut transaction, hold_id)
            .await
            .map_err(map_passenger_error)?;
        ensure_passengers_ready(&passengers, counts)?;
        let extras_saved_at = fixture
            .extras_saved_at
            .ok_or(ReviewRepositoryError::ExtrasNotReady)?;
        let selections = load_selections(&mut transaction, hold_id)
            .await
            .map_err(map_extra_error)?;
        let extras_amount = checked_extras_total(&selections)?;
        let base_fare_amount = fixture
            .base_fare_amount
            .ok_or(ReviewRepositoryError::PricingUnavailable)?;
        if fixture.currency_code.as_deref() != Some(REVIEW_CURRENCY)
            || selections
                .iter()
                .any(|selection| selection.unit_price.currency_code != REVIEW_CURRENCY)
        {
            return Err(ReviewRepositoryError::PricingUnavailable);
        }

        let snapshot = match load_snapshot(&mut transaction, hold_id, extras_saved_at).await? {
            Some(snapshot) => snapshot,
            None => {
                let calculated = calculate_review_pricing(base_fare_amount, counts, extras_amount)
                    .map_err(|_| ReviewRepositoryError::PricingUnavailable)?;
                materialize_snapshot(&mut transaction, hold_id, extras_saved_at, &calculated)
                    .await?
            }
        };
        if snapshot.currency_code != REVIEW_CURRENCY {
            return Err(ReviewRepositoryError::PricingUnavailable);
        }
        let journey = journey_from_fixture(&fixture)?;
        let catalog = catalog_for_cabin(hold.cabin);
        let review = ReviewContext {
            journey,
            passengers: review_passengers(passengers, &selections),
            seats: hold
                .seats
                .iter()
                .cloned()
                .map(|seat_number| ReviewSeat { seat_number })
                .collect(),
            included_benefits: ReviewIncludedBenefits {
                allowances: catalog.allowances,
                benefits: catalog.included_benefits,
            },
            pricing: pricing_from_snapshot(&snapshot, counts),
            fare_conditions: DemoFareConditions {
                code: "DEMO_FIXTURE_NONREFUNDABLE_NO_CHANGES",
                fixture: true,
                refundable: false,
                changes_allowed: false,
            },
            ready_for_payment: true,
            hold,
        };
        transaction
            .commit()
            .await
            .map_err(ReviewRepositoryError::Infrastructure)?;
        Ok(review)
    }
}

async fn load_fixture(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
) -> Result<ReviewFixtureRow, ReviewRepositoryError> {
    sqlx::query_as::<_, ReviewFixtureRow>(
        "SELECT service.flight_number, service.origin_code, service.destination_code,
                service.aircraft_code, service.departure_time, service.arrival_time,
                service.arrival_day_offset, service.duration_minutes, service.stops,
                cabin.base_fare_amount, cabin.currency_code, hold.extras_saved_at
         FROM seat_holds AS hold
         JOIN flight_instances AS instance ON instance.id = hold.flight_instance_id
         JOIN flight_services AS service ON service.id = instance.flight_service_id
         JOIN flight_service_cabins AS cabin
           ON cabin.flight_service_id = service.id AND cabin.cabin = hold.cabin
         WHERE hold.id = $1",
    )
    .bind(hold_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(ReviewRepositoryError::Infrastructure)
}

async fn ensure_seats_ready(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
    counts: PassengerCounts,
    held_seat_count: usize,
) -> Result<(), ReviewRepositoryError> {
    let valid_seat_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM flight_seats
         WHERE hold_id = $1 AND sellable = TRUE AND booking_status = 'AVAILABLE'",
    )
    .bind(hold_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(ReviewRepositoryError::Infrastructure)?;
    if held_seat_count != counts.required_seats()
        || valid_seat_count as usize != counts.required_seats()
    {
        return Err(ReviewRepositoryError::SeatsNotReady);
    }
    Ok(())
}

fn ensure_passengers_ready(
    passengers: &[Passenger],
    counts: PassengerCounts,
) -> Result<(), ReviewRepositoryError> {
    let expected = expected_passenger_slots(counts);
    if passengers.len() != expected.len()
        || passengers.iter().zip(expected).any(|(actual, expected)| {
            actual.ordinal != expected.ordinal || actual.passenger_type != expected.passenger_type
        })
    {
        return Err(ReviewRepositoryError::PassengersNotReady);
    }
    Ok(())
}

fn checked_extras_total(selections: &[PricedExtraSelection]) -> Result<i64, ReviewRepositoryError> {
    selections.iter().try_fold(0_i64, |total, selection| {
        total
            .checked_add(selection.line_total.amount)
            .ok_or(ReviewRepositoryError::PricingUnavailable)
    })
}

fn review_passengers(
    passengers: Vec<Passenger>,
    selections: &[PricedExtraSelection],
) -> Vec<ReviewPassenger> {
    passengers
        .into_iter()
        .map(|passenger| {
            let middle = passenger
                .middle_name
                .as_deref()
                .map(|name| format!(" {name}"))
                .unwrap_or_default();
            ReviewPassenger {
                ordinal: passenger.ordinal,
                passenger_type: passenger.passenger_type,
                title: passenger.title,
                display_name: format!(
                    "{} {}{} {}",
                    passenger.title.as_str(),
                    passenger.given_name,
                    middle,
                    passenger.family_name
                ),
                nationality_code: passenger.nationality_code,
                travel_document_complete: true,
                extras: selections
                    .iter()
                    .filter(|selection| selection.passenger_ordinal == passenger.ordinal)
                    .cloned()
                    .collect(),
            }
        })
        .collect()
}

fn journey_from_fixture(
    fixture: &ReviewFixtureRow,
) -> Result<ReviewJourney, ReviewRepositoryError> {
    Ok(ReviewJourney {
        flight_number: fixture.flight_number.clone(),
        origin_code: fixture.origin_code.clone(),
        destination_code: fixture.destination_code.clone(),
        aircraft_code: fixture.aircraft_code.clone(),
        departure_time: required_time(fixture.departure_time)?,
        arrival_time: required_time(fixture.arrival_time)?,
        arrival_day_offset: fixture
            .arrival_day_offset
            .and_then(|value| u8::try_from(value).ok())
            .ok_or(ReviewRepositoryError::PricingUnavailable)?,
        duration_minutes: fixture
            .duration_minutes
            .and_then(|value| u16::try_from(value).ok())
            .ok_or(ReviewRepositoryError::PricingUnavailable)?,
        stops: match fixture.stops.as_deref() {
            Some("DIRECT") => StopType::Direct,
            Some("ONE_STOP") => StopType::OneStop,
            _ => return Err(ReviewRepositoryError::PricingUnavailable),
        },
    })
}

fn required_time(value: Option<NaiveTime>) -> Result<String, ReviewRepositoryError> {
    value
        .map(|time| time.format("%H:%M").to_string())
        .ok_or(ReviewRepositoryError::PricingUnavailable)
}

async fn load_snapshot(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
    source_extras_saved_at: DateTime<Utc>,
) -> Result<Option<SnapshotRow>, ReviewRepositoryError> {
    sqlx::query_as::<_, SnapshotRow>(
        "SELECT currency_code, seated_base_fare_unit_amount, infant_base_fare_unit_amount,
                base_fare_amount, extras_amount, demo_passenger_tax_unit_amount,
                taxes_amount, demo_airport_fee_unit_amount, demo_airport_fee_amount,
                demo_booking_fee_amount, fees_amount, grand_total_amount, priced_at
         FROM hold_review_pricing
         WHERE seat_hold_id = $1 AND source_extras_saved_at = $2",
    )
    .bind(hold_id)
    .bind(source_extras_saved_at)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(ReviewRepositoryError::Infrastructure)
}

async fn materialize_snapshot(
    transaction: &mut Transaction<'_, Postgres>,
    hold_id: Uuid,
    source_extras_saved_at: DateTime<Utc>,
    pricing: &CalculatedReviewPricing,
) -> Result<SnapshotRow, ReviewRepositoryError> {
    let seated_unit_amount = pricing
        .base_fare_lines
        .iter()
        .find(|line| line.passenger_type != PassengerType::Infant)
        .map(|line| line.unit_amount)
        .ok_or(ReviewRepositoryError::PricingUnavailable)?;
    let airport_fee_amount = pricing
        .fee_lines
        .iter()
        .find(|line| line.code == "DEMO_AIRPORT_FEE")
        .map(|line| line.amount)
        .ok_or(ReviewRepositoryError::PricingUnavailable)?;
    sqlx::query_as::<_, SnapshotRow>(
        "INSERT INTO hold_review_pricing (
            seat_hold_id, source_extras_saved_at, currency_code,
            seated_base_fare_unit_amount, infant_base_fare_unit_amount,
            base_fare_amount, extras_amount, demo_passenger_tax_unit_amount,
            taxes_amount, demo_airport_fee_unit_amount, demo_airport_fee_amount,
            demo_booking_fee_amount, fees_amount, grand_total_amount
         ) VALUES ($1, $2, $3, $4, 0, $5, $6, $7, $8, $9, $10, $11, $12, $13)
         ON CONFLICT (seat_hold_id) DO UPDATE SET
            source_extras_saved_at = EXCLUDED.source_extras_saved_at,
            currency_code = EXCLUDED.currency_code,
            seated_base_fare_unit_amount = EXCLUDED.seated_base_fare_unit_amount,
            infant_base_fare_unit_amount = EXCLUDED.infant_base_fare_unit_amount,
            base_fare_amount = EXCLUDED.base_fare_amount,
            extras_amount = EXCLUDED.extras_amount,
            demo_passenger_tax_unit_amount = EXCLUDED.demo_passenger_tax_unit_amount,
            taxes_amount = EXCLUDED.taxes_amount,
            demo_airport_fee_unit_amount = EXCLUDED.demo_airport_fee_unit_amount,
            demo_airport_fee_amount = EXCLUDED.demo_airport_fee_amount,
            demo_booking_fee_amount = EXCLUDED.demo_booking_fee_amount,
            fees_amount = EXCLUDED.fees_amount,
            grand_total_amount = EXCLUDED.grand_total_amount,
            priced_at = NOW()
         RETURNING currency_code, seated_base_fare_unit_amount, infant_base_fare_unit_amount,
            base_fare_amount, extras_amount, demo_passenger_tax_unit_amount,
            taxes_amount, demo_airport_fee_unit_amount, demo_airport_fee_amount,
            demo_booking_fee_amount, fees_amount, grand_total_amount, priced_at",
    )
    .bind(hold_id)
    .bind(source_extras_saved_at)
    .bind(pricing.currency_code)
    .bind(seated_unit_amount)
    .bind(pricing.base_fare_amount)
    .bind(pricing.extras_amount)
    .bind(DEMO_PASSENGER_TAX_AMOUNT)
    .bind(pricing.taxes_amount)
    .bind(DEMO_AIRPORT_FEE_AMOUNT)
    .bind(airport_fee_amount)
    .bind(DEMO_BOOKING_FEE_AMOUNT)
    .bind(pricing.fees_amount)
    .bind(pricing.grand_total_amount)
    .fetch_one(&mut **transaction)
    .await
    .map_err(ReviewRepositoryError::Infrastructure)
}

fn pricing_from_snapshot(snapshot: &SnapshotRow, counts: PassengerCounts) -> ReviewPricing {
    let money = |amount| Money {
        amount,
        currency_code: snapshot.currency_code.clone(),
    };
    let base_inputs = [
        (
            PassengerType::Adult,
            counts.adults(),
            snapshot.seated_base_fare_unit_amount,
        ),
        (
            PassengerType::Child,
            counts.children(),
            snapshot.seated_base_fare_unit_amount,
        ),
        (
            PassengerType::Infant,
            counts.infants(),
            snapshot.infant_base_fare_unit_amount,
        ),
    ];
    let lines = base_inputs
        .into_iter()
        .filter(|(_, quantity, _)| *quantity > 0)
        .map(
            |(passenger_type, quantity, unit_amount)| ReviewBaseFareLine {
                passenger_type,
                quantity,
                unit_amount: money(unit_amount),
                amount: money(unit_amount * i64::from(quantity)),
            },
        )
        .collect();
    let seated_count = counts.adults() + counts.children();
    ReviewPricing {
        currency_code: snapshot.currency_code.clone(),
        base_fare: ReviewBaseFare {
            lines,
            amount: money(snapshot.base_fare_amount),
        },
        extras: money(snapshot.extras_amount),
        taxes: vec![ReviewPricingLine {
            code: "DEMO_PASSENGER_TAX",
            quantity: seated_count,
            unit_amount: money(snapshot.demo_passenger_tax_unit_amount),
            amount: money(snapshot.taxes_amount),
        }],
        fees: vec![
            ReviewPricingLine {
                code: "DEMO_AIRPORT_FEE",
                quantity: seated_count,
                unit_amount: money(snapshot.demo_airport_fee_unit_amount),
                amount: money(snapshot.demo_airport_fee_amount),
            },
            ReviewPricingLine {
                code: "DEMO_BOOKING_FEE",
                quantity: 1,
                unit_amount: money(snapshot.demo_booking_fee_amount),
                amount: money(snapshot.demo_booking_fee_amount),
            },
        ],
        grand_total: money(snapshot.grand_total_amount),
        priced_at: snapshot.priced_at,
    }
}

fn map_hold_error(error: SeatHoldRepositoryError) -> ReviewRepositoryError {
    match error {
        SeatHoldRepositoryError::HoldNotFound => ReviewRepositoryError::HoldNotFound,
        SeatHoldRepositoryError::Unauthorized => ReviewRepositoryError::Unauthorized,
        SeatHoldRepositoryError::HoldExpired => ReviewRepositoryError::HoldExpired,
        SeatHoldRepositoryError::HoldReleased => ReviewRepositoryError::HoldReleased,
        SeatHoldRepositoryError::HoldConsumed => ReviewRepositoryError::HoldConsumed,
        SeatHoldRepositoryError::Infrastructure(error) => {
            ReviewRepositoryError::Infrastructure(error)
        }
        _ => ReviewRepositoryError::SeatsNotReady,
    }
}

fn map_passenger_error(error: PassengerRepositoryError) -> ReviewRepositoryError {
    match error {
        PassengerRepositoryError::Infrastructure(error) => {
            ReviewRepositoryError::Infrastructure(error)
        }
        _ => ReviewRepositoryError::PassengersNotReady,
    }
}

fn map_extra_error(error: ExtraRepositoryError) -> ReviewRepositoryError {
    match error {
        ExtraRepositoryError::Infrastructure(error) => ReviewRepositoryError::Infrastructure(error),
        _ => ReviewRepositoryError::PricingUnavailable,
    }
}

#[derive(FromRow)]
struct ReviewFixtureRow {
    flight_number: String,
    origin_code: String,
    destination_code: String,
    aircraft_code: String,
    departure_time: Option<NaiveTime>,
    arrival_time: Option<NaiveTime>,
    arrival_day_offset: Option<i16>,
    duration_minutes: Option<i16>,
    stops: Option<String>,
    base_fare_amount: Option<i64>,
    currency_code: Option<String>,
    extras_saved_at: Option<DateTime<Utc>>,
}

#[derive(FromRow)]
struct SnapshotRow {
    currency_code: String,
    seated_base_fare_unit_amount: i64,
    infant_base_fare_unit_amount: i64,
    base_fare_amount: i64,
    extras_amount: i64,
    demo_passenger_tax_unit_amount: i64,
    taxes_amount: i64,
    demo_airport_fee_unit_amount: i64,
    demo_airport_fee_amount: i64,
    demo_booking_fee_amount: i64,
    #[allow(dead_code)]
    fees_amount: i64,
    grand_total_amount: i64,
    priced_at: DateTime<Utc>,
}
