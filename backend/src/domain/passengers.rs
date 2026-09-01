use std::{collections::HashSet, sync::LazyLock};

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::{entities::SeatHold, value_objects::PassengerCounts};

static COUNTRY_CODES: LazyLock<HashSet<String>> = LazyLock::new(|| {
    serde_json::from_str::<Vec<String>>(include_str!("../../../shared/country-codes.json"))
        .expect("shared country codes must be valid JSON")
        .into_iter()
        .collect()
});

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PassengerType {
    Adult,
    Child,
    Infant,
}

impl PassengerType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Adult => "ADULT",
            Self::Child => "CHILD",
            Self::Infant => "INFANT",
        }
    }

    pub fn parse_database(value: &str) -> Option<Self> {
        match value {
            "ADULT" => Some(Self::Adult),
            "CHILD" => Some(Self::Child),
            "INFANT" => Some(Self::Infant),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Title {
    Mr,
    Ms,
}

impl Title {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mr => "MR",
            Self::Ms => "MS",
        }
    }

    pub fn parse_database(value: &str) -> Option<Self> {
        match value {
            "MR" => Some(Self::Mr),
            "MS" => Some(Self::Ms),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Gender {
    Male,
    Female,
    Unspecified,
}

impl Gender {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Male => "MALE",
            Self::Female => "FEMALE",
            Self::Unspecified => "UNSPECIFIED",
        }
    }

    pub fn parse_database(value: &str) -> Option<Self> {
        match value {
            "MALE" => Some(Self::Male),
            "FEMALE" => Some(Self::Female),
            "UNSPECIFIED" => Some(Self::Unspecified),
            _ => None,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyContact {
    pub name: String,
    pub relationship: String,
    pub phone_country_code: String,
    pub phone_number: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyContactInput {
    pub name: String,
    pub relationship: String,
    pub phone_country_code: String,
    pub phone_number: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassengerInput {
    pub ordinal: u8,
    pub passenger_type: PassengerType,
    pub title: Title,
    pub given_name: String,
    pub middle_name: Option<String>,
    pub family_name: String,
    pub date_of_birth: NaiveDate,
    pub gender: Gender,
    pub nationality_code: String,
    pub passport_number: String,
    pub passport_issuing_country_code: String,
    pub email: String,
    pub phone_country_code: String,
    pub phone_number: String,
    pub emergency_contact: Option<EmergencyContactInput>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Passenger {
    pub ordinal: u8,
    pub passenger_type: PassengerType,
    pub title: Title,
    pub given_name: String,
    pub middle_name: Option<String>,
    pub family_name: String,
    pub date_of_birth: NaiveDate,
    pub gender: Gender,
    pub nationality_code: String,
    pub passport_number: String,
    pub passport_issuing_country_code: String,
    pub email: String,
    pub phone_country_code: String,
    pub phone_number: String,
    pub emergency_contact: Option<EmergencyContact>,
}

pub struct PassengerDraft {
    passengers: Vec<Passenger>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassengerSlot {
    pub ordinal: u8,
    pub passenger_type: PassengerType,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassengerContext {
    pub hold: SeatHold,
    pub expected_passengers: Vec<PassengerSlot>,
    pub passengers: Vec<Passenger>,
    pub ready_to_continue: bool,
}

impl PassengerDraft {
    pub fn validate(
        inputs: Vec<PassengerInput>,
        counts: PassengerCounts,
        departure_date: NaiveDate,
        today: NaiveDate,
    ) -> Result<Self, PassengerDraftError> {
        let expected_types = expected_passenger_types(counts);
        if inputs.len() != expected_types.len() {
            return Err(PassengerDraftError::CountMismatch);
        }
        if inputs.iter().enumerate().any(|(index, passenger)| {
            passenger.ordinal as usize != index + 1
                || passenger.passenger_type != expected_types[index]
        }) {
            return Err(PassengerDraftError::TypeMismatch);
        }

        let mut errors = Vec::new();
        let mut passengers = Vec::with_capacity(inputs.len());
        let mut passports = HashSet::new();

        for input in inputs {
            let ordinal = input.ordinal;
            let given_name = normalize_name(&input.given_name);
            validate_name(ordinal, "givenName", &given_name, &mut errors);
            let middle_name = normalize_optional_name(input.middle_name);
            if let Some(value) = &middle_name {
                validate_name(ordinal, "middleName", value, &mut errors);
            }
            let family_name = normalize_name(&input.family_name);
            validate_name(ordinal, "familyName", &family_name, &mut errors);

            let future_birth = input.date_of_birth > today;
            if future_birth {
                errors.push(field_error(
                    ordinal,
                    "dateOfBirth",
                    PassengerValidationCode::DateOfBirthFuture,
                ));
            } else if input.date_of_birth > departure_date {
                errors.push(field_error(
                    ordinal,
                    "dateOfBirth",
                    PassengerValidationCode::InvalidDate,
                ));
            } else if passenger_type_at(input.date_of_birth, departure_date) != input.passenger_type
            {
                errors.push(field_error(
                    ordinal,
                    "dateOfBirth",
                    PassengerValidationCode::AgeCategoryMismatch,
                ));
            }

            let nationality_code = normalize_country_code(&input.nationality_code);
            if !is_country_code(&nationality_code) {
                errors.push(field_error(
                    ordinal,
                    "nationalityCode",
                    PassengerValidationCode::InvalidCountry,
                ));
            }
            let passport_issuing_country_code =
                normalize_country_code(&input.passport_issuing_country_code);
            if !is_country_code(&passport_issuing_country_code) {
                errors.push(field_error(
                    ordinal,
                    "passportIssuingCountryCode",
                    PassengerValidationCode::InvalidCountry,
                ));
            }

            let passport_number = normalize_passport_number(&input.passport_number);
            if !(3..=20).contains(&passport_number.len())
                || !passport_number
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
            {
                errors.push(field_error(
                    ordinal,
                    "passportNumber",
                    PassengerValidationCode::InvalidPassport,
                ));
            } else if !passports.insert(passport_number.clone()) {
                errors.push(field_error(
                    ordinal,
                    "passportNumber",
                    PassengerValidationCode::DuplicatePassport,
                ));
            }

            let email = input.email.trim().to_owned();
            if !is_valid_email(&email) {
                errors.push(field_error(
                    ordinal,
                    "email",
                    PassengerValidationCode::InvalidEmail,
                ));
            }
            let phone_input_valid =
                is_valid_phone_input(&input.phone_country_code, &input.phone_number);
            let phone_country_code = normalize_phone_country_code(&input.phone_country_code);
            let phone_number = normalize_phone_number(&input.phone_number);
            if phone_country_code == "+" {
                errors.push(field_error(
                    ordinal,
                    "phoneCountryCode",
                    PassengerValidationCode::Required,
                ));
            }
            if phone_number.is_empty() {
                errors.push(field_error(
                    ordinal,
                    "phoneNumber",
                    PassengerValidationCode::Required,
                ));
            } else if phone_country_code != "+"
                && (!phone_input_valid || !is_valid_phone(&phone_country_code, &phone_number))
            {
                errors.push(field_error(
                    ordinal,
                    "phoneNumber",
                    PassengerValidationCode::InvalidPhone,
                ));
            }

            let emergency_contact = input.emergency_contact.map(|contact| {
                let phone_input_valid =
                    is_valid_phone_input(&contact.phone_country_code, &contact.phone_number);
                let name = normalize_name(&contact.name);
                let relationship = normalize_name(&contact.relationship);
                let phone_country_code = normalize_phone_country_code(&contact.phone_country_code);
                let phone_number = normalize_phone_number(&contact.phone_number);
                if !is_valid_name(&name)
                    || !is_valid_name(&relationship)
                    || !phone_input_valid
                    || !is_valid_phone(&phone_country_code, &phone_number)
                {
                    errors.push(field_error(
                        ordinal,
                        "emergencyContact",
                        PassengerValidationCode::EmergencyContactIncomplete,
                    ));
                }
                EmergencyContact {
                    name,
                    relationship,
                    phone_country_code,
                    phone_number,
                }
            });

            passengers.push(Passenger {
                ordinal,
                passenger_type: input.passenger_type,
                title: input.title,
                given_name,
                middle_name,
                family_name,
                date_of_birth: input.date_of_birth,
                gender: input.gender,
                nationality_code,
                passport_number,
                passport_issuing_country_code,
                email,
                phone_country_code,
                phone_number,
                emergency_contact,
            });
        }

        if errors.is_empty() {
            Ok(Self { passengers })
        } else {
            Err(PassengerDraftError::Fields(errors))
        }
    }

    pub fn passengers(&self) -> &[Passenger] {
        &self.passengers
    }

    pub fn into_passengers(self) -> Vec<Passenger> {
        self.passengers
    }
}

#[derive(Debug, Error)]
pub enum PassengerDraftError {
    #[error("passenger count does not match the active hold")]
    CountMismatch,
    #[error("passenger types do not match the active hold")]
    TypeMismatch,
    #[error("passenger fields are invalid")]
    Fields(Vec<PassengerFieldError>),
}

impl PassengerDraftError {
    pub fn field_errors(&self) -> &[PassengerFieldError] {
        match self {
            Self::Fields(errors) => errors,
            Self::CountMismatch | Self::TypeMismatch => &[],
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassengerFieldError {
    pub passenger: u8,
    pub field: &'static str,
    pub code: PassengerValidationCode,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PassengerValidationCode {
    Required,
    InvalidName,
    InvalidDate,
    DateOfBirthFuture,
    AgeCategoryMismatch,
    InvalidCountry,
    InvalidPassport,
    DuplicatePassport,
    InvalidEmail,
    InvalidPhone,
    EmergencyContactIncomplete,
}

impl PassengerValidationCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Required => "REQUIRED",
            Self::InvalidName => "INVALID_NAME",
            Self::InvalidDate => "INVALID_DATE",
            Self::DateOfBirthFuture => "DATE_OF_BIRTH_FUTURE",
            Self::AgeCategoryMismatch => "AGE_CATEGORY_MISMATCH",
            Self::InvalidCountry => "INVALID_COUNTRY",
            Self::InvalidPassport => "INVALID_PASSPORT",
            Self::DuplicatePassport => "DUPLICATE_PASSPORT",
            Self::InvalidEmail => "INVALID_EMAIL",
            Self::InvalidPhone => "INVALID_PHONE",
            Self::EmergencyContactIncomplete => "EMERGENCY_CONTACT_INCOMPLETE",
        }
    }
}

pub fn expected_passenger_types(counts: PassengerCounts) -> Vec<PassengerType> {
    std::iter::repeat_n(PassengerType::Adult, counts.adults() as usize)
        .chain(std::iter::repeat_n(
            PassengerType::Child,
            counts.children() as usize,
        ))
        .chain(std::iter::repeat_n(
            PassengerType::Infant,
            counts.infants() as usize,
        ))
        .collect()
}

pub fn expected_passenger_slots(counts: PassengerCounts) -> Vec<PassengerSlot> {
    expected_passenger_types(counts)
        .into_iter()
        .enumerate()
        .map(|(index, passenger_type)| PassengerSlot {
            ordinal: (index + 1) as u8,
            passenger_type,
        })
        .collect()
}

fn passenger_type_at(date_of_birth: NaiveDate, departure_date: NaiveDate) -> PassengerType {
    let mut age = departure_date.year() - date_of_birth.year();
    if (departure_date.month(), departure_date.day()) < (date_of_birth.month(), date_of_birth.day())
    {
        age -= 1;
    }
    if age >= 12 {
        PassengerType::Adult
    } else if age >= 2 {
        PassengerType::Child
    } else {
        PassengerType::Infant
    }
}

fn normalize_name(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_optional_name(value: Option<String>) -> Option<String> {
    value
        .map(|value| normalize_name(&value))
        .filter(|value| !value.is_empty())
}

fn is_valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= 100
        && value.chars().all(|character| {
            character.is_alphabetic()
                || character.is_whitespace()
                || matches!(character, '-' | '\'' | '’' | '.')
        })
}

fn validate_name(
    passenger: u8,
    field: &'static str,
    value: &str,
    errors: &mut Vec<PassengerFieldError>,
) {
    if value.is_empty() {
        errors.push(field_error(
            passenger,
            field,
            PassengerValidationCode::Required,
        ));
    } else if !is_valid_name(value) {
        errors.push(field_error(
            passenger,
            field,
            PassengerValidationCode::InvalidName,
        ));
    }
}

fn normalize_country_code(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

fn is_country_code(value: &str) -> bool {
    COUNTRY_CODES.contains(value)
}

fn normalize_passport_number(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_ascii_whitespace() && *character != '-')
        .collect::<String>()
        .to_ascii_uppercase()
}

fn is_valid_email(value: &str) -> bool {
    if value.len() > 254 || value.chars().any(char::is_whitespace) {
        return false;
    }
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && local.len() <= 64
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && domain.contains('.')
        && !domain.contains('@')
}

fn normalize_phone_country_code(value: &str) -> String {
    let digits: String = value.chars().filter(char::is_ascii_digit).collect();
    format!("+{digits}")
}

fn normalize_phone_number(value: &str) -> String {
    value.chars().filter(char::is_ascii_digit).collect()
}

fn is_valid_phone_input(country_code: &str, phone_number: &str) -> bool {
    let country_code = country_code.trim();
    let country_body = country_code.strip_prefix('+').unwrap_or(country_code);
    !country_body.is_empty()
        && country_body.chars().all(|character| {
            character.is_ascii_digit() || character.is_ascii_whitespace() || character == '-'
        })
        && !phone_number.is_empty()
        && phone_number.chars().all(|character| {
            character.is_ascii_digit()
                || character.is_ascii_whitespace()
                || matches!(character, '-' | '(' | ')' | '.')
        })
}

fn is_valid_phone(country_code: &str, phone_number: &str) -> bool {
    let country_digits = country_code.strip_prefix('+').unwrap_or_default();
    let total_length = country_digits.len() + phone_number.len();
    (1..=3).contains(&country_digits.len())
        && !country_digits.starts_with('0')
        && phone_number.len() >= 4
        && (7..=15).contains(&total_length)
}

fn field_error(
    passenger: u8,
    field: &'static str,
    code: PassengerValidationCode,
) -> PassengerFieldError {
    PassengerFieldError {
        passenger,
        field,
        code,
    }
}
