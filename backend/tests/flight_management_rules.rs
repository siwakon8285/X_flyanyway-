use chrono::{NaiveDate, NaiveTime};
use x_fly_api::domain::flight::{FlightCommand, FlightNumber, FlightValidationError};

fn valid_command() -> FlightCommand {
    FlightCommand {
        flight_number: " xf   701 ".to_owned(),
        origin_code: "BKK".to_owned(),
        destination_code: "DXB".to_owned(),
        operating_date: NaiveDate::from_ymd_opt(2026, 9, 8),
        departure_time: NaiveTime::from_hms_opt(9, 20, 0).unwrap(),
        arrival_time: NaiveTime::from_hms_opt(13, 5, 0).unwrap(),
        arrival_day_offset: 0,
        aircraft_code: "Boeing 787-9".to_owned(),
        business_price_amount: 46_900,
        first_price_amount: 78_900,
        currency_code: "THB".to_owned(),
        business_capacity: 16,
        first_capacity: 4,
    }
}

#[test]
fn normalizes_the_existing_xf_flight_number_format() {
    assert_eq!(
        FlightNumber::parse(" xf   701 ").unwrap().as_str(),
        "XF 701"
    );
    for invalid in ["XF701", "AB 701", "XF 07", "XF 1000", "XF 7A1"] {
        assert!(FlightNumber::parse(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn validates_the_business_and_first_management_contract() {
    let validated = valid_command().validate().unwrap();
    assert_eq!(validated.flight_number.as_str(), "XF 701");
    assert_eq!(validated.currency_code, "THB");
    assert_eq!(validated.business_capacity, 16);
    assert_eq!(validated.first_capacity, 4);
}

#[test]
fn rejects_invalid_route_schedule_price_and_inventory_values() {
    let cases = [
        (
            "same route",
            Box::new(|command: &mut FlightCommand| command.destination_code = "BKK".into())
                as Box<dyn Fn(&mut FlightCommand)>,
            FlightValidationError::Route,
        ),
        (
            "bad airport",
            Box::new(|command: &mut FlightCommand| command.origin_code = "Bangkok".into()),
            FlightValidationError::Route,
        ),
        (
            "bad day offset",
            Box::new(|command: &mut FlightCommand| command.arrival_day_offset = 2),
            FlightValidationError::Schedule,
        ),
        (
            "empty aircraft",
            Box::new(|command: &mut FlightCommand| command.aircraft_code = "  ".into()),
            FlightValidationError::Aircraft,
        ),
        (
            "zero price",
            Box::new(|command: &mut FlightCommand| command.business_price_amount = 0),
            FlightValidationError::Price,
        ),
        (
            "wrong currency",
            Box::new(|command: &mut FlightCommand| command.currency_code = "USD".into()),
            FlightValidationError::Price,
        ),
        (
            "zero capacity",
            Box::new(|command: &mut FlightCommand| command.first_capacity = 0),
            FlightValidationError::Capacity,
        ),
        (
            "unsupported business geometry",
            Box::new(|command: &mut FlightCommand| command.business_capacity = 15),
            FlightValidationError::Capacity,
        ),
        (
            "unsupported first geometry",
            Box::new(|command: &mut FlightCommand| command.first_capacity = 3),
            FlightValidationError::Capacity,
        ),
    ];

    for (name, mutate, expected) in cases {
        let mut command = valid_command();
        mutate(&mut command);
        assert_eq!(command.validate().unwrap_err(), expected, "{name}");
    }
}
