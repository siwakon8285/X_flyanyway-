use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use uuid::Uuid;

use crate::domain::{
    api_client::ApiClientScope,
    external_api::{
        AccessTokenHash, CredentialAdministrationError, CredentialDigest, CredentialMetadata,
        CredentialRevocationReason, ExternalAuthenticationError, ExternalPrincipal,
        ExternalTokenExchangeError, IssuedAccessToken, IssuedCredential, PlaintextAccessToken,
        PlaintextClientSecret,
    },
};

pub use crate::domain::external_api::ACCESS_TOKEN_TTL;

/// A deliberately low-cardinality description of an external authentication outcome.
///
/// Diagnostics are safe to attach to request spans: they never contain a client
/// secret, access token, credential digest, token hash, or request payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalAuthDiagnostic {
    Missing,
    Malformed,
    UnknownClient,
    SecretMismatch,
    Expired,
    Suspended,
    Revoked,
    ScopeDenied,
    Succeeded,
}

impl ExternalAuthDiagnostic {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Malformed => "malformed",
            Self::UnknownClient => "unknown_client",
            Self::SecretMismatch => "secret_mismatch",
            Self::Expired => "expired",
            Self::Suspended => "suspended",
            Self::Revoked => "revoked",
            Self::ScopeDenied => "scope_denied",
            Self::Succeeded => "succeeded",
        }
    }
}

/// Record only a safe authentication category on the request span.
pub fn record_external_auth_diagnostic(span: &tracing::Span, diagnostic: ExternalAuthDiagnostic) {
    let category = diagnostic.as_str();
    span.record("external_auth_diagnostic", category);
    tracing::debug!(parent: span, external_auth_diagnostic = category, "external authentication outcome");
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExternalAuthFlow {
    TokenExchange,
    Bearer,
}

impl ExternalAuthFlow {
    const fn as_str(self) -> &'static str {
        match self {
            Self::TokenExchange => "token_exchange",
            Self::Bearer => "bearer",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AnonymousExternalAuthDiagnostic {
    Missing,
    Malformed,
    UnknownClient,
}

impl AnonymousExternalAuthDiagnostic {
    const fn as_diagnostic(self) -> ExternalAuthDiagnostic {
        match self {
            Self::Missing => ExternalAuthDiagnostic::Missing,
            Self::Malformed => ExternalAuthDiagnostic::Malformed,
            Self::UnknownClient => ExternalAuthDiagnostic::UnknownClient,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttributedExternalAuthDiagnostic {
    SecretMismatch,
    Expired,
    Suspended,
    Revoked,
}

impl AttributedExternalAuthDiagnostic {
    const fn as_diagnostic(self) -> ExternalAuthDiagnostic {
        match self {
            Self::SecretMismatch => ExternalAuthDiagnostic::SecretMismatch,
            Self::Expired => ExternalAuthDiagnostic::Expired,
            Self::Suspended => ExternalAuthDiagnostic::Suspended,
            Self::Revoked => ExternalAuthDiagnostic::Revoked,
        }
    }
}

pub(crate) fn record_external_auth_anonymous(
    span: &tracing::Span,
    flow: ExternalAuthFlow,
    diagnostic: AnonymousExternalAuthDiagnostic,
) {
    let category = diagnostic.as_diagnostic().as_str();
    let flow = flow.as_str();
    span.record("external_auth_diagnostic", category);
    tracing::debug!(
        parent: span,
        external_auth_diagnostic = category,
        external_auth_flow = flow,
        "external authentication outcome"
    );
}

pub(crate) fn record_external_auth_attributed(
    span: &tracing::Span,
    flow: ExternalAuthFlow,
    diagnostic: AttributedExternalAuthDiagnostic,
    client_id: &ResolvedExternalClientId,
) {
    let category = diagnostic.as_diagnostic().as_str();
    let flow = flow.as_str();
    let client_id = client_id.as_str();
    span.record("external_auth_diagnostic", category);
    tracing::debug!(
        parent: span,
        external_auth_diagnostic = category,
        external_auth_flow = flow,
        external_client_id = client_id,
        "external authentication outcome"
    );
}

pub(crate) fn record_external_auth_succeeded(
    span: &tracing::Span,
    flow: ExternalAuthFlow,
    client_id: &ResolvedExternalClientId,
) {
    let category = ExternalAuthDiagnostic::Succeeded.as_str();
    let flow_name = flow.as_str();
    let client_id = client_id.as_str();
    span.record("external_auth_diagnostic", category);
    match flow {
        ExternalAuthFlow::TokenExchange => tracing::info!(
            parent: span,
            external_auth_diagnostic = category,
            external_auth_flow = flow_name,
            external_client_id = client_id,
            "external authentication outcome"
        ),
        ExternalAuthFlow::Bearer => tracing::debug!(
            parent: span,
            external_auth_diagnostic = category,
            external_auth_flow = flow_name,
            external_client_id = client_id,
            "external authentication outcome"
        ),
    }
}

pub(crate) fn record_external_scope_denied(
    span: &tracing::Span,
    client_id: &str,
    required: ApiClientScope,
) {
    let category = ExternalAuthDiagnostic::ScopeDenied.as_str();
    let scope = required.as_str();
    span.record("external_auth_diagnostic", category);
    tracing::debug!(
        parent: span,
        external_auth_diagnostic = category,
        external_client_id = client_id,
        external_required_scope = scope,
        "external authentication outcome"
    );
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedExternalClientId(String);

impl ResolvedExternalClientId {
    pub(crate) fn from_database(value: String) -> Self {
        Self(value)
    }

    fn from_principal(value: &str) -> Self {
        Self(value.to_owned())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenExchangeRejection {
    UnknownClient,
    SecretMismatch,
    CredentialUnavailable,
    Suspended,
    Revoked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenExchangeRepositoryOutcome {
    Issued {
        client_id: Option<ResolvedExternalClientId>,
    },
    Rejected {
        reason: TokenExchangeRejection,
        client_id: Option<ResolvedExternalClientId>,
    },
}

impl TokenExchangeRepositoryOutcome {
    pub(crate) fn issued(client_id: String) -> Self {
        Self::Issued {
            client_id: Some(ResolvedExternalClientId::from_database(client_id)),
        }
    }

    fn issued_without_attribution() -> Self {
        Self::Issued { client_id: None }
    }

    pub(crate) fn rejected(reason: TokenExchangeRejection, client_id: Option<String>) -> Self {
        Self::Rejected {
            reason,
            client_id: client_id.map(ResolvedExternalClientId::from_database),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BearerRejection {
    UnknownToken,
    Expired,
    Suspended,
    Revoked,
}

#[derive(Clone, Eq, PartialEq)]
pub enum BearerRepositoryOutcome {
    Authenticated {
        principal: ExternalPrincipal,
        client_id: ResolvedExternalClientId,
    },
    Rejected {
        reason: BearerRejection,
        client_id: Option<ResolvedExternalClientId>,
    },
}

impl BearerRepositoryOutcome {
    pub(crate) fn authenticated(principal: ExternalPrincipal) -> Self {
        let client_id = ResolvedExternalClientId::from_principal(principal.client_id());
        Self::Authenticated {
            principal,
            client_id,
        }
    }

    pub(crate) fn authenticated_from_database(
        principal: ExternalPrincipal,
        client_id: String,
    ) -> Self {
        Self::Authenticated {
            principal,
            client_id: ResolvedExternalClientId::from_database(client_id),
        }
    }

    pub(crate) fn rejected(reason: BearerRejection, client_id: Option<String>) -> Self {
        Self::Rejected {
            reason,
            client_id: client_id.map(ResolvedExternalClientId::from_database),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalScopeError {
    Missing,
}

pub fn require_scope(
    principal: &ExternalPrincipal,
    required: ApiClientScope,
) -> Result<(), ExternalScopeError> {
    principal
        .scopes()
        .contains(&required)
        .then_some(())
        .ok_or(ExternalScopeError::Missing)
}

#[async_trait]
pub trait ExternalAuthRepository: Send + Sync {
    async fn issue_credential(
        &self,
        client_id: &str,
        actor_staff_user_id: Uuid,
        expected_client_version: i64,
        digest: &CredentialDigest,
        issued_at: DateTime<Utc>,
    ) -> Result<CredentialMetadata, CredentialAdministrationError>;

    async fn revoke_credential(
        &self,
        client_id: &str,
        actor_staff_user_id: Uuid,
        expected_client_version: i64,
        reason: CredentialRevocationReason,
        revoked_at: DateTime<Utc>,
    ) -> Result<CredentialMetadata, CredentialAdministrationError>;

    async fn issue_access_token(
        &self,
        client_id: &str,
        digest: &CredentialDigest,
        token_hash: &AccessTokenHash,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), ExternalTokenExchangeError>;

    async fn issue_access_token_outcome(
        &self,
        client_id: &str,
        digest: &CredentialDigest,
        token_hash: &AccessTokenHash,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<TokenExchangeRepositoryOutcome, ExternalTokenExchangeError> {
        self.issue_access_token(client_id, digest, token_hash, issued_at, expires_at)
            .await
            .map(|_| TokenExchangeRepositoryOutcome::issued_without_attribution())
    }

    async fn authenticate_access_token(
        &self,
        token_hash: &AccessTokenHash,
        now: DateTime<Utc>,
    ) -> Result<ExternalPrincipal, ExternalAuthenticationError>;

    async fn authenticate_access_token_outcome(
        &self,
        token_hash: &AccessTokenHash,
        now: DateTime<Utc>,
    ) -> Result<BearerRepositoryOutcome, ExternalAuthenticationError> {
        self.authenticate_access_token(token_hash, now)
            .await
            .map(BearerRepositoryOutcome::authenticated)
    }
}

pub trait ExternalCredentialCrypto: Send + Sync {
    fn generate_client_secret(&self) -> PlaintextClientSecret;
    fn credential_digest(
        &self,
        public_client_id: &str,
        secret: &PlaintextClientSecret,
    ) -> CredentialDigest;
    fn dummy_credential_digest(&self, public_client_id: &str) -> CredentialDigest;
    fn digest_matches(&self, supplied: &CredentialDigest, stored: &CredentialDigest) -> bool;
    fn generate_access_token(&self) -> PlaintextAccessToken;
    fn access_token_hash(&self, token: &PlaintextAccessToken) -> AccessTokenHash;
}

#[derive(Clone)]
pub struct ApiClientCredentialService {
    repository: Arc<dyn ExternalAuthRepository>,
    crypto: Arc<dyn ExternalCredentialCrypto>,
}

impl ApiClientCredentialService {
    pub fn new(
        repository: Arc<dyn ExternalAuthRepository>,
        crypto: Arc<dyn ExternalCredentialCrypto>,
    ) -> Self {
        Self { repository, crypto }
    }

    pub async fn issue(
        &self,
        actor_staff_user_id: Uuid,
        client_id: &str,
        expected_client_version: i64,
        issued_at: DateTime<Utc>,
    ) -> Result<IssuedCredential, CredentialAdministrationError> {
        let secret = self.crypto.generate_client_secret();
        let digest = self.crypto.credential_digest(client_id, &secret);
        let metadata = self
            .repository
            .issue_credential(
                client_id,
                actor_staff_user_id,
                expected_client_version,
                &digest,
                issued_at,
            )
            .await?;
        Ok(IssuedCredential { secret, metadata })
    }

    pub async fn revoke(
        &self,
        actor_staff_user_id: Uuid,
        client_id: &str,
        expected_client_version: i64,
        reason: CredentialRevocationReason,
        revoked_at: DateTime<Utc>,
    ) -> Result<CredentialMetadata, CredentialAdministrationError> {
        self.repository
            .revoke_credential(
                client_id,
                actor_staff_user_id,
                expected_client_version,
                reason,
                revoked_at,
            )
            .await
    }
}

#[derive(Clone)]
pub struct ExternalAuthService {
    repository: Arc<dyn ExternalAuthRepository>,
    crypto: Arc<dyn ExternalCredentialCrypto>,
}

impl ExternalAuthService {
    pub fn new(
        repository: Arc<dyn ExternalAuthRepository>,
        crypto: Arc<dyn ExternalCredentialCrypto>,
    ) -> Self {
        Self { repository, crypto }
    }

    pub async fn exchange(
        &self,
        client_id: &str,
        client_secret_text: &str,
        now: DateTime<Utc>,
    ) -> Result<IssuedAccessToken, ExternalTokenExchangeError> {
        match self
            .exchange_with_outcome(client_id, client_secret_text, now)
            .await?
        {
            ExternalTokenExchangeDecision::Issued { issued, .. } => Ok(issued),
            ExternalTokenExchangeDecision::Rejected { .. } => {
                Err(ExternalTokenExchangeError::InvalidCredential)
            }
        }
    }

    pub(crate) async fn exchange_with_outcome(
        &self,
        client_id: &str,
        client_secret_text: &str,
        now: DateTime<Utc>,
    ) -> Result<ExternalTokenExchangeDecision, ExternalTokenExchangeError> {
        let secret = PlaintextClientSecret::parse_hex(client_secret_text)
            .map_err(|_| ExternalTokenExchangeError::InvalidCredential)?;
        let candidate = self.crypto.credential_digest(client_id, &secret);
        let _dummy = self.crypto.dummy_credential_digest(client_id);
        let token = self.crypto.generate_access_token();
        let token_hash = self.crypto.access_token_hash(&token);
        let expires_at = now
            .checked_add_signed(
                ChronoDuration::from_std(ACCESS_TOKEN_TTL)
                    .expect("900-second token TTL fits chrono duration"),
            )
            .ok_or(ExternalTokenExchangeError::Unavailable)?;
        let outcome = self
            .repository
            .issue_access_token_outcome(client_id, &candidate, &token_hash, now, expires_at)
            .await?;
        match outcome {
            TokenExchangeRepositoryOutcome::Issued { client_id } => {
                Ok(ExternalTokenExchangeDecision::Issued {
                    issued: IssuedAccessToken { token, expires_at },
                    client_id,
                })
            }
            TokenExchangeRepositoryOutcome::Rejected { reason, client_id } => {
                Ok(ExternalTokenExchangeDecision::Rejected { reason, client_id })
            }
        }
    }

    pub async fn authenticate_bearer(
        &self,
        authorization_header: &str,
        now: DateTime<Utc>,
    ) -> Result<ExternalPrincipal, ExternalAuthenticationError> {
        match self
            .authenticate_bearer_with_outcome(authorization_header, now)
            .await?
        {
            ExternalBearerDecision::Authenticated { principal, .. } => Ok(principal),
            ExternalBearerDecision::Rejected { .. } => {
                Err(ExternalAuthenticationError::InvalidToken)
            }
        }
    }

    pub(crate) async fn authenticate_bearer_with_outcome(
        &self,
        authorization_header: &str,
        now: DateTime<Utc>,
    ) -> Result<ExternalBearerDecision, ExternalAuthenticationError> {
        let token_text = authorization_header
            .strip_prefix("Bearer ")
            .ok_or(ExternalAuthenticationError::InvalidToken)?;
        let token = PlaintextAccessToken::parse_bearer_text(token_text)
            .map_err(|_| ExternalAuthenticationError::InvalidToken)?;
        let token_hash = self.crypto.access_token_hash(&token);
        match self
            .repository
            .authenticate_access_token_outcome(&token_hash, now)
            .await?
        {
            BearerRepositoryOutcome::Authenticated {
                principal,
                client_id,
            } => Ok(ExternalBearerDecision::Authenticated {
                principal,
                client_id,
            }),
            BearerRepositoryOutcome::Rejected { reason, client_id } => {
                Ok(ExternalBearerDecision::Rejected { reason, client_id })
            }
        }
    }
}

pub(crate) enum ExternalTokenExchangeDecision {
    Issued {
        issued: IssuedAccessToken,
        client_id: Option<ResolvedExternalClientId>,
    },
    Rejected {
        reason: TokenExchangeRejection,
        client_id: Option<ResolvedExternalClientId>,
    },
}

pub(crate) enum ExternalBearerDecision {
    Authenticated {
        principal: ExternalPrincipal,
        client_id: ResolvedExternalClientId,
    },
    Rejected {
        reason: BearerRejection,
        client_id: Option<ResolvedExternalClientId>,
    },
}
