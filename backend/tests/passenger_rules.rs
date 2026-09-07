use chrono::NaiveDate;

use x_fly_api::domain::{
    passengers::{
        validate_booking_contact, BookingContactInput, EmergencyContactInput, Gender,
        PassengerDraft, PassengerDraftError, PassengerInput, PassengerType, Title,
    },
    value_objects::PassengerCounts,
};

fn date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
}

fn passenger(ordinal: u8, passenger_type: PassengerType, date_of_birth: &str) -> PassengerInput {
    PassengerInput {
        ordinal,
        passenger_type,
        title: Title::Mr,
        given_name: "  Alex  ".to_owned(),
        middle_name: Some("  Morgan  ".to_owned()),
        family_name: "  O’Connor  ".to_owned(),
        date_of_birth: date(date_of_birth),
        gender: Gender::Female,
        nationality_code: "th".to_owned(),
        passport_number: " ab-123 456 ".to_owned(),
        passport_issuing_country_code: "th".to_owned(),
        emergency_contact: None,
    }
}

#[test]
fn validates_and_normalizes_a_complete_passenger_draft() {
    let draft = PassengerDraft::validate(
        vec![passenger(1, PassengerType::Adult, "1990-05-10")],
        PassengerCounts::new(1, 0, 0).unwrap(),
        date("2030-05-10"),
        date("2026-08-31"),
    )
    .unwrap();

    let saved = &draft.passengers()[0];
    assert_eq!(saved.given_name, "Alex");
    assert_eq!(saved.middle_name.as_deref(), Some("Morgan"));
    assert_eq!(saved.family_name, "O’Connor");
    assert_eq!(saved.nationality_code, "TH");
    assert_eq!(saved.passport_number, "AB123456");
    assert_eq!(saved.gender, Gender::Female);
}

#[test]
fn requires_the_hold_defined_count_and_adult_child_infant_order() {
    let wrong_count = PassengerDraft::validate(
        vec![passenger(1, PassengerType::Adult, "1990-01-01")],
        PassengerCounts::new(1, 1, 0).unwrap(),
        date("2030-01-01"),
        date("2026-08-31"),
    );
    assert!(matches!(
        wrong_count,
        Err(PassengerDraftError::CountMismatch)
    ));

    let wrong_type = PassengerDraft::validate(
        vec![
            passenger(1, PassengerType::Child, "2020-01-01"),
            passenger(2, PassengerType::Adult, "1990-01-01"),
        ],
        PassengerCounts::new(1, 1, 0).unwrap(),
        date("2030-01-01"),
        date("2026-08-31"),
    );
    assert!(matches!(wrong_type, Err(PassengerDraftError::TypeMismatch)));
}

#[test]
fn applies_exact_age_boundaries_on_outbound_departure() {
    let departure = date("2030-06-15");
    let today = date("2030-06-01");

    for (passenger_type, dob) in [
        (PassengerType::Adult, "2018-06-15"),
        (PassengerType::Child, "2018-06-16"),
        (PassengerType::Child, "2028-06-15"),
        (PassengerType::Infant, "2028-06-16"),
    ] {
        let counts = match passenger_type {
            PassengerType::Adult => PassengerCounts::new(1, 0, 0).unwrap(),
            PassengerType::Child => PassengerCounts::new(1, 1, 0).unwrap(),
            PassengerType::Infant => PassengerCounts::new(1, 0, 1).unwrap(),
        };
        let mut passengers = vec![passenger(1, PassengerType::Adult, "1990-01-01")];
        if passenger_type != PassengerType::Adult {
            let mut dependent = passenger(2, passenger_type, dob);
            dependent.passport_number = "CD987654".to_owned();
            passengers.push(dependent);
        } else {
            passengers[0] = passenger(1, passenger_type, dob);
        }

        assert!(PassengerDraft::validate(passengers, counts, departure, today).is_ok());
    }

    let mismatch = PassengerDraft::validate(
        vec![passenger(1, PassengerType::Adult, "2018-06-16")],
        PassengerCounts::new(1, 0, 0).unwrap(),
        departure,
        today,
    )
    .err()
    .expect("adult younger than 12 must be rejected");
    assert_eq!(
        mismatch.field_errors()[0].code.as_str(),
        "AGE_CATEGORY_MISMATCH"
    );
}

#[test]
fn rejects_future_birth_without_requiring_passport_dates() {
    let departure = date("2030-05-10");
    let today = date("2026-08-31");

    let future = PassengerDraft::validate(
        vec![passenger(1, PassengerType::Adult, "2026-09-01")],
        PassengerCounts::new(1, 0, 0).unwrap(),
        departure,
        today,
    )
    .err()
    .expect("future date of birth must be rejected");
    assert_eq!(
        future.field_errors()[0].code.as_str(),
        "DATE_OF_BIRTH_FUTURE"
    );
}

#[test]
fn rejects_unspecified_gender_duplicate_passports_and_partial_emergency_contact() {
    let departure = date("2030-05-10");
    let today = date("2026-08-31");
    let mut invalid = passenger(1, PassengerType::Adult, "1990-01-01");
    invalid.gender = Gender::Unspecified;
    invalid.emergency_contact = Some(EmergencyContactInput {
        name: "Sam Lee".to_owned(),
        relationship: "".to_owned(),
        phone_country_code: "+66".to_owned(),
        phone_number: "0812345678".to_owned(),
    });
    let errors = PassengerDraft::validate(
        vec![invalid],
        PassengerCounts::new(1, 0, 0).unwrap(),
        departure,
        today,
    )
    .err()
    .expect("invalid contact fields must be rejected");
    let codes: Vec<_> = errors
        .field_errors()
        .iter()
        .map(|error| error.code.as_str())
        .collect();
    assert!(codes.contains(&"INVALID_GENDER"));
    assert!(codes.contains(&"EMERGENCY_CONTACT_INCOMPLETE"));

    let duplicate = PassengerDraft::validate(
        vec![
            passenger(1, PassengerType::Adult, "1990-01-01"),
            passenger(2, PassengerType::Adult, "1992-01-01"),
        ],
        PassengerCounts::new(2, 0, 0).unwrap(),
        departure,
        today,
    )
    .err()
    .expect("duplicate passports must be rejected");
    assert_eq!(
        duplicate.field_errors()[0].code.as_str(),
        "DUPLICATE_PASSPORT"
    );
}

#[test]
fn reports_missing_phone_parts_without_echoing_their_values() {
    let error = validate_booking_contact(BookingContactInput {
        phone_country_code: " ".to_owned(),
        phone_number: "812345678".to_owned(),
    })
    .err()
    .expect("phone country code is required");
    assert_eq!(error.as_str(), "INVALID_PHONE");

    let error = validate_booking_contact(BookingContactInput {
        phone_country_code: "+66".to_owned(),
        phone_number: " ".to_owned(),
    })
    .err()
    .expect("phone number is required");
    assert_eq!(error.as_str(), "INVALID_PHONE");
}

#[test]
fn rejects_letters_in_phone_fields_instead_of_normalizing_them_away() {
    let error = validate_booking_contact(BookingContactInput {
        phone_country_code: "+6x6".to_owned(),
        phone_number: "0812CALL345678".to_owned(),
    })
    .err()
    .expect("letters in phone fields must be rejected");
    assert_eq!(error.as_str(), "INVALID_PHONE");
}

#[test]
fn accepts_male_and_female_for_new_passengers() {
    for gender in [Gender::Male, Gender::Female] {
        let mut input = passenger(1, PassengerType::Adult, "1990-01-01");
        input.gender = gender;
        assert!(PassengerDraft::validate(
            vec![input],
            PassengerCounts::new(1, 0, 0).unwrap(),
            date("2030-05-10"),
            date("2026-08-31"),
        )
        .is_ok());
    }
}
