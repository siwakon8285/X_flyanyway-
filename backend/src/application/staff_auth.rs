use std::{sync::Arc, time::Duration};

use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::{
    domain::{
        repositories::{StaffAuthRepository, StaffAuthRepositoryError},
        staff::{RoleCode, StaffEmail, StaffPrincipal},
    },
    infrastructure::password::Argon2PasswordService,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvisionMode {
    Bootstrap,
    Create,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaffAdminCommand {
    mode: ProvisionMode,
    email: String,
    roles: Vec<RoleCode>,
}

impl StaffAdminCommand {
    pub fn parse<I, S>(arguments: I) -> Result<Self, StaffAuthError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut arguments = arguments.into_iter();
        let mode = match arguments.next().as_ref().map(AsRef::as_ref) {
            Some("bootstrap") => ProvisionMode::Bootstrap,
            Some("create") => ProvisionMode::Create,
            _ => return Err(StaffAuthError::InvalidProvisioningInput),
        };
        let mut email = None;
        let mut roles = std::collections::BTreeSet::new();
        while let Some(argument) = arguments.next() {
            match argument.as_ref() {
                "--email" if email.is_none() => {
                    let value = arguments
                        .next()
                        .ok_or(StaffAuthError::InvalidProvisioningInput)?;
                    email = Some(
                        StaffEmail::parse(value.as_ref())
                            .map_err(|_| StaffAuthError::InvalidProvisioningInput)?
                            .as_str()
                            .to_owned(),
                    );
                }
                "--role" => {
                    let value = arguments
                        .next()
                        .ok_or(StaffAuthError::InvalidProvisioningInput)?;
                    roles.insert(
                        value
                            .as_ref()
                            .parse()
                            .map_err(|_| StaffAuthError::UnknownRole)?,
                    );
                }
                _ => return Err(StaffAuthError::InvalidProvisioningInput),
            }
        }
        let email = email.ok_or(StaffAuthError::InvalidProvisioningInput)?;
        if roles.is_empty() {
            return Err(StaffAuthError::InvalidProvisioningInput);
        }
        Ok(Self {
            mode,
            email,
            roles: roles.into_iter().collect(),
        })
    }

    pub fn mode(&self) -> ProvisionMode {
        self.mode
    }
    pub fn email(&self) -> &str {
        &self.email
    }
    pub fn roles(&self) -> &[RoleCode] {
        &self.roles
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaffLogin {
    pub token: String,
    pub principal: StaffPrincipal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaffAuthError {
    InvalidCredentials,
    Throttled,
    Unauthenticated,
    InvalidProvisioningInput,
    DuplicateEmail,
    BootstrapAlreadyCompleted,
    BootstrapRequired,
    UnknownRole,
    Infrastructure,
}

impl std::fmt::Display for StaffAuthError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidCredentials => "email or password is incorrect",
            Self::Throttled => "login is temporarily throttled",
            Self::Unauthenticated => "staff authentication is required",
            Self::InvalidProvisioningInput => "staff provisioning input is invalid",
            Self::DuplicateEmail => "a staff account already uses that email",
            Self::BootstrapAlreadyCompleted => "staff bootstrap has already been completed",
            Self::BootstrapRequired => "bootstrap the first staff account before creating another",
            Self::UnknownRole => "one or more staff roles are not canonical",
            Self::Infrastructure => "staff authentication storage is unavailable",
        })
    }
}

impl std::error::Error for StaffAuthError {}

#[derive(Clone)]
pub struct StaffAuthService {
    repository: Arc<dyn StaffAuthRepository>,
    passwords: Argon2PasswordService,
    session_lifetime: Duration,
    dummy_hash: String,
}

impl StaffAuthService {
    pub fn new(
        repository: Arc<dyn StaffAuthRepository>,
        passwords: Argon2PasswordService,
        session_lifetime: Duration,
    ) -> Result<Self, StaffAuthError> {
        let dummy_hash = passwords
            .hash("x-fly-dummy-password-that-is-never-an-account")
            .map_err(|_| StaffAuthError::Infrastructure)?;
        Ok(Self {
            repository,
            passwords,
            session_lifetime,
            dummy_hash,
        })
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<StaffLogin, StaffAuthError> {
        let normalized_identifier = email.trim().to_lowercase();
        let identifier_hash = hash_bytes(normalized_identifier.as_bytes());
        let blocked = self
            .repository
            .login_is_blocked(identifier_hash)
            .await
            .map_err(StaffAuthError::from)?;
        let credential = match StaffEmail::parse(email) {
            Ok(email) => self
                .repository
                .credential_for_email(email.as_str())
                .await
                .map_err(StaffAuthError::from)?,
            Err(_) => None,
        };
        let encoded = credential
            .as_ref()
            .map(|value| value.password_hash.clone())
            .unwrap_or_else(|| self.dummy_hash.clone());
        let password_value = password.to_owned();
        let passwords = self.passwords.clone();
        let verified =
            tokio::task::spawn_blocking(move || passwords.verify(&password_value, &encoded))
                .await
                .map_err(|_| StaffAuthError::Infrastructure)?;

        if blocked {
            return Err(StaffAuthError::Throttled);
        }
        let Some(credential) = credential.filter(|value| value.active && verified) else {
            let now_blocked = self
                .repository
                .record_login_failure(identifier_hash)
                .await
                .map_err(StaffAuthError::from)?;
            return Err(if now_blocked {
                StaffAuthError::Throttled
            } else {
                StaffAuthError::InvalidCredentials
            });
        };

        self.repository
            .clear_login_failures(identifier_hash)
            .await
            .map_err(StaffAuthError::from)?;
        if self.passwords.needs_rehash(&credential.password_hash) {
            let password_value = password.to_owned();
            let passwords = self.passwords.clone();
            let hash = tokio::task::spawn_blocking(move || passwords.hash(&password_value))
                .await
                .map_err(|_| StaffAuthError::Infrastructure)?
                .map_err(|_| StaffAuthError::Infrastructure)?;
            self.repository
                .update_password_hash(credential.id, &hash)
                .await
                .map_err(StaffAuthError::from)?;
        }

        let mut raw_token = [0_u8; 32];
        rand::rng().fill_bytes(&mut raw_token);
        let token_hash = hash_bytes(&raw_token);
        let principal = self
            .repository
            .create_session(credential.id, token_hash, self.session_lifetime)
            .await
            .map_err(StaffAuthError::from)?;
        Ok(StaffLogin {
            token: hex::encode(raw_token),
            principal,
        })
    }

    pub async fn authenticate(&self, token: &str) -> Result<StaffPrincipal, StaffAuthError> {
        let raw = decode_token(token)?;
        self.repository
            .authenticate_session(hash_bytes(&raw))
            .await
            .map_err(StaffAuthError::from)?
            .ok_or(StaffAuthError::Unauthenticated)
    }

    pub async fn logout(&self, token: &str) -> Result<(), StaffAuthError> {
        if let Ok(raw) = decode_token(token) {
            self.repository
                .revoke_session(hash_bytes(&raw))
                .await
                .map_err(StaffAuthError::from)?;
        }
        Ok(())
    }

    pub async fn provision(
        &self,
        mode: ProvisionMode,
        email: &str,
        password: &str,
        roles: &[RoleCode],
    ) -> Result<(), StaffAuthError> {
        let email =
            StaffEmail::parse(email).map_err(|_| StaffAuthError::InvalidProvisioningInput)?;
        let password_length = password.chars().count();
        if !(14..=128).contains(&password_length) || roles.is_empty() {
            return Err(StaffAuthError::InvalidProvisioningInput);
        }
        let password_value = password.to_owned();
        let passwords = self.passwords.clone();
        let hash = tokio::task::spawn_blocking(move || passwords.hash(&password_value))
            .await
            .map_err(|_| StaffAuthError::Infrastructure)?
            .map_err(|_| StaffAuthError::Infrastructure)?;
        self.repository
            .provision_staff(
                matches!(mode, ProvisionMode::Bootstrap),
                email.as_str(),
                &hash,
                roles,
            )
            .await
            .map_err(StaffAuthError::from)
    }
}

fn decode_token(token: &str) -> Result<Vec<u8>, StaffAuthError> {
    let raw = hex::decode(token).map_err(|_| StaffAuthError::Unauthenticated)?;
    (raw.len() == 32)
        .then_some(raw)
        .ok_or(StaffAuthError::Unauthenticated)
}

fn hash_bytes(value: &[u8]) -> [u8; 32] {
    Sha256::digest(value).into()
}

impl From<StaffAuthRepositoryError> for StaffAuthError {
    fn from(value: StaffAuthRepositoryError) -> Self {
        match value {
            StaffAuthRepositoryError::DuplicateEmail => Self::DuplicateEmail,
            StaffAuthRepositoryError::BootstrapAlreadyCompleted => Self::BootstrapAlreadyCompleted,
            StaffAuthRepositoryError::BootstrapRequired => Self::BootstrapRequired,
            StaffAuthRepositoryError::UnknownRole => Self::UnknownRole,
            StaffAuthRepositoryError::InconsistentState => Self::Infrastructure,
            StaffAuthRepositoryError::Infrastructure(_) => Self::Infrastructure,
        }
    }
}
