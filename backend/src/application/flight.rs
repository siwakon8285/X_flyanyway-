use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::flight::{FlightCommand, FlightManagementError, FlightRecord, FlightStatus};
use crate::domain::value_objects::CabinClass;

#[derive(Clone, Debug, Default)]
pub struct FlightListFilter {
    pub search: Option<String>,
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub date: Option<NaiveDate>,
    pub status: Option<FlightStatus>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlightPage {
    pub items: Vec<FlightRecord>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlightAuditEntry {
    pub id: Uuid,
    pub actor_email: String,
    pub action: String,
    pub before_state: Option<serde_json::Value>,
    pub after_state: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlightDetail {
    #[serde(flatten)]
    pub flight: FlightRecord,
    pub audit: Vec<FlightAuditEntry>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AirportReference {
    pub code: String,
    pub name: String,
    pub city: String,
    pub country_code: String,
    pub country_name: String,
    pub time_zone: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlightReferenceData {
    pub airports: Vec<AirportReference>,
    pub aircraft: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PublicFlightFilter {
    pub origin: String,
    pub destination: String,
    pub departure: NaiveDate,
    pub cabin: CabinClass,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicCabinPrice {
    pub amount_thb: i64,
    pub cabin: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicFlight {
    pub id: String,
    pub flight_number: String,
    pub origin_code: String,
    pub destination_code: String,
    pub departure_time: String,
    pub arrival_time: String,
    pub arrival_day_offset: u8,
    pub duration_minutes: u16,
    pub stops: String,
    pub aircraft: String,
    pub status: String,
    pub recommended_rank: usize,
    pub cabin_prices: Vec<PublicCabinPrice>,
}

#[async_trait]
pub trait FlightRepository: Send + Sync {
    async fn list(&self, filter: FlightListFilter) -> Result<FlightPage, FlightManagementError>;
    async fn detail(&self, id: Uuid) -> Result<FlightDetail, FlightManagementError>;
    async fn reference_data(&self) -> Result<FlightReferenceData, FlightManagementError>;
    async fn search_public(
        &self,
        filter: PublicFlightFilter,
    ) -> Result<Vec<PublicFlight>, FlightManagementError>;
    async fn public_detail(
        &self,
        public_id: &str,
        departure: NaiveDate,
        cabin: CabinClass,
    ) -> Result<PublicFlight, FlightManagementError>;
    async fn create(
        &self,
        actor: Uuid,
        command: FlightCommand,
    ) -> Result<FlightRecord, FlightManagementError>;
    async fn update(
        &self,
        actor: Uuid,
        id: Uuid,
        version: i64,
        command: FlightCommand,
    ) -> Result<FlightRecord, FlightManagementError>;
    async fn cancel(
        &self,
        actor: Uuid,
        id: Uuid,
        version: i64,
    ) -> Result<FlightRecord, FlightManagementError>;
}

type ResultFlight = Result<FlightRecord, FlightManagementError>;

#[derive(Clone)]
pub struct FlightManagement {
    repository: Arc<dyn FlightRepository>,
}

impl FlightManagement {
    pub fn new(repository: Arc<dyn FlightRepository>) -> Self {
        Self { repository }
    }
    pub async fn list(
        &self,
        filter: FlightListFilter,
    ) -> Result<FlightPage, FlightManagementError> {
        self.repository.list(filter).await
    }
    pub async fn detail(&self, id: Uuid) -> Result<FlightDetail, FlightManagementError> {
        self.repository.detail(id).await
    }
    pub async fn reference_data(&self) -> Result<FlightReferenceData, FlightManagementError> {
        self.repository.reference_data().await
    }
    pub async fn search_public(
        &self,
        filter: PublicFlightFilter,
    ) -> Result<Vec<PublicFlight>, FlightManagementError> {
        self.repository.search_public(filter).await
    }
    pub async fn public_detail(
        &self,
        public_id: &str,
        departure: NaiveDate,
        cabin: CabinClass,
    ) -> Result<PublicFlight, FlightManagementError> {
        self.repository
            .public_detail(public_id, departure, cabin)
            .await
    }
    pub async fn create(&self, actor: Uuid, command: FlightCommand) -> ResultFlight {
        self.repository.create(actor, command).await
    }
    pub async fn update(
        &self,
        actor: Uuid,
        id: Uuid,
        version: i64,
        command: FlightCommand,
    ) -> ResultFlight {
        self.repository.update(actor, id, version, command).await
    }
    pub async fn cancel(&self, actor: Uuid, id: Uuid, version: i64) -> ResultFlight {
        self.repository.cancel(actor, id, version).await
    }
}
