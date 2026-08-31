use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::domain::DomainError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CabinClass {
    Economy,
    PremiumEconomy,
    Business,
    First,
}

impl CabinClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Economy => "economy",
            Self::PremiumEconomy => "premium-economy",
            Self::Business => "business",
            Self::First => "first",
        }
    }
}

impl fmt::Display for CabinClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for CabinClass {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "economy" => Ok(Self::Economy),
            "premium-economy" => Ok(Self::PremiumEconomy),
            "business" => Ok(Self::Business),
            "first" => Ok(Self::First),
            _ => Err(DomainError::InvalidCabin),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassengerCounts {
    adults: u8,
    children: u8,
    infants: u8,
}

impl PassengerCounts {
    pub fn new(adults: u8, children: u8, infants: u8) -> Result<Self, DomainError> {
        if adults == 0 {
            return Err(DomainError::AdultRequired);
        }
        if infants > adults {
            return Err(DomainError::TooManyInfants);
        }
        if u16::from(adults) + u16::from(children) > 9 {
            return Err(DomainError::PartyTooLarge);
        }

        Ok(Self {
            adults,
            children,
            infants,
        })
    }

    pub const fn adults(self) -> u8 {
        self.adults
    }

    pub const fn children(self) -> u8 {
        self.children
    }

    pub const fn infants(self) -> u8 {
        self.infants
    }

    pub const fn required_seats(self) -> usize {
        self.adults as usize + self.children as usize
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct SeatNumber(String);

impl SeatNumber {
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        let normalized = value.trim().to_ascii_uppercase();
        let digit_count = normalized.chars().take_while(char::is_ascii_digit).count();
        let valid = digit_count > 0
            && digit_count < normalized.len()
            && !normalized.starts_with('0')
            && normalized[digit_count..]
                .chars()
                .all(|character| character.is_ascii_uppercase());

        valid
            .then_some(Self(normalized))
            .ok_or(DomainError::InvalidSeatNumber)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SeatNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
