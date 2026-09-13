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

    async fn authenticate_access_token(
        &self,
        token_hash: &AccessTokenHash,
        now: DateTime<Utc>,
    ) -> Result<ExternalPrincipal, ExternalAuthenticationError>;
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
        self.repository
            .issue_access_token(client_id, &candidate, &token_hash, now, expires_at)
            .await?;
        Ok(IssuedAccessToken { token, expires_at })
    }

    pub async fn authenticate_bearer(
        &self,
        authorization_header: &str,
        now: DateTime<Utc>,
    ) -> Result<ExternalPrincipal, ExternalAuthenticationError> {
        let token_text = authorization_header
            .strip_prefix("Bearer ")
            .ok_or(ExternalAuthenticationError::InvalidToken)?;
        let token = PlaintextAccessToken::parse_bearer_text(token_text)
            .map_err(|_| ExternalAuthenticationError::InvalidToken)?;
        let token_hash = self.crypto.access_token_hash(&token);
        self.repository
            .authenticate_access_token(&token_hash, now)
            .await
    }
}
