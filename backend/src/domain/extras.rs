use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::{
    entities::SeatHold,
    passengers::{PassengerSlot, PassengerType},
    value_objects::CabinClass,
};

pub const EXTRAS_CURRENCY: &str = "THB";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExtraCategory {
    Baggage,
    Meal,
    Assistance,
}

impl ExtraCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Baggage => "BAGGAGE",
            Self::Meal => "MEAL",
            Self::Assistance => "ASSISTANCE",
        }
    }

    pub fn parse_database(value: &str) -> Option<Self> {
        match value {
            "BAGGAGE" => Some(Self::Baggage),
            "MEAL" => Some(Self::Meal),
            "ASSISTANCE" => Some(Self::Assistance),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Money {
    pub amount: i64,
    pub currency_code: String,
}

fn money(amount: i64) -> Money {
    Money {
        amount,
        currency_code: EXTRAS_CURRENCY.to_owned(),
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaggageAllowances {
    pub cabin_baggage_kg: u8,
    pub checked_baggage_kg: u8,
    pub applies_to_passenger_types: Vec<PassengerType>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncludedBenefits {
    pub seat_selection_included: bool,
    pub meal_service: &'static str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraProduct {
    pub code: &'static str,
    pub category: ExtraCategory,
    pub unit_price: Money,
    pub max_quantity: u8,
    pub eligible_passenger_types: Vec<PassengerType>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraCatalog {
    pub currency_code: &'static str,
    pub allowances: BaggageAllowances,
    pub included_benefits: IncludedBenefits,
    pub products: Vec<ExtraProduct>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtraSelectionInput {
    pub passenger_ordinal: u8,
    pub product_code: String,
    pub quantity: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PricedExtraSelection {
    pub passenger_ordinal: u8,
    pub product_code: String,
    pub category: ExtraCategory,
    pub quantity: u8,
    pub unit_price: Money,
    pub line_total: Money,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraContext {
    pub hold: SeatHold,
    pub passengers: Vec<PassengerSlot>,
    pub catalog: ExtraCatalog,
    pub selections: Vec<PricedExtraSelection>,
    pub total: Money,
    pub ready_to_continue: bool,
    pub saved_at: Option<DateTime<Utc>>,
}

pub fn total_for_selections(selections: &[PricedExtraSelection]) -> Money {
    money(
        selections
            .iter()
            .map(|selection| selection.line_total.amount)
            .sum(),
    )
}

#[derive(Debug, Error)]
pub enum ExtraValidationError {
    #[error("extra product code is unknown")]
    UnknownProduct,
    #[error("extra quantity is invalid")]
    InvalidQuantity,
    #[error("passenger ordinal does not belong to the active hold")]
    InvalidPassenger,
    #[error("passenger is not eligible for the selected extra")]
    PassengerIneligible,
    #[error("passenger has conflicting selections in one category")]
    CategoryConflict,
}

pub fn catalog_for_cabin(cabin: CabinClass) -> ExtraCatalog {
    let (cabin_baggage_kg, checked_baggage_kg, meal_service) = match cabin {
        CabinClass::Economy => (7, 20, "STANDARD"),
        CabinClass::PremiumEconomy => (7, 25, "ENHANCED"),
        CabinClass::Business => (10, 40, "PREMIUM"),
        CabinClass::First => (14, 50, "SIGNATURE"),
    };

    ExtraCatalog {
        currency_code: EXTRAS_CURRENCY,
        allowances: BaggageAllowances {
            cabin_baggage_kg,
            checked_baggage_kg,
            applies_to_passenger_types: vec![PassengerType::Adult, PassengerType::Child],
        },
        included_benefits: IncludedBenefits {
            seat_selection_included: true,
            meal_service,
        },
        products: product_catalog(),
    }
}

pub fn price_extra_selections(
    inputs: &[ExtraSelectionInput],
    passengers: &[PassengerSlot],
) -> Result<Vec<PricedExtraSelection>, ExtraValidationError> {
    let products = product_catalog();
    let mut selected_products = HashSet::new();
    let mut exclusive_categories = HashSet::new();
    let mut priced = Vec::with_capacity(inputs.len());

    for input in inputs {
        let product = products
            .iter()
            .find(|product| product.code == input.product_code)
            .ok_or(ExtraValidationError::UnknownProduct)?;
        if input.quantity == 0 || input.quantity > product.max_quantity {
            return Err(ExtraValidationError::InvalidQuantity);
        }
        let passenger = passengers
            .iter()
            .find(|passenger| passenger.ordinal == input.passenger_ordinal)
            .ok_or(ExtraValidationError::InvalidPassenger)?;
        if !product
            .eligible_passenger_types
            .contains(&passenger.passenger_type)
        {
            return Err(ExtraValidationError::PassengerIneligible);
        }
        if !selected_products.insert((input.passenger_ordinal, product.code)) {
            return Err(ExtraValidationError::CategoryConflict);
        }
        if product.category != ExtraCategory::Assistance
            && !exclusive_categories.insert((input.passenger_ordinal, product.category))
        {
            return Err(ExtraValidationError::CategoryConflict);
        }

        let line_amount = product.unit_price.amount * i64::from(input.quantity);
        priced.push(PricedExtraSelection {
            passenger_ordinal: input.passenger_ordinal,
            product_code: product.code.to_owned(),
            category: product.category,
            quantity: input.quantity,
            unit_price: product.unit_price.clone(),
            line_total: money(line_amount),
        });
    }

    priced.sort_by_key(|selection| {
        (
            selection.passenger_ordinal,
            selection.category.as_str(),
            selection.product_code.clone(),
        )
    });
    Ok(priced)
}

fn product_catalog() -> Vec<ExtraProduct> {
    let adult_child = || vec![PassengerType::Adult, PassengerType::Child];
    let child = || vec![PassengerType::Child];
    vec![
        product("BAG_10KG", ExtraCategory::Baggage, 1_500, adult_child()),
        product("BAG_20KG", ExtraCategory::Baggage, 2_800, adult_child()),
        product("BAG_30KG", ExtraCategory::Baggage, 3_900, adult_child()),
        product("MEAL_VEGETARIAN", ExtraCategory::Meal, 0, adult_child()),
        product("MEAL_VEGAN", ExtraCategory::Meal, 0, adult_child()),
        product("MEAL_HALAL", ExtraCategory::Meal, 0, adult_child()),
        product("MEAL_KOSHER", ExtraCategory::Meal, 0, adult_child()),
        product("MEAL_CHILD", ExtraCategory::Meal, 0, child()),
        product(
            "ASSIST_WHEELCHAIR",
            ExtraCategory::Assistance,
            0,
            adult_child(),
        ),
        product(
            "ASSIST_MOBILITY",
            ExtraCategory::Assistance,
            0,
            adult_child(),
        ),
        product("ASSIST_VISUAL", ExtraCategory::Assistance, 0, adult_child()),
        product(
            "ASSIST_HEARING",
            ExtraCategory::Assistance,
            0,
            adult_child(),
        ),
    ]
}

fn product(
    code: &'static str,
    category: ExtraCategory,
    amount: i64,
    eligible_passenger_types: Vec<PassengerType>,
) -> ExtraProduct {
    ExtraProduct {
        code,
        category,
        unit_price: money(amount),
        max_quantity: 1,
        eligible_passenger_types,
    }
}
