use std::{sync::Arc, time::Duration};

use rand::RngCore;
use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::{
    domain::{
        repositories::{StaffAuthRepository, StaffAuthRepositoryError},
        staff::{PermissionCode, RoleCode, StaffEmail, StaffPrincipal},
    },
    infrastructure::password::Argon2PasswordService,
};

pub const MIN_CONCURRENT_PASSWORD_VERIFICATIONS: usize = 1;
pub const MAX_CONCURRENT_PASSWORD_VERIFICATIONS: usize = 4;
pub const DEFAULT_MAX_CONCURRENT_PASSWORD_VERIFICATIONS: usize =
    MAX_CONCURRENT_PASSWORD_VERIFICATIONS;

pub(crate) fn is_valid_password_verification_limit(value: usize) -> bool {
    (MIN_CONCURRENT_PASSWORD_VERIFICATIONS..=MAX_CONCURRENT_PASSWORD_VERIFICATIONS).contains(&value)
}

trait PasswordOperations: Send + Sync {
    fn hash(&self, password: &str) -> Result<String, ()>;
    fn verify(&self, password: &str, encoded: &str) -> Result<bool, ()>;
    fn needs_rehash(&self, encoded: &str) -> bool;
}

#[derive(Clone)]
struct Argon2PasswordOperations {
    service: Argon2PasswordService,
}

impl PasswordOperations for Argon2PasswordOperations {
    fn hash(&self, password: &str) -> Result<String, ()> {
        self.service.hash(password).map_err(|_| ())
    }

    fn verify(&self, password: &str, encoded: &str) -> Result<bool, ()> {
        Ok(self.service.verify(password, encoded))
    }

    fn needs_rehash(&self, encoded: &str) -> bool {
        self.service.needs_rehash(encoded)
    }
}

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
    InvalidConfiguration,
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
            Self::InvalidConfiguration => "staff authentication configuration is invalid",
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
    passwords: Arc<dyn PasswordOperations>,
    session_lifetime: Duration,
    dummy_hash: String,
    // The API constructs one service at process startup; clones share this gate.
    verifier_admission: Arc<Semaphore>,
}

impl StaffAuthService {
    pub fn new(
        repository: Arc<dyn StaffAuthRepository>,
        passwords: Argon2PasswordService,
        session_lifetime: Duration,
    ) -> Result<Self, StaffAuthError> {
        Self::new_with_max_concurrent_password_verifications(
            repository,
            passwords,
            session_lifetime,
            DEFAULT_MAX_CONCURRENT_PASSWORD_VERIFICATIONS,
        )
    }

    pub fn new_with_max_concurrent_password_verifications(
        repository: Arc<dyn StaffAuthRepository>,
        passwords: Argon2PasswordService,
        session_lifetime: Duration,
        max_concurrent_verifications: usize,
    ) -> Result<Self, StaffAuthError> {
        if !is_valid_password_verification_limit(max_concurrent_verifications) {
            return Err(StaffAuthError::InvalidConfiguration);
        }
        let passwords: Arc<dyn PasswordOperations> =
            Arc::new(Argon2PasswordOperations { service: passwords });
        let dummy_hash = passwords
            .hash("x-fly-dummy-password-that-is-never-an-account")
            .map_err(|_| StaffAuthError::Infrastructure)?;
        Self::from_parts(
            repository,
            passwords,
            session_lifetime,
            dummy_hash,
            max_concurrent_verifications,
        )
    }

    fn from_parts(
        repository: Arc<dyn StaffAuthRepository>,
        passwords: Arc<dyn PasswordOperations>,
        session_lifetime: Duration,
        dummy_hash: String,
        max_concurrent_verifications: usize,
    ) -> Result<Self, StaffAuthError> {
        if !is_valid_password_verification_limit(max_concurrent_verifications) {
            return Err(StaffAuthError::InvalidConfiguration);
        }
        Ok(Self {
            repository,
            passwords,
            session_lifetime,
            dummy_hash,
            verifier_admission: Arc::new(Semaphore::new(max_concurrent_verifications)),
        })
    }

    #[cfg(test)]
    fn new_for_tests(
        repository: Arc<dyn StaffAuthRepository>,
        passwords: Arc<dyn PasswordOperations>,
        dummy_hash: String,
        session_lifetime: Duration,
        max_concurrent_verifications: usize,
    ) -> Result<Self, StaffAuthError> {
        Self::from_parts(
            repository,
            passwords,
            session_lifetime,
            dummy_hash,
            max_concurrent_verifications,
        )
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<StaffLogin, StaffAuthError> {
        self.login_with_request_id(email, password, Uuid::new_v4())
            .await
    }

    pub async fn login_with_request_id(
        &self,
        email: &str,
        password: &str,
        request_id: Uuid,
    ) -> Result<StaffLogin, StaffAuthError> {
        let normalized_identifier = email.trim().to_lowercase();
        let identifier_hash = hash_bytes(normalized_identifier.as_bytes());
        let blocked = self
            .repository
            .login_is_blocked(identifier_hash)
            .await
            .map_err(StaffAuthError::from)?;
        if blocked {
            return Err(StaffAuthError::Throttled);
        }
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
        // Fail fast when all verifier slots are in use; do not queue login work.
        let permit = self
            .verifier_admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| StaffAuthError::Throttled)?;
        let verified = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            passwords.verify(&password_value, &encoded)
        })
        .await
        .map_err(|_| StaffAuthError::Infrastructure)?
        .map_err(|_| StaffAuthError::Infrastructure)?;
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
            .create_session(credential.id, token_hash, self.session_lifetime, request_id)
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
        self.logout_with_request_id(token, Uuid::new_v4()).await
    }

    pub async fn logout_with_request_id(
        &self,
        token: &str,
        request_id: Uuid,
    ) -> Result<(), StaffAuthError> {
        if let Ok(raw) = decode_token(token) {
            self.repository
                .revoke_session(hash_bytes(&raw), request_id)
                .await
                .map_err(StaffAuthError::from)?;
        }
        Ok(())
    }

    pub async fn record_authorization_denied(
        &self,
        principal: &StaffPrincipal,
        permission: PermissionCode,
        request_id: Uuid,
    ) -> Result<(), StaffAuthError> {
        self.repository
            .record_authorization_denied(
                principal.staff_user_id(),
                principal.session_id(),
                permission,
                request_id,
            )
            .await
            .map_err(StaffAuthError::from)
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::{
        collections::VecDeque,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Barrier, Mutex,
        },
    };
    use tokio::sync::Notify;
    use uuid::Uuid;

    use crate::domain::{
        repositories::StaffAuthRepositoryError,
        staff::{PermissionCode, RoleCode, StaffCredential},
    };

    #[derive(Clone, Default)]
    struct RecordingRepository {
        state: Arc<Mutex<RepositoryState>>,
    }

    #[derive(Default)]
    struct RepositoryState {
        blocked: bool,
        credential: Option<StaffCredential>,
        credential_lookups: usize,
        failure_count: usize,
        clear_count: usize,
        sessions: usize,
        block_on_failure: bool,
    }

    #[derive(Clone, Copy)]
    struct RepositorySnapshot {
        credential_lookups: usize,
        failure_count: usize,
        clear_count: usize,
        sessions: usize,
    }

    impl RecordingRepository {
        fn new(credential: Option<StaffCredential>) -> Self {
            Self {
                state: Arc::new(Mutex::new(RepositoryState {
                    credential,
                    ..RepositoryState::default()
                })),
            }
        }

        fn blocked(&self, blocked: bool) {
            self.state.lock().expect("repository state lock").blocked = blocked;
        }

        fn block_on_failure(&self, blocked: bool) {
            self.state
                .lock()
                .expect("repository state lock")
                .block_on_failure = blocked;
        }

        fn snapshot(&self) -> RepositorySnapshot {
            let state = self.state.lock().expect("repository state lock");
            RepositorySnapshot {
                credential_lookups: state.credential_lookups,
                failure_count: state.failure_count,
                clear_count: state.clear_count,
                sessions: state.sessions,
            }
        }
    }

    #[async_trait]
    impl StaffAuthRepository for RecordingRepository {
        async fn credential_for_email(
            &self,
            _email: &str,
        ) -> Result<Option<StaffCredential>, StaffAuthRepositoryError> {
            let mut state = self.state.lock().expect("repository state lock");
            state.credential_lookups += 1;
            Ok(state.credential.clone())
        }

        async fn login_is_blocked(
            &self,
            _identifier_hash: [u8; 32],
        ) -> Result<bool, StaffAuthRepositoryError> {
            Ok(self.state.lock().expect("repository state lock").blocked)
        }

        async fn record_login_failure(
            &self,
            _identifier_hash: [u8; 32],
        ) -> Result<bool, StaffAuthRepositoryError> {
            let mut state = self.state.lock().expect("repository state lock");
            state.failure_count += 1;
            Ok(state.block_on_failure)
        }

        async fn clear_login_failures(
            &self,
            _identifier_hash: [u8; 32],
        ) -> Result<(), StaffAuthRepositoryError> {
            self.state
                .lock()
                .expect("repository state lock")
                .clear_count += 1;
            Ok(())
        }

        async fn update_password_hash(
            &self,
            _staff_user_id: Uuid,
            _password_hash: &str,
        ) -> Result<(), StaffAuthRepositoryError> {
            Ok(())
        }

        async fn create_session(
            &self,
            staff_user_id: Uuid,
            _token_hash: [u8; 32],
            _lifetime: Duration,
            _request_id: Uuid,
        ) -> Result<StaffPrincipal, StaffAuthRepositoryError> {
            self.state.lock().expect("repository state lock").sessions += 1;
            Ok(StaffPrincipal::new(
                staff_user_id,
                Uuid::new_v4(),
                "staff@x-fly.internal".to_owned(),
                vec![RoleCode::Executive],
                vec![PermissionCode::DashboardRead],
                Utc::now() + chrono::Duration::hours(1),
            ))
        }

        async fn authenticate_session(
            &self,
            _token_hash: [u8; 32],
        ) -> Result<Option<StaffPrincipal>, StaffAuthRepositoryError> {
            Ok(None)
        }

        async fn revoke_session(
            &self,
            _token_hash: [u8; 32],
            _request_id: Uuid,
        ) -> Result<(), StaffAuthRepositoryError> {
            Ok(())
        }

        async fn record_authorization_denied(
            &self,
            _staff_user_id: Uuid,
            _session_id: Uuid,
            _permission: PermissionCode,
            _request_id: Uuid,
        ) -> Result<(), StaffAuthRepositoryError> {
            Ok(())
        }

        async fn provision_staff(
            &self,
            _first_only: bool,
            _email: &str,
            _password_hash: &str,
            _roles: &[RoleCode],
        ) -> Result<(), StaffAuthRepositoryError> {
            Ok(())
        }
    }

    #[derive(Clone)]
    struct RecordingPasswordOperations {
        calls: Arc<AtomicUsize>,
        dummy_calls: Arc<AtomicUsize>,
        real_calls: Arc<AtomicUsize>,
        active: Arc<AtomicUsize>,
        max_active: Arc<AtomicUsize>,
        started: Arc<AtomicUsize>,
        started_notify: Arc<Notify>,
        barrier: Option<Arc<Barrier>>,
        barrier_call_limit: usize,
        outcomes: Arc<Mutex<VecDeque<Result<bool, ()>>>>,
        rehash: bool,
    }

    impl RecordingPasswordOperations {
        fn new(outcomes: Vec<Result<bool, ()>>) -> Arc<Self> {
            Arc::new(Self {
                calls: Arc::new(AtomicUsize::new(0)),
                dummy_calls: Arc::new(AtomicUsize::new(0)),
                real_calls: Arc::new(AtomicUsize::new(0)),
                active: Arc::new(AtomicUsize::new(0)),
                max_active: Arc::new(AtomicUsize::new(0)),
                started: Arc::new(AtomicUsize::new(0)),
                started_notify: Arc::new(Notify::new()),
                barrier: None,
                barrier_call_limit: 0,
                outcomes: Arc::new(Mutex::new(outcomes.into_iter().collect())),
                rehash: false,
            })
        }

        fn gated(
            outcomes: Vec<Result<bool, ()>>,
            barrier: Arc<Barrier>,
            barrier_call_limit: usize,
        ) -> Arc<Self> {
            Arc::new(Self {
                barrier: Some(barrier),
                barrier_call_limit,
                ..Self::new(outcomes).as_ref().clone()
            })
        }
    }

    impl PasswordOperations for RecordingPasswordOperations {
        fn hash(&self, _password: &str) -> Result<String, ()> {
            Ok("test-password-hash".to_owned())
        }

        fn verify(&self, _password: &str, encoded: &str) -> Result<bool, ()> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if encoded == "dummy-hash" {
                self.dummy_calls.fetch_add(1, Ordering::SeqCst);
            } else {
                self.real_calls.fetch_add(1, Ordering::SeqCst);
            }
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            self.max_active.fetch_max(active, Ordering::SeqCst);
            let ordinal = self.started.fetch_add(1, Ordering::SeqCst) + 1;
            self.started_notify.notify_one();
            if ordinal <= self.barrier_call_limit {
                if let Some(barrier) = &self.barrier {
                    barrier.wait();
                }
            }
            self.active.fetch_sub(1, Ordering::SeqCst);
            self.outcomes
                .lock()
                .expect("password outcome lock")
                .pop_front()
                .unwrap_or(Ok(false))
        }

        fn needs_rehash(&self, _encoded: &str) -> bool {
            self.rehash
        }
    }

    fn credential() -> StaffCredential {
        StaffCredential {
            id: Uuid::new_v4(),
            password_hash: "real-hash".to_owned(),
            active: true,
        }
    }

    fn service(
        repository: RecordingRepository,
        passwords: Arc<RecordingPasswordOperations>,
        limit: usize,
    ) -> StaffAuthService {
        StaffAuthService::new_for_tests(
            Arc::new(repository),
            passwords,
            "dummy-hash".to_owned(),
            Duration::from_secs(3600),
            limit,
        )
        .expect("valid test verifier limit")
    }

    async fn wait_for_started(passwords: &RecordingPasswordOperations, target: usize) {
        loop {
            let notified = passwords.started_notify.notified();
            if passwords.started.load(Ordering::SeqCst) >= target {
                return;
            }
            notified.await;
        }
    }

    #[test]
    fn verifier_limit_must_be_nonzero() {
        let result = StaffAuthService::new_with_max_concurrent_password_verifications(
            Arc::new(RecordingRepository::new(None)),
            Argon2PasswordService::default(),
            Duration::from_secs(3600),
            0,
        );
        assert!(matches!(result, Err(StaffAuthError::InvalidConfiguration)));
    }

    #[test]
    fn verifier_limit_must_not_exceed_security_maximum() {
        let result = StaffAuthService::new_with_max_concurrent_password_verifications(
            Arc::new(RecordingRepository::new(None)),
            Argon2PasswordService::default(),
            Duration::from_secs(3600),
            5,
        );
        assert!(matches!(result, Err(StaffAuthError::InvalidConfiguration)));
    }

    async fn release_gate(barrier: Arc<Barrier>) {
        tokio::task::spawn_blocking(move || barrier.wait())
            .await
            .expect("release password verifier gate");
    }

    #[tokio::test]
    async fn blocked_identifier_skips_all_password_verification() {
        let repository = RecordingRepository::new(Some(credential()));
        repository.blocked(true);
        let passwords = RecordingPasswordOperations::new(vec![Ok(true)]);
        let auth = service(repository.clone(), passwords.clone(), 1);

        assert_eq!(
            auth.login("blocked@x-fly.internal", "password").await,
            Err(StaffAuthError::Throttled)
        );
        assert_eq!(passwords.calls.load(Ordering::SeqCst), 0);
        assert_eq!(repository.snapshot().credential_lookups, 0);
        assert_eq!(repository.snapshot().sessions, 0);
    }

    #[tokio::test]
    async fn unknown_non_blocked_identifier_keeps_dummy_verification() {
        let repository = RecordingRepository::new(None);
        let passwords = RecordingPasswordOperations::new(vec![Ok(false)]);
        let auth = service(repository.clone(), passwords.clone(), 1);

        assert_eq!(
            auth.login("unknown@x-fly.internal", "password").await,
            Err(StaffAuthError::InvalidCredentials)
        );
        assert_eq!(passwords.calls.load(Ordering::SeqCst), 1);
        assert_eq!(passwords.dummy_calls.load(Ordering::SeqCst), 1);
        assert_eq!(passwords.real_calls.load(Ordering::SeqCst), 0);
        assert_eq!(repository.snapshot().sessions, 0);
    }

    #[tokio::test]
    async fn known_non_blocked_identifier_keeps_real_verification_and_success() {
        let repository = RecordingRepository::new(Some(credential()));
        let passwords = RecordingPasswordOperations::new(vec![Ok(true)]);
        let auth = service(repository.clone(), passwords.clone(), 1);

        assert!(auth.login("STAFF@X-FLY.INTERNAL", "password").await.is_ok());
        assert_eq!(passwords.calls.load(Ordering::SeqCst), 1);
        assert_eq!(passwords.real_calls.load(Ordering::SeqCst), 1);
        assert_eq!(passwords.dummy_calls.load(Ordering::SeqCst), 0);
        let snapshot = repository.snapshot();
        assert_eq!(snapshot.clear_count, 1);
        assert_eq!(snapshot.failure_count, 0);
        assert_eq!(snapshot.sessions, 1);
    }

    #[tokio::test]
    async fn global_verifier_concurrency_is_bounded_for_repeated_identifier_attempts() {
        let limit = 2;
        let barrier = Arc::new(Barrier::new(limit + 1));
        let repository = RecordingRepository::new(None);
        let passwords = RecordingPasswordOperations::gated(vec![], barrier.clone(), limit);
        let auth = service(repository, passwords.clone(), limit);
        let attempts = (0..limit + 3)
            .map(|_| {
                let auth = auth.clone();
                tokio::spawn(async move { auth.login("same@x-fly.internal", "password").await })
            })
            .collect::<Vec<_>>();

        wait_for_started(&passwords, limit).await;
        assert_eq!(passwords.calls.load(Ordering::SeqCst), limit);
        assert_eq!(passwords.max_active.load(Ordering::SeqCst), limit);
        release_gate(barrier).await;
        for attempt in attempts {
            let _ = attempt.await.expect("login attempt task");
        }
        assert!(passwords.max_active.load(Ordering::SeqCst) <= limit);
    }

    #[tokio::test]
    async fn saturated_verifier_rejects_without_starting_password_verification() {
        let barrier = Arc::new(Barrier::new(2));
        let repository = RecordingRepository::new(None);
        let passwords = RecordingPasswordOperations::gated(vec![Ok(false)], barrier.clone(), 1);
        let auth = service(repository.clone(), passwords.clone(), 1);
        let first = {
            let auth = auth.clone();
            tokio::spawn(async move { auth.login("first@x-fly.internal", "password").await })
        };
        wait_for_started(&passwords, 1).await;

        assert_eq!(
            auth.login("second@x-fly.internal", "password").await,
            Err(StaffAuthError::Throttled)
        );
        assert_eq!(passwords.calls.load(Ordering::SeqCst), 1);
        let snapshot = repository.snapshot();
        assert_eq!(snapshot.failure_count, 0);
        assert_eq!(snapshot.sessions, 0);
        release_gate(barrier).await;
        assert_eq!(
            first.await.expect("first login task"),
            Err(StaffAuthError::InvalidCredentials)
        );
    }

    #[tokio::test]
    async fn saturation_does_not_increment_a_victim_failure_counter() {
        let barrier = Arc::new(Barrier::new(2));
        let repository = RecordingRepository::new(None);
        let passwords = RecordingPasswordOperations::gated(vec![Ok(false)], barrier.clone(), 1);
        let auth = service(repository.clone(), passwords.clone(), 1);
        let attacker = {
            let auth = auth.clone();
            tokio::spawn(async move { auth.login("attacker@x-fly.internal", "password").await })
        };
        wait_for_started(&passwords, 1).await;

        assert_eq!(
            auth.login("victim@x-fly.internal", "password").await,
            Err(StaffAuthError::Throttled)
        );
        let snapshot = repository.snapshot();
        assert_eq!(snapshot.failure_count, 0);
        assert_eq!(snapshot.sessions, 0);
        release_gate(barrier).await;
        let _ = attacker.await.expect("attacker login task");
        assert_eq!(repository.snapshot().failure_count, 1);
    }

    #[tokio::test]
    async fn admitted_bad_password_preserves_identifier_throttle_update() {
        let repository = RecordingRepository::new(Some(credential()));
        repository.block_on_failure(true);
        let passwords = RecordingPasswordOperations::new(vec![Ok(false)]);
        let auth = service(repository.clone(), passwords.clone(), 1);

        assert_eq!(
            auth.login("staff@x-fly.internal", "wrong-password").await,
            Err(StaffAuthError::Throttled)
        );
        assert_eq!(passwords.real_calls.load(Ordering::SeqCst), 1);
        assert_eq!(repository.snapshot().failure_count, 1);
    }

    #[tokio::test]
    async fn successful_login_preserves_existing_failure_clear_semantics() {
        let repository = RecordingRepository::new(Some(credential()));
        let passwords = RecordingPasswordOperations::new(vec![Ok(true)]);
        let auth = service(repository.clone(), passwords, 1);

        assert!(auth.login("staff@x-fly.internal", "password").await.is_ok());
        assert_eq!(repository.snapshot().clear_count, 1);
        assert_eq!(repository.snapshot().failure_count, 0);
    }

    #[tokio::test]
    async fn verifier_permit_recovers_after_error_and_invalid_password() {
        let repository = RecordingRepository::new(Some(credential()));
        let passwords = RecordingPasswordOperations::new(vec![Err(()), Ok(false), Ok(true)]);
        let auth = service(repository.clone(), passwords.clone(), 1);

        assert_eq!(
            auth.login("staff@x-fly.internal", "password").await,
            Err(StaffAuthError::Infrastructure)
        );
        assert_eq!(repository.snapshot().failure_count, 0);
        assert_eq!(
            auth.login("staff@x-fly.internal", "password").await,
            Err(StaffAuthError::InvalidCredentials)
        );
        assert!(auth.login("staff@x-fly.internal", "password").await.is_ok());
        assert_eq!(passwords.calls.load(Ordering::SeqCst), 3);
        assert_eq!(repository.snapshot().sessions, 1);
    }

    #[tokio::test]
    async fn different_identifiers_cannot_bypass_global_verifier_bound() {
        let limit = 2;
        let barrier = Arc::new(Barrier::new(limit + 1));
        let repository = RecordingRepository::new(None);
        let passwords = RecordingPasswordOperations::gated(vec![], barrier.clone(), limit);
        let auth = service(repository, passwords.clone(), limit);
        let attempts = (0..limit + 5)
            .map(|index| {
                let auth = auth.clone();
                tokio::spawn(async move {
                    auth.login(&format!("user{index}@x-fly.internal"), "password")
                        .await
                })
            })
            .collect::<Vec<_>>();

        wait_for_started(&passwords, limit).await;
        assert_eq!(passwords.calls.load(Ordering::SeqCst), limit);
        assert_eq!(passwords.max_active.load(Ordering::SeqCst), limit);
        release_gate(barrier).await;
        for attempt in attempts {
            let _ = attempt.await.expect("login attempt task");
        }
        assert!(passwords.max_active.load(Ordering::SeqCst) <= limit);
    }
}
