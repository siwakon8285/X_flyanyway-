use std::{str::FromStr, time::Duration};

use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::domain::{
    repositories::{StaffAuthRepository, StaffAuthRepositoryError},
    staff::{PermissionCode, RoleCode, StaffCredential, StaffPrincipal},
};

#[derive(Clone, Debug)]
pub struct SqlxStaffAuthRepository {
    pool: PgPool,
}

impl SqlxStaffAuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    async fn principal_for_hash(
        &self,
        token_hash: [u8; 32],
    ) -> Result<Option<StaffPrincipal>, StaffAuthRepositoryError> {
        let row = sqlx::query_as::<_, PrincipalRow>(
            "SELECT session.id AS session_id, staff.id AS staff_user_id, staff.email,
                    session.expires_at,
                    COALESCE(array_agg(DISTINCT assignment.role_code)
                        FILTER (WHERE assignment.role_code IS NOT NULL), '{}') AS roles,
                    COALESCE(array_agg(DISTINCT role_permission.permission_code)
                        FILTER (WHERE role_permission.permission_code IS NOT NULL), '{}') AS permissions
             FROM staff_sessions AS session
             JOIN staff_users AS staff ON staff.id = session.staff_user_id
             LEFT JOIN staff_user_roles AS assignment ON assignment.staff_user_id = staff.id
             LEFT JOIN role_permissions AS role_permission ON role_permission.role_code = assignment.role_code
             WHERE session.token_hash = $1
               AND session.revoked_at IS NULL
               AND session.expires_at > NOW()
               AND staff.status = 'ACTIVE'
             GROUP BY session.id, staff.id",
        )
        .bind(token_hash.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(StaffAuthRepositoryError::Infrastructure)?;
        row.map(PrincipalRow::into_principal).transpose()
    }
}

#[derive(FromRow)]
struct PrincipalRow {
    session_id: Uuid,
    staff_user_id: Uuid,
    email: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    roles: Vec<String>,
    permissions: Vec<String>,
}

impl PrincipalRow {
    fn into_principal(self) -> Result<StaffPrincipal, StaffAuthRepositoryError> {
        let roles = self
            .roles
            .iter()
            .map(|value| RoleCode::from_str(value))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| StaffAuthRepositoryError::InconsistentState)?;
        let permissions = self
            .permissions
            .iter()
            .map(|value| PermissionCode::from_str(value))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| StaffAuthRepositoryError::InconsistentState)?;
        Ok(StaffPrincipal::new(
            self.staff_user_id,
            self.session_id,
            self.email,
            roles,
            permissions,
            self.expires_at,
        ))
    }
}

#[derive(FromRow)]
struct CredentialRow {
    id: Uuid,
    password_hash: String,
    status: String,
}

#[async_trait]
impl StaffAuthRepository for SqlxStaffAuthRepository {
    async fn credential_for_email(
        &self,
        email: &str,
    ) -> Result<Option<StaffCredential>, StaffAuthRepositoryError> {
        sqlx::query_as::<_, CredentialRow>(
            "SELECT id, password_hash, status FROM staff_users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|row| StaffCredential {
                id: row.id,
                password_hash: row.password_hash,
                active: row.status == "ACTIVE",
            })
        })
        .map_err(StaffAuthRepositoryError::Infrastructure)
    }

    async fn login_is_blocked(
        &self,
        identifier_hash: [u8; 32],
    ) -> Result<bool, StaffAuthRepositoryError> {
        sqlx::query_scalar(
            "SELECT COALESCE((SELECT blocked_until > NOW() FROM staff_login_throttles WHERE identifier_hash = $1), FALSE)",
        )
        .bind(identifier_hash.as_slice())
        .fetch_one(&self.pool)
        .await
        .map_err(StaffAuthRepositoryError::Infrastructure)
    }

    async fn record_login_failure(
        &self,
        identifier_hash: [u8; 32],
    ) -> Result<bool, StaffAuthRepositoryError> {
        sqlx::query(
            "DELETE FROM staff_login_throttles WHERE updated_at < NOW() - INTERVAL '24 hours'",
        )
        .execute(&self.pool)
        .await
        .map_err(StaffAuthRepositoryError::Infrastructure)?;
        sqlx::query_scalar(
            "INSERT INTO staff_login_throttles (
                identifier_hash, failure_count, window_started_at, blocked_until, updated_at
             ) VALUES ($1, 1, NOW(), NULL, NOW())
             ON CONFLICT (identifier_hash) DO UPDATE SET
                failure_count = CASE
                    WHEN staff_login_throttles.window_started_at <= NOW() - INTERVAL '15 minutes' THEN 1
                    ELSE staff_login_throttles.failure_count + 1 END,
                window_started_at = CASE
                    WHEN staff_login_throttles.window_started_at <= NOW() - INTERVAL '15 minutes' THEN NOW()
                    ELSE staff_login_throttles.window_started_at END,
                blocked_until = CASE
                    WHEN staff_login_throttles.window_started_at <= NOW() - INTERVAL '15 minutes' THEN NULL
                    WHEN staff_login_throttles.failure_count + 1 >= 5 THEN NOW() + INTERVAL '15 minutes'
                    ELSE staff_login_throttles.blocked_until END,
                updated_at = NOW()
             RETURNING COALESCE(blocked_until > NOW(), FALSE)",
        )
        .bind(identifier_hash.as_slice())
        .fetch_one(&self.pool)
        .await
        .map_err(StaffAuthRepositoryError::Infrastructure)
    }

    async fn clear_login_failures(
        &self,
        identifier_hash: [u8; 32],
    ) -> Result<(), StaffAuthRepositoryError> {
        sqlx::query("DELETE FROM staff_login_throttles WHERE identifier_hash = $1")
            .bind(identifier_hash.as_slice())
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(StaffAuthRepositoryError::Infrastructure)
    }

    async fn update_password_hash(
        &self,
        staff_user_id: Uuid,
        password_hash: &str,
    ) -> Result<(), StaffAuthRepositoryError> {
        sqlx::query("UPDATE staff_users SET password_hash = $2, updated_at = NOW() WHERE id = $1")
            .bind(staff_user_id)
            .bind(password_hash)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(StaffAuthRepositoryError::Infrastructure)
    }

    async fn create_session(
        &self,
        staff_user_id: Uuid,
        token_hash: [u8; 32],
        lifetime: Duration,
    ) -> Result<StaffPrincipal, StaffAuthRepositoryError> {
        let seconds = i64::try_from(lifetime.as_secs())
            .map_err(|_| StaffAuthRepositoryError::InconsistentState)?;
        sqlx::query(
            "INSERT INTO staff_sessions (staff_user_id, token_hash, expires_at)
             SELECT id, $2, NOW() + make_interval(secs => $3::double precision)
             FROM staff_users WHERE id = $1 AND status = 'ACTIVE'",
        )
        .bind(staff_user_id)
        .bind(token_hash.as_slice())
        .bind(seconds as f64)
        .execute(&self.pool)
        .await
        .map_err(StaffAuthRepositoryError::Infrastructure)?;
        self.principal_for_hash(token_hash)
            .await?
            .ok_or(StaffAuthRepositoryError::InconsistentState)
    }

    async fn authenticate_session(
        &self,
        token_hash: [u8; 32],
    ) -> Result<Option<StaffPrincipal>, StaffAuthRepositoryError> {
        self.principal_for_hash(token_hash).await
    }

    async fn revoke_session(&self, token_hash: [u8; 32]) -> Result<(), StaffAuthRepositoryError> {
        sqlx::query(
            "UPDATE staff_sessions SET revoked_at = COALESCE(revoked_at, NOW()),
                    revocation_reason = COALESCE(revocation_reason, 'LOGOUT')
             WHERE token_hash = $1",
        )
        .bind(token_hash.as_slice())
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(StaffAuthRepositoryError::Infrastructure)
    }

    async fn provision_staff(
        &self,
        first_only: bool,
        email: &str,
        password_hash: &str,
        roles: &[RoleCode],
    ) -> Result<(), StaffAuthRepositoryError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(StaffAuthRepositoryError::Infrastructure)?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext('x_fly_staff_provisioning'))")
            .execute(&mut *transaction)
            .await
            .map_err(StaffAuthRepositoryError::Infrastructure)?;
        let staff_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM staff_users")
            .fetch_one(&mut *transaction)
            .await
            .map_err(StaffAuthRepositoryError::Infrastructure)?;
        if first_only && staff_count != 0 {
            return Err(StaffAuthRepositoryError::BootstrapAlreadyCompleted);
        }
        if !first_only && staff_count == 0 {
            return Err(StaffAuthRepositoryError::BootstrapRequired);
        }
        let role_values: Vec<&str> = roles.iter().map(|role| role.as_str()).collect();
        let canonical_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM roles WHERE code = ANY($1)")
                .bind(&role_values)
                .fetch_one(&mut *transaction)
                .await
                .map_err(StaffAuthRepositoryError::Infrastructure)?;
        let unique_count = role_values
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        if canonical_count != unique_count as i64 {
            return Err(StaffAuthRepositoryError::UnknownRole);
        }
        let staff_user_id: Uuid = sqlx::query_scalar(
            "INSERT INTO staff_users (email, password_hash) VALUES ($1, $2) RETURNING id",
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| {
            if error
                .as_database_error()
                .is_some_and(|value| value.is_unique_violation())
            {
                StaffAuthRepositoryError::DuplicateEmail
            } else {
                StaffAuthRepositoryError::Infrastructure(error)
            }
        })?;
        sqlx::query(
            "INSERT INTO staff_user_roles (staff_user_id, role_code)
             SELECT $1, role_code FROM unnest($2::text[]) AS role_code",
        )
        .bind(staff_user_id)
        .bind(&role_values)
        .execute(&mut *transaction)
        .await
        .map_err(StaffAuthRepositoryError::Infrastructure)?;
        transaction
            .commit()
            .await
            .map_err(StaffAuthRepositoryError::Infrastructure)
    }
}
