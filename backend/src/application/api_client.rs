use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::api_client::{
    ApiClientScope, ApiClientStatus, CreateApiClientCommand, UpdateApiClientCommand,
    ValidatedCreateApiClient,
};

const CLIENT_ID_ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

#[derive(Clone, Debug, Default)]
pub struct ApiClientListFilter {
    pub search: Option<String>,
    pub status: Option<ApiClientStatus>,
    pub scope: Option<ApiClientScope>,
    pub limit: i64,
    pub offset: i64,
}

impl ApiClientListFilter {
    fn validate(mut self) -> Result<Self, ApiClientManagementError> {
        if !(1..=50).contains(&self.limit)
            || self.offset < 0
            || self.offset.checked_add(self.limit).is_none()
        {
            return Err(ApiClientManagementError::Validation);
        }
        self.search = self
            .search
            .map(|value| value.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|value| !value.is_empty());
        if self
            .search
            .as_ref()
            .is_some_and(|value| value.chars().count() > 100 || value.chars().any(char::is_control))
        {
            return Err(ApiClientManagementError::Validation);
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiClientRecord {
    pub client_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: ApiClientStatus,
    pub allowed_scopes: Vec<ApiClientScope>,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_by: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiClientPage {
    pub items: Vec<ApiClientRecord>,
    pub next_offset: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiClientAuditSnapshot {
    pub name: String,
    pub description: Option<String>,
    pub status: ApiClientStatus,
    pub allowed_scopes: Vec<ApiClientScope>,
}

impl From<&ApiClientRecord> for ApiClientAuditSnapshot {
    fn from(client: &ApiClientRecord) -> Self {
        Self {
            name: client.name.clone(),
            description: client.description.clone(),
            status: client.status,
            allowed_scopes: client.allowed_scopes.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiClientAuditAction {
    ClientCreated,
    ClientMetadataUpdated,
    ClientScopesUpdated,
    ClientActivated,
    ClientSuspended,
    ClientRevoked,
}

impl ApiClientAuditAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClientCreated => "CLIENT_CREATED",
            Self::ClientMetadataUpdated => "CLIENT_METADATA_UPDATED",
            Self::ClientScopesUpdated => "CLIENT_SCOPES_UPDATED",
            Self::ClientActivated => "CLIENT_ACTIVATED",
            Self::ClientSuspended => "CLIENT_SUSPENDED",
            Self::ClientRevoked => "CLIENT_REVOKED",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "CLIENT_CREATED" => Some(Self::ClientCreated),
            "CLIENT_METADATA_UPDATED" => Some(Self::ClientMetadataUpdated),
            "CLIENT_SCOPES_UPDATED" => Some(Self::ClientScopesUpdated),
            "CLIENT_ACTIVATED" => Some(Self::ClientActivated),
            "CLIENT_SUSPENDED" => Some(Self::ClientSuspended),
            "CLIENT_REVOKED" => Some(Self::ClientRevoked),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiClientAuditEntry {
    pub actor_email: String,
    pub action: ApiClientAuditAction,
    pub before: Option<ApiClientAuditSnapshot>,
    pub after: ApiClientAuditSnapshot,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiClientDetail {
    #[serde(flatten)]
    pub client: ApiClientRecord,
    pub audit: Vec<ApiClientAuditEntry>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiScopeDefinition {
    pub code: ApiClientScope,
    pub description: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApiClientManagementError {
    Validation,
    NotFound,
    Conflict,
    InvalidStatus,
    IdentityGeneration,
    Infrastructure,
}

#[async_trait]
pub trait ApiClientRepository: Send + Sync {
    async fn list(
        &self,
        filter: ApiClientListFilter,
    ) -> Result<ApiClientPage, ApiClientManagementError>;
    async fn detail(&self, client_id: &str) -> Result<ApiClientDetail, ApiClientManagementError>;
    async fn scope_catalog(&self) -> Result<Vec<ApiScopeDefinition>, ApiClientManagementError>;
    async fn create(
        &self,
        actor: Uuid,
        client_id: &str,
        command: &ValidatedCreateApiClient,
    ) -> Result<ApiClientRecord, ApiClientManagementError>;
    async fn update(
        &self,
        actor: Uuid,
        client_id: &str,
        command: UpdateApiClientCommand,
    ) -> Result<ApiClientRecord, ApiClientManagementError>;
    async fn transition(
        &self,
        actor: Uuid,
        client_id: &str,
        version: i64,
        status: ApiClientStatus,
    ) -> Result<ApiClientRecord, ApiClientManagementError>;
}

#[derive(Clone)]
pub struct ApiClientManagement {
    repository: Arc<dyn ApiClientRepository>,
}

impl ApiClientManagement {
    pub fn new(repository: Arc<dyn ApiClientRepository>) -> Self {
        Self { repository }
    }

    pub async fn list(
        &self,
        filter: ApiClientListFilter,
    ) -> Result<ApiClientPage, ApiClientManagementError> {
        self.repository.list(filter.validate()?).await
    }

    pub async fn detail(
        &self,
        client_id: &str,
    ) -> Result<ApiClientDetail, ApiClientManagementError> {
        validate_client_id(client_id)?;
        self.repository.detail(client_id).await
    }

    pub async fn scope_catalog(&self) -> Result<Vec<ApiScopeDefinition>, ApiClientManagementError> {
        self.repository.scope_catalog().await
    }

    pub async fn create(
        &self,
        actor: Uuid,
        command: CreateApiClientCommand,
    ) -> Result<ApiClientRecord, ApiClientManagementError> {
        let command = command
            .validate()
            .map_err(|_| ApiClientManagementError::Validation)?;
        for _ in 0..8 {
            let client_id = random_client_id();
            match self.repository.create(actor, &client_id, &command).await {
                Err(ApiClientManagementError::IdentityGeneration) => continue,
                result => return result,
            }
        }
        Err(ApiClientManagementError::IdentityGeneration)
    }

    pub async fn update(
        &self,
        actor: Uuid,
        client_id: &str,
        command: UpdateApiClientCommand,
    ) -> Result<ApiClientRecord, ApiClientManagementError> {
        validate_client_id(client_id)?;
        self.repository.update(actor, client_id, command).await
    }

    pub async fn transition(
        &self,
        actor: Uuid,
        client_id: &str,
        version: i64,
        status: ApiClientStatus,
    ) -> Result<ApiClientRecord, ApiClientManagementError> {
        validate_client_id(client_id)?;
        if version < 1 {
            return Err(ApiClientManagementError::Validation);
        }
        self.repository
            .transition(actor, client_id, version, status)
            .await
    }
}

fn validate_client_id(client_id: &str) -> Result<(), ApiClientManagementError> {
    let valid = client_id.len() == 19
        && client_id.starts_with("XFC")
        && client_id[3..]
            .bytes()
            .all(|byte| CLIENT_ID_ALPHABET.contains(&byte));
    valid
        .then_some(())
        .ok_or(ApiClientManagementError::Validation)
}

fn random_client_id() -> String {
    let mut bytes = [0_u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    let suffix: String = bytes
        .into_iter()
        .map(|byte| CLIENT_ID_ALPHABET[(byte & 31) as usize] as char)
        .collect();
    format!("XFC{suffix}")
}
