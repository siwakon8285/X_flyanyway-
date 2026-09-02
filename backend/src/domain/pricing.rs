use thiserror::Error;

use crate::domain::{passengers::PassengerType, value_objects::PassengerCounts};

pub const REVIEW_CURRENCY: &str = "THB";
pub const DEMO_PASSENGER_TAX_AMOUNT: i64 = 700;
pub const DEMO_AIRPORT_FEE_AMOUNT: i64 = 500;
pub const DEMO_BOOKING_FEE_AMOUNT: i64 = 300;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PricingLine {
    pub code: &'static str,
    pub quantity: u8,
    pub unit_amount: i64,
    pub amount: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseFareLine {
    pub passenger_type: PassengerType,
    pub quantity: u8,
    pub unit_amount: i64,
    pub amount: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalculatedReviewPricing {
    pub currency_code: &'static str,
    pub base_fare_lines: Vec<BaseFareLine>,
    pub base_fare_amount: i64,
    pub extras_amount: i64,
    pub tax_lines: Vec<PricingLine>,
    pub taxes_amount: i64,
    pub fee_lines: Vec<PricingLine>,
    pub fees_amount: i64,
    pub grand_total_amount: i64,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PricingError {
    #[error("base fare fixture must be positive")]
    InvalidBaseFare,
    #[error("extras amount must not be negative")]
    InvalidExtrasAmount,
    #[error("pricing arithmetic overflowed")]
    Overflow,
}

pub fn calculate_review_pricing(
    seated_base_fare_amount: i64,
    counts: PassengerCounts,
    extras_amount: i64,
) -> Result<CalculatedReviewPricing, PricingError> {
    if seated_base_fare_amount <= 0 {
        return Err(PricingError::InvalidBaseFare);
    }
    if extras_amount < 0 {
        return Err(PricingError::InvalidExtrasAmount);
    }

    let adults_amount = multiply(seated_base_fare_amount, counts.adults())?;
    let children_amount = multiply(seated_base_fare_amount, counts.children())?;
    let base_fare_amount = add(adults_amount, children_amount)?;
    let seated_count = counts.adults() + counts.children();
    let passenger_tax_amount = multiply(DEMO_PASSENGER_TAX_AMOUNT, seated_count)?;
    let airport_fee_amount = multiply(DEMO_AIRPORT_FEE_AMOUNT, seated_count)?;
    let taxes_amount = passenger_tax_amount;
    let fees_amount = add(airport_fee_amount, DEMO_BOOKING_FEE_AMOUNT)?;
    let grand_total_amount = [base_fare_amount, extras_amount, taxes_amount, fees_amount]
        .into_iter()
        .try_fold(0_i64, add)?;

    let mut base_fare_lines = Vec::with_capacity(3);
    push_base_fare_line(
        &mut base_fare_lines,
        PassengerType::Adult,
        counts.adults(),
        seated_base_fare_amount,
        adults_amount,
    );
    push_base_fare_line(
        &mut base_fare_lines,
        PassengerType::Child,
        counts.children(),
        seated_base_fare_amount,
        children_amount,
    );
    push_base_fare_line(
        &mut base_fare_lines,
        PassengerType::Infant,
        counts.infants(),
        0,
        0,
    );

    Ok(CalculatedReviewPricing {
        currency_code: REVIEW_CURRENCY,
        base_fare_lines,
        base_fare_amount,
        extras_amount,
        tax_lines: vec![PricingLine {
            code: "DEMO_PASSENGER_TAX",
            quantity: seated_count,
            unit_amount: DEMO_PASSENGER_TAX_AMOUNT,
            amount: passenger_tax_amount,
        }],
        taxes_amount,
        fee_lines: vec![
            PricingLine {
                code: "DEMO_AIRPORT_FEE",
                quantity: seated_count,
                unit_amount: DEMO_AIRPORT_FEE_AMOUNT,
                amount: airport_fee_amount,
            },
            PricingLine {
                code: "DEMO_BOOKING_FEE",
                quantity: 1,
                unit_amount: DEMO_BOOKING_FEE_AMOUNT,
                amount: DEMO_BOOKING_FEE_AMOUNT,
            },
        ],
        fees_amount,
        grand_total_amount,
    })
}

fn multiply(amount: i64, quantity: u8) -> Result<i64, PricingError> {
    amount
        .checked_mul(i64::from(quantity))
        .ok_or(PricingError::Overflow)
}

fn add(first: i64, second: i64) -> Result<i64, PricingError> {
    first.checked_add(second).ok_or(PricingError::Overflow)
}

fn push_base_fare_line(
    lines: &mut Vec<BaseFareLine>,
    passenger_type: PassengerType,
    quantity: u8,
    unit_amount: i64,
    amount: i64,
) {
    if quantity > 0 {
        lines.push(BaseFareLine {
            passenger_type,
            quantity,
            unit_amount,
            amount,
        });
    }
}
