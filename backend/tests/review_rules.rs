use x_fly_api::domain::{
    pricing::{calculate_review_pricing, PricingError},
    value_objects::PassengerCounts,
};

#[test]
fn calculates_server_owned_whole_baht_review_pricing() {
    let pricing =
        calculate_review_pricing(21_900, PassengerCounts::new(1, 1, 1).unwrap(), 2_800).unwrap();

    assert_eq!(pricing.currency_code, "THB");
    assert_eq!(pricing.base_fare_lines.len(), 3);
    assert_eq!(pricing.base_fare_lines[0].passenger_type.as_str(), "ADULT");
    assert_eq!(pricing.base_fare_lines[0].unit_amount, 21_900);
    assert_eq!(pricing.base_fare_lines[0].amount, 21_900);
    assert_eq!(pricing.base_fare_lines[1].passenger_type.as_str(), "CHILD");
    assert_eq!(pricing.base_fare_lines[1].amount, 21_900);
    assert_eq!(pricing.base_fare_lines[2].passenger_type.as_str(), "INFANT");
    assert_eq!(pricing.base_fare_lines[2].unit_amount, 0);
    assert_eq!(pricing.base_fare_lines[2].amount, 0);
    assert_eq!(pricing.base_fare_amount, 43_800);
    assert_eq!(pricing.extras_amount, 2_800);
    assert_eq!(pricing.tax_lines[0].code, "DEMO_PASSENGER_TAX");
    assert_eq!(pricing.tax_lines[0].quantity, 2);
    assert_eq!(pricing.tax_lines[0].unit_amount, 700);
    assert_eq!(pricing.tax_lines[0].amount, 1_400);
    assert_eq!(pricing.fee_lines[0].code, "DEMO_AIRPORT_FEE");
    assert_eq!(pricing.fee_lines[0].amount, 1_000);
    assert_eq!(pricing.fee_lines[1].code, "DEMO_BOOKING_FEE");
    assert_eq!(pricing.fee_lines[1].amount, 300);
    assert_eq!(pricing.grand_total_amount, 49_300);
}

#[test]
fn rejects_invalid_or_overflowing_authoritative_amounts() {
    let counts = PassengerCounts::new(9, 0, 0).unwrap();

    assert_eq!(
        calculate_review_pricing(0, counts, 0),
        Err(PricingError::InvalidBaseFare)
    );
    assert_eq!(
        calculate_review_pricing(10_000, counts, -1),
        Err(PricingError::InvalidExtrasAmount)
    );
    assert_eq!(
        calculate_review_pricing(i64::MAX, counts, 0),
        Err(PricingError::Overflow)
    );
}
