use serde::Serialize;

use crate::{
    application::flight::{FlightManagement, PublicFlight, PublicFlightFilter},
    domain::flight::FlightManagementError,
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCabinPrice {
    pub amount_thb: i64,
    pub cabin: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalFlight {
    pub flight_public_id: String,
    pub flight_number: String,
    pub origin_code: String,
    pub destination_code: String,
    pub departure_time: String,
    pub arrival_time: String,
    pub arrival_day_offset: u8,
    pub duration_minutes: u16,
    pub stops: String,
    pub aircraft_code: String,
    pub status: String,
    pub cabin_prices: Vec<ExternalCabinPrice>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExternalFlightPage {
    pub items: Vec<ExternalFlight>,
}

impl From<PublicFlight> for ExternalFlight {
    fn from(flight: PublicFlight) -> Self {
        Self {
            flight_public_id: flight.id,
            flight_number: flight.flight_number,
            origin_code: flight.origin_code,
            destination_code: flight.destination_code,
            departure_time: flight.departure_time,
            arrival_time: flight.arrival_time,
            arrival_day_offset: flight.arrival_day_offset,
            duration_minutes: flight.duration_minutes,
            stops: flight.stops,
            aircraft_code: flight.aircraft,
            status: flight.status,
            cabin_prices: flight
                .cabin_prices
                .into_iter()
                .map(|price| ExternalCabinPrice {
                    amount_thb: price.amount_thb,
                    cabin: price.cabin,
                })
                .collect(),
        }
    }
}

#[derive(Clone)]
pub struct ExternalFlightService {
    flights: FlightManagement,
}

impl ExternalFlightService {
    pub fn new(flights: FlightManagement) -> Self {
        Self { flights }
    }

    pub async fn search(
        &self,
        filter: PublicFlightFilter,
    ) -> Result<ExternalFlightPage, FlightManagementError> {
        let items = self
            .flights
            .search_public(filter)
            .await?
            .into_iter()
            .map(ExternalFlight::from)
            .collect();
        Ok(ExternalFlightPage { items })
    }

    pub async fn detail(
        &self,
        public_id: &str,
        departure: chrono::NaiveDate,
        cabin: crate::domain::value_objects::CabinClass,
    ) -> Result<ExternalFlight, FlightManagementError> {
        self.flights
            .public_detail(public_id, departure, cabin)
            .await
            .map(ExternalFlight::from)
    }
}
