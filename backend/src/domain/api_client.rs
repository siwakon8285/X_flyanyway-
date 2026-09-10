use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiClientStatus {
    Active,
    Suspended,
    Revoked,
}

impl ApiClientStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Suspended => "SUSPENDED",
            Self::Revoked => "REVOKED",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "ACTIVE" => Some(Self::Active),
            "SUSPENDED" => Some(Self::Suspended),
            "REVOKED" => Some(Self::Revoked),
            _ => None,
        }
    }

    pub fn can_transition_to(self, next: Self, has_scope: bool) -> bool {
        matches!(
            (self, next),
            (Self::Active, Self::Suspended)
                | (Self::Active, Self::Revoked)
                | (Self::Suspended, Self::Revoked)
        ) || matches!((self, next), (Self::Suspended, Self::Active)) && has_scope
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ApiClientScope {
    #[serde(rename = "flights:read")]
    FlightsRead,
    #[serde(rename = "analytics:read")]
    AnalyticsRead,
}

impl ApiClientScope {
    pub const ALL: [Self; 2] = [Self::FlightsRead, Self::AnalyticsRead];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FlightsRead => "flights:read",
            Self::AnalyticsRead => "analytics:read",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.as_str() == value)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateApiClientCommand {
    pub name: String,
    pub description: Option<String>,
    pub status: ApiClientStatus,
    pub allowed_scopes: Vec<ApiClientScope>,
}

impl CreateApiClientCommand {
    pub fn validate(self) -> Result<ValidatedCreateApiClient, ApiClientValidationError> {
        let (name, description) = validate_metadata(self.name, self.description)?;
        let allowed_scopes = normalize_scopes(self.allowed_scopes);
        if self.status == ApiClientStatus::Active && allowed_scopes.is_empty() {
            return Err(ApiClientValidationError::ActiveRequiresScope);
        }
        if self.status == ApiClientStatus::Revoked {
            return Err(ApiClientValidationError::InitialStatus);
        }
        Ok(ValidatedCreateApiClient {
            name,
            description,
            status: self.status,
            allowed_scopes,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedCreateApiClient {
    pub name: String,
    pub description: Option<String>,
    pub status: ApiClientStatus,
    pub allowed_scopes: Vec<ApiClientScope>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateApiClientCommand {
    pub name: String,
    pub description: Option<String>,
    pub allowed_scopes: Vec<ApiClientScope>,
    pub version: i64,
}

impl UpdateApiClientCommand {
    pub fn validate(
        self,
        current_status: ApiClientStatus,
    ) -> Result<ValidatedUpdateApiClient, ApiClientValidationError> {
        if self.version < 1 {
            return Err(ApiClientValidationError::Version);
        }
        if current_status == ApiClientStatus::Revoked {
            return Err(ApiClientValidationError::Revoked);
        }
        let (name, description) = validate_metadata(self.name, self.description)?;
        let allowed_scopes = normalize_scopes(self.allowed_scopes);
        if current_status == ApiClientStatus::Active && allowed_scopes.is_empty() {
            return Err(ApiClientValidationError::ActiveRequiresScope);
        }
        Ok(ValidatedUpdateApiClient {
            name,
            description,
            allowed_scopes,
            version: self.version,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedUpdateApiClient {
    pub name: String,
    pub description: Option<String>,
    pub allowed_scopes: Vec<ApiClientScope>,
    pub version: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApiClientValidationError {
    Metadata,
    ActiveRequiresScope,
    InitialStatus,
    Version,
    Revoked,
}

fn validate_metadata(
    name: String,
    description: Option<String>,
) -> Result<(String, Option<String>), ApiClientValidationError> {
    let name = normalize_text(&name);
    if name.is_empty() || name.chars().count() > 100 || name.chars().any(char::is_control) {
        return Err(ApiClientValidationError::Metadata);
    }
    let description = description
        .map(|value| normalize_text(&value))
        .filter(|value| !value.is_empty());
    if description
        .as_ref()
        .is_some_and(|value| value.chars().count() > 500 || value.chars().any(char::is_control))
    {
        return Err(ApiClientValidationError::Metadata);
    }
    Ok((name, description))
}

fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_scopes(mut scopes: Vec<ApiClientScope>) -> Vec<ApiClientScope> {
    scopes.sort_unstable_by(|left, right| left.as_str().cmp(right.as_str()));
    scopes.dedup_by(|left, right| left.as_str() == right.as_str());
    scopes
}
