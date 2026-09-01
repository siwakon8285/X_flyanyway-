use x_fly_api::domain::{
    extras::{
        catalog_for_cabin, price_extra_selections, ExtraSelectionInput, ExtraValidationError,
    },
    passengers::{PassengerSlot, PassengerType},
    value_objects::CabinClass,
};

fn passenger(ordinal: u8, passenger_type: PassengerType) -> PassengerSlot {
    PassengerSlot {
        ordinal,
        passenger_type,
    }
}

#[test]
fn cabin_catalogs_expose_the_approved_included_allowances() {
    let cases = [
        (CabinClass::Economy, 7, 20, "STANDARD"),
        (CabinClass::PremiumEconomy, 7, 25, "ENHANCED"),
        (CabinClass::Business, 10, 40, "PREMIUM"),
        (CabinClass::First, 14, 50, "SIGNATURE"),
    ];

    for (cabin, cabin_kg, checked_kg, meal_service) in cases {
        let catalog = catalog_for_cabin(cabin);
        assert_eq!(catalog.currency_code, "THB");
        assert_eq!(catalog.allowances.cabin_baggage_kg, cabin_kg);
        assert_eq!(catalog.allowances.checked_baggage_kg, checked_kg);
        assert!(catalog.included_benefits.seat_selection_included);
        assert_eq!(catalog.included_benefits.meal_service, meal_service);
    }
}

#[test]
fn baggage_prices_are_server_defined_whole_baht_amounts() {
    let passengers = [passenger(1, PassengerType::Adult)];
    let selections = price_extra_selections(
        &[ExtraSelectionInput {
            passenger_ordinal: 1,
            product_code: "BAG_20KG".to_owned(),
            quantity: 1,
        }],
        &passengers,
    )
    .unwrap();

    assert_eq!(selections[0].unit_price.amount, 2_800);
    assert_eq!(selections[0].line_total.amount, 2_800);
    assert_eq!(selections[0].unit_price.currency_code, "THB");
}

#[test]
fn rejects_unknown_products_invalid_quantities_and_unknown_passengers() {
    let passengers = [passenger(1, PassengerType::Adult)];
    let unknown = price_extra_selections(
        &[ExtraSelectionInput {
            passenger_ordinal: 1,
            product_code: "CLIENT_PRICE_1".to_owned(),
            quantity: 1,
        }],
        &passengers,
    );
    assert!(matches!(unknown, Err(ExtraValidationError::UnknownProduct)));

    let invalid_quantity = price_extra_selections(
        &[ExtraSelectionInput {
            passenger_ordinal: 1,
            product_code: "BAG_10KG".to_owned(),
            quantity: 2,
        }],
        &passengers,
    );
    assert!(matches!(
        invalid_quantity,
        Err(ExtraValidationError::InvalidQuantity)
    ));

    let unknown_passenger = price_extra_selections(
        &[ExtraSelectionInput {
            passenger_ordinal: 2,
            product_code: "BAG_10KG".to_owned(),
            quantity: 1,
        }],
        &passengers,
    );
    assert!(matches!(
        unknown_passenger,
        Err(ExtraValidationError::InvalidPassenger)
    ));
}

#[test]
fn enforces_infant_child_meal_and_category_rules() {
    let passengers = [
        passenger(1, PassengerType::Adult),
        passenger(2, PassengerType::Child),
        passenger(3, PassengerType::Infant),
    ];
    let infant_baggage = price_extra_selections(
        &[ExtraSelectionInput {
            passenger_ordinal: 3,
            product_code: "BAG_10KG".to_owned(),
            quantity: 1,
        }],
        &passengers,
    );
    assert!(matches!(
        infant_baggage,
        Err(ExtraValidationError::PassengerIneligible)
    ));

    let adult_child_meal = price_extra_selections(
        &[ExtraSelectionInput {
            passenger_ordinal: 1,
            product_code: "MEAL_CHILD".to_owned(),
            quantity: 1,
        }],
        &passengers,
    );
    assert!(matches!(
        adult_child_meal,
        Err(ExtraValidationError::PassengerIneligible)
    ));

    let duplicate_baggage = price_extra_selections(
        &[
            ExtraSelectionInput {
                passenger_ordinal: 2,
                product_code: "BAG_10KG".to_owned(),
                quantity: 1,
            },
            ExtraSelectionInput {
                passenger_ordinal: 2,
                product_code: "BAG_20KG".to_owned(),
                quantity: 1,
            },
        ],
        &passengers,
    );
    assert!(matches!(
        duplicate_baggage,
        Err(ExtraValidationError::CategoryConflict)
    ));

    let assistance = price_extra_selections(
        &[
            ExtraSelectionInput {
                passenger_ordinal: 1,
                product_code: "ASSIST_VISUAL".to_owned(),
                quantity: 1,
            },
            ExtraSelectionInput {
                passenger_ordinal: 1,
                product_code: "ASSIST_HEARING".to_owned(),
                quantity: 1,
            },
        ],
        &passengers,
    )
    .unwrap();
    assert_eq!(assistance.len(), 2);
}
