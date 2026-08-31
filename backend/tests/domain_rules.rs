use x_fly_api::domain::value_objects::{CabinClass, PassengerCounts, SeatNumber};

#[test]
fn lap_infants_do_not_increase_required_seat_count() {
    let party = PassengerCounts::new(2, 1, 1).expect("valid passenger party");

    assert_eq!(party.required_seats(), 3);
}

#[test]
fn passenger_party_requires_an_adult_and_cannot_have_more_infants_than_adults() {
    assert!(PassengerCounts::new(0, 1, 0).is_err());
    assert!(PassengerCounts::new(1, 0, 2).is_err());
}

#[test]
fn seat_numbers_are_normalized_and_reject_invalid_input() {
    assert_eq!(SeatNumber::parse(" 18a ").unwrap().as_str(), "18A");
    assert!(SeatNumber::parse("row-18-a").is_err());
}

#[test]
fn cabin_values_match_the_existing_query_contract() {
    assert_eq!(CabinClass::PremiumEconomy.as_str(), "premium-economy");
    assert_eq!(
        "business".parse::<CabinClass>().unwrap(),
        CabinClass::Business
    );
}
