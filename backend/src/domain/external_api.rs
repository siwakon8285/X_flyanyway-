use std::{collections::BTreeSet, time::Duration};

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::api_client::ApiClientScope;

const EXTERNAL_SECRET_BYTES: usize = 32;
const EXTERNAL_SECRET_HEX_CHARS: usize = EXTERNAL_SECRET_BYTES * 2;
const ACCESS_TOKEN_PREFIX: &str = "xfa_v1_";

#[derive(Clone)]
pub struct ExternalApiCredentialPepper([u8; EXTERNAL_SECRET_BYTES]);

impl ExternalApiCredentialPepper {
    pub fn parse_hex(value: &str) -> Result<Self, PepperConfigError> {
        let bytes = decode_hex(value, false).map_err(|error| match error {
            HexDecodeError::Length => PepperConfigError::InvalidLength,
            HexDecodeError::Encoding => PepperConfigError::InvalidEncoding,
        })?;
        Ok(Self(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; EXTERNAL_SECRET_BYTES] {
        &self.0
    }
}

#[derive(Clone)]
pub struct PlaintextClientSecret([u8; EXTERNAL_SECRET_BYTES]);

impl PlaintextClientSecret {
    pub fn parse_hex(value: &str) -> Result<Self, CredentialFormatError> {
        decode_hex(value, true)
            .map(Self)
            .map_err(CredentialFormatError::from)
    }

    pub fn as_bytes(&self) -> &[u8; EXTERNAL_SECRET_BYTES] {
        &self.0
    }

    // The concrete Task 3 crypto adapter will construct this value in-crate.
    #[allow(dead_code)]
    pub(crate) fn from_bytes(bytes: [u8; EXTERNAL_SECRET_BYTES]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone)]
pub struct PlaintextAccessToken([u8; EXTERNAL_SECRET_BYTES]);

impl PlaintextAccessToken {
    pub fn parse_bearer_text(value: &str) -> Result<Self, CredentialFormatError> {
        let encoded = value
            .strip_prefix(ACCESS_TOKEN_PREFIX)
            .ok_or(CredentialFormatError::InvalidPrefix)?;
        decode_hex(encoded, true)
            .map(Self)
            .map_err(CredentialFormatError::from)
    }

    pub fn as_bytes(&self) -> &[u8; EXTERNAL_SECRET_BYTES] {
        &self.0
    }

    // The concrete Task 3 crypto adapter will construct this value in-crate.
    #[allow(dead_code)]
    pub(crate) fn from_bytes(bytes: [u8; EXTERNAL_SECRET_BYTES]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone)]
pub struct CredentialDigest([u8; EXTERNAL_SECRET_BYTES]);

impl CredentialDigest {
    pub fn as_bytes(&self) -> &[u8; EXTERNAL_SECRET_BYTES] {
        &self.0
    }

    // The concrete Task 3 crypto adapter will construct this value in-crate.
    #[allow(dead_code)]
    pub(crate) fn from_bytes(bytes: [u8; EXTERNAL_SECRET_BYTES]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone)]
pub struct AccessTokenHash([u8; EXTERNAL_SECRET_BYTES]);

impl AccessTokenHash {
    pub fn as_bytes(&self) -> &[u8; EXTERNAL_SECRET_BYTES] {
        &self.0
    }

    // The concrete Task 3 crypto adapter will construct this value in-crate.
    #[allow(dead_code)]
    pub(crate) fn from_bytes(bytes: [u8; EXTERNAL_SECRET_BYTES]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ExternalPrincipal {
    api_client_id: Uuid,
    client_id: String,
    scopes: BTreeSet<ApiClientScope>,
}

impl ExternalPrincipal {
    pub fn new(api_client_id: Uuid, client_id: String, scopes: BTreeSet<ApiClientScope>) -> Self {
        Self {
            api_client_id,
            client_id,
            scopes,
        }
    }

    pub fn api_client_id(&self) -> Uuid {
        self.api_client_id
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn scopes(&self) -> &BTreeSet<ApiClientScope> {
        &self.scopes
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct CredentialMetadata {
    pub credential_id: Uuid,
    pub issued_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocation_reason: Option<CredentialRevocationReason>,
}

#[derive(Clone)]
pub struct IssuedCredential {
    pub secret: PlaintextClientSecret,
    pub metadata: CredentialMetadata,
}

pub struct IssuedAccessToken {
    pub token: PlaintextAccessToken,
    pub expires_at: DateTime<Utc>,
}

pub const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(900);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialRevocationReason {
    AdminRequest,
    ClientSuspended,
    ClientRevoked,
    Replaced,
}

impl CredentialRevocationReason {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::AdminRequest => "ADMIN_REQUEST",
            Self::ClientSuspended => "CLIENT_SUSPENDED",
            Self::ClientRevoked => "CLIENT_REVOKED",
            Self::Replaced => "REPLACED",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CredentialFormatError {
    #[error("credential representation has an invalid length")]
    InvalidLength,
    #[error("credential representation has an invalid prefix")]
    InvalidPrefix,
    #[error("credential representation is not lowercase hexadecimal")]
    InvalidEncoding,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PepperConfigError {
    #[error("external credential pepper has an invalid length")]
    InvalidLength,
    #[error("external credential pepper is not hexadecimal")]
    InvalidEncoding,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CredentialAdministrationError {
    #[error("API client or credential was not found")]
    NotFound,
    #[error("API client version does not match")]
    VersionConflict,
    #[error("API client is revoked")]
    RevokedClient,
    #[error("API client already has a live credential")]
    LiveCredential,
    #[error("API client has no live credential")]
    NoLiveCredential,
    #[error("credential administration storage is unavailable")]
    Infrastructure,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ExternalTokenExchangeError {
    #[error("external credential is invalid")]
    InvalidCredential,
    #[error("external authentication storage is unavailable")]
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ExternalAuthenticationError {
    #[error("external access token is invalid")]
    InvalidToken,
    #[error("external authentication storage is unavailable")]
    Unavailable,
}

#[derive(Clone, Copy)]
enum HexDecodeError {
    Length,
    Encoding,
}

fn decode_hex(
    value: &str,
    lowercase_only: bool,
) -> Result<[u8; EXTERNAL_SECRET_BYTES], HexDecodeError> {
    if value.len() != EXTERNAL_SECRET_HEX_CHARS {
        return Err(HexDecodeError::Length);
    }
    if !value.bytes().all(|byte| {
        byte.is_ascii_digit()
            || (b'a'..=b'f').contains(&byte)
            || (!lowercase_only && (b'A'..=b'F').contains(&byte))
    }) {
        return Err(HexDecodeError::Encoding);
    }

    let mut bytes = [0_u8; EXTERNAL_SECRET_BYTES];
    hex::decode_to_slice(value, &mut bytes).map_err(|_| HexDecodeError::Encoding)?;
    Ok(bytes)
}

impl From<HexDecodeError> for CredentialFormatError {
    fn from(value: HexDecodeError) -> Self {
        match value {
            HexDecodeError::Length => Self::InvalidLength,
            HexDecodeError::Encoding => Self::InvalidEncoding,
        }
    }
}
