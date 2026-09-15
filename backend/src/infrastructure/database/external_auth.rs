use std::collections::BTreeSet;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::{
    application::external_auth::ExternalAuthRepository,
    domain::{
        api_client::{ApiClientScope, ApiClientStatus},
        external_api::{
            AccessTokenHash, CredentialAdministrationError, CredentialDigest, CredentialMetadata,
            CredentialRevocationReason, ExternalAuthenticationError, ExternalPrincipal,
            ExternalTokenExchangeError,
        },
    },
};

#[derive(Clone)]
pub struct SqlxExternalAuthRepository {
    pool: PgPool,
}

impl SqlxExternalAuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl ExternalAuthRepository for SqlxExternalAuthRepository {
    async fn issue_credential(
        &self,
        client_id: &str,
        actor_staff_user_id: Uuid,
        expected_client_version: i64,
        digest: &CredentialDigest,
        issued_at: DateTime<Utc>,
    ) -> Result<CredentialMetadata, CredentialAdministrationError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CredentialAdministrationError::Infrastructure)?;
        let client =
            lock_client(&mut transaction, client_id)
                .await
                .map_err(|error| match error {
                    CredentialAdministrationError::Infrastructure => error,
                    other => other,
                })?;
        if client.version != expected_client_version {
            return Err(CredentialAdministrationError::VersionConflict);
        }
        if client.status() == ApiClientStatus::Revoked {
            return Err(CredentialAdministrationError::RevokedClient);
        }

        let has_live_credential: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                 SELECT 1 FROM api_client_credentials
                 WHERE api_client_id=$1 AND revoked_at IS NULL
             )",
        )
        .bind(client.id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| CredentialAdministrationError::Infrastructure)?;
        if has_live_credential {
            return Err(CredentialAdministrationError::LiveCredential);
        }

        let metadata = match sqlx::query_as::<_, CredentialRow>(
            "INSERT INTO api_client_credentials (
                 api_client_id,secret_digest,digest_version,issued_at,issued_by_staff_user_id
             ) VALUES ($1,$2,1,$3,$4)
             RETURNING id,issued_at,revoked_at,revocation_reason",
        )
        .bind(client.id)
        .bind(digest.as_bytes().as_slice())
        .bind(issued_at)
        .bind(actor_staff_user_id)
        .fetch_one(&mut *transaction)
        .await
        {
            Ok(row) => row
                .metadata()
                .map_err(|_| CredentialAdministrationError::Infrastructure)?,
            Err(error)
                if error
                    .as_database_error()
                    .is_some_and(|database| database.is_unique_violation()) =>
            {
                return Err(CredentialAdministrationError::LiveCredential)
            }
            Err(_) => return Err(CredentialAdministrationError::Infrastructure),
        };

        increment_client_version(&mut transaction, client.id, actor_staff_user_id)
            .await
            .map_err(|_| CredentialAdministrationError::Infrastructure)?;
        insert_credential_audit(
            &mut transaction,
            &client,
            actor_staff_user_id,
            "CREDENTIAL_ISSUED",
            metadata.credential_id,
            None,
            &metadata,
        )
        .await
        .map_err(|_| CredentialAdministrationError::Infrastructure)?;

        transaction
            .commit()
            .await
            .map_err(|_| CredentialAdministrationError::Infrastructure)?;
        Ok(metadata)
    }

    async fn revoke_credential(
        &self,
        client_id: &str,
        actor_staff_user_id: Uuid,
        expected_client_version: i64,
        reason: CredentialRevocationReason,
        revoked_at: DateTime<Utc>,
    ) -> Result<CredentialMetadata, CredentialAdministrationError> {
        let revoked_at = postgres_timestamp(revoked_at);
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CredentialAdministrationError::Infrastructure)?;
        let client = lock_client(&mut transaction, client_id).await?;
        if client.version != expected_client_version {
            return Err(CredentialAdministrationError::VersionConflict);
        }
        if client.status() == ApiClientStatus::Revoked {
            return Err(CredentialAdministrationError::RevokedClient);
        }
        let credential = sqlx::query_as::<_, CredentialRow>(
            "SELECT id,issued_at,revoked_at,revocation_reason
             FROM api_client_credentials
             WHERE api_client_id=$1 AND revoked_at IS NULL
             ORDER BY issued_at DESC, id DESC
             LIMIT 1
             FOR UPDATE",
        )
        .bind(client.id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| CredentialAdministrationError::Infrastructure)?
        .ok_or(CredentialAdministrationError::NoLiveCredential)?;
        let metadata = credential
            .metadata()
            .map_err(|_| CredentialAdministrationError::Infrastructure)?;

        revoke_live_tokens(&mut transaction, credential.id, revoked_at)
            .await
            .map_err(|_| CredentialAdministrationError::Infrastructure)?;
        sqlx::query(
            "UPDATE api_client_credentials
             SET revoked_at=$2,revoked_by_staff_user_id=$3,revocation_reason=$4
             WHERE id=$1 AND revoked_at IS NULL",
        )
        .bind(credential.id)
        .bind(revoked_at)
        .bind(actor_staff_user_id)
        .bind(reason.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(|_| CredentialAdministrationError::Infrastructure)?;
        let revoked_metadata = CredentialMetadata {
            credential_id: metadata.credential_id,
            issued_at: metadata.issued_at,
            revoked_at: Some(revoked_at),
            revocation_reason: Some(reason),
        };
        increment_client_version(&mut transaction, client.id, actor_staff_user_id)
            .await
            .map_err(|_| CredentialAdministrationError::Infrastructure)?;
        insert_credential_audit(
            &mut transaction,
            &client,
            actor_staff_user_id,
            "CREDENTIAL_REVOKED",
            credential.id,
            Some(&metadata),
            &revoked_metadata,
        )
        .await
        .map_err(|_| CredentialAdministrationError::Infrastructure)?;

        transaction
            .commit()
            .await
            .map_err(|_| CredentialAdministrationError::Infrastructure)?;
        Ok(revoked_metadata)
    }

    async fn issue_access_token(
        &self,
        client_id: &str,
        digest: &CredentialDigest,
        token_hash: &AccessTokenHash,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<(), ExternalTokenExchangeError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ExternalTokenExchangeError::Unavailable)?;
        let client = sqlx::query_as::<_, ClientLockRow>(
            "SELECT id,status FROM api_clients WHERE client_id=$1 FOR UPDATE",
        )
        .bind(client_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| ExternalTokenExchangeError::Unavailable)?
        .ok_or(ExternalTokenExchangeError::InvalidCredential)?;
        if client.status() != ApiClientStatus::Active {
            return Err(ExternalTokenExchangeError::InvalidCredential);
        }
        let credential = sqlx::query_as::<_, CredentialVerifierRow>(
            "SELECT id,secret_digest
             FROM api_client_credentials
             WHERE api_client_id=$1 AND revoked_at IS NULL
             ORDER BY issued_at DESC, id DESC
             LIMIT 1
             FOR UPDATE",
        )
        .bind(client.id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|_| ExternalTokenExchangeError::Unavailable)?
        .ok_or(ExternalTokenExchangeError::InvalidCredential)?;
        if !digest_matches(digest.as_bytes(), credential.secret_digest.as_slice()) {
            return Err(ExternalTokenExchangeError::InvalidCredential);
        }

        sqlx::query(
            "INSERT INTO external_access_tokens (
                 api_client_credential_id,token_hash,issued_at,expires_at
             ) VALUES ($1,$2,$3,$4)",
        )
        .bind(credential.id)
        .bind(token_hash.as_bytes().as_slice())
        .bind(issued_at)
        .bind(expires_at)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ExternalTokenExchangeError::Unavailable)?;
        transaction
            .commit()
            .await
            .map_err(|_| ExternalTokenExchangeError::Unavailable)
    }

    async fn authenticate_access_token(
        &self,
        token_hash: &AccessTokenHash,
        now: DateTime<Utc>,
    ) -> Result<ExternalPrincipal, ExternalAuthenticationError> {
        let row = sqlx::query_as::<_, PrincipalRow>(
            "SELECT client.id AS api_client_id,client.client_id,
                    ARRAY(
                        SELECT assignment.scope_code
                        FROM api_client_allowed_scopes assignment
                        WHERE assignment.api_client_id=client.id
                        ORDER BY assignment.scope_code
                    ) AS scopes
             FROM external_access_tokens token
             JOIN api_client_credentials credential
               ON credential.id=token.api_client_credential_id
             JOIN api_clients client ON client.id=credential.api_client_id
             WHERE token.token_hash=$1
               AND token.revoked_at IS NULL
               AND token.expires_at > $2
               AND credential.revoked_at IS NULL
               AND client.status='ACTIVE'
             LIMIT 1",
        )
        .bind(token_hash.as_bytes().as_slice())
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| ExternalAuthenticationError::Unavailable)?
        .ok_or(ExternalAuthenticationError::InvalidToken)?;
        row.principal()
            .map_err(|_| ExternalAuthenticationError::Unavailable)
    }
}

pub async fn revoke_client_credentials_and_tokens(
    transaction: &mut Transaction<'_, Postgres>,
    client_id: Uuid,
    reason: CredentialRevocationReason,
    revoked_at: DateTime<Utc>,
) -> Result<Option<CredentialMetadata>, CredentialAdministrationError> {
    let revoked_at = postgres_timestamp(revoked_at);
    let _ = sqlx::query_scalar::<_, Uuid>("SELECT id FROM api_clients WHERE id=$1 FOR UPDATE")
        .bind(client_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| CredentialAdministrationError::Infrastructure)?
        .ok_or(CredentialAdministrationError::NotFound)?;
    let Some(credential) = sqlx::query_as::<_, CredentialRow>(
        "SELECT id,issued_at,revoked_at,revocation_reason
         FROM api_client_credentials
         WHERE api_client_id=$1 AND revoked_at IS NULL
         ORDER BY issued_at DESC, id DESC
         LIMIT 1
         FOR UPDATE",
    )
    .bind(client_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| CredentialAdministrationError::Infrastructure)?
    else {
        return Ok(None);
    };
    revoke_live_tokens(transaction, credential.id, revoked_at)
        .await
        .map_err(|_| CredentialAdministrationError::Infrastructure)?;
    sqlx::query(
        "UPDATE api_client_credentials
         SET revoked_at=$2,revoked_by_staff_user_id=NULL,revocation_reason=$3
         WHERE id=$1 AND revoked_at IS NULL",
    )
    .bind(credential.id)
    .bind(revoked_at)
    .bind(reason.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(|_| CredentialAdministrationError::Infrastructure)?;
    Ok(Some(CredentialMetadata {
        credential_id: credential.id,
        issued_at: credential.issued_at,
        revoked_at: Some(revoked_at),
        revocation_reason: Some(reason),
    }))
}

async fn lock_client(
    transaction: &mut Transaction<'_, Postgres>,
    client_id: &str,
) -> Result<ClientRow, CredentialAdministrationError> {
    sqlx::query_as::<_, ClientRow>(
        "SELECT client.id,client.client_id,client.display_name,client.description,
                client.status,client.version,
                ARRAY(
                    SELECT assignment.scope_code
                    FROM api_client_allowed_scopes assignment
                    WHERE assignment.api_client_id=client.id
                    ORDER BY assignment.scope_code
                ) AS scopes
         FROM api_clients client
         WHERE client.client_id=$1
         FOR UPDATE OF client",
    )
    .bind(client_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| CredentialAdministrationError::Infrastructure)?
    .ok_or(CredentialAdministrationError::NotFound)
}

async fn increment_client_version(
    transaction: &mut Transaction<'_, Postgres>,
    client_id: Uuid,
    actor_staff_user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE api_clients
         SET version=version+1,updated_by_staff_user_id=$2,updated_at=clock_timestamp()
         WHERE id=$1",
    )
    .bind(client_id)
    .bind(actor_staff_user_id)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
}

async fn revoke_live_tokens(
    transaction: &mut Transaction<'_, Postgres>,
    credential_id: Uuid,
    revoked_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE external_access_tokens
         SET revoked_at=$2
         WHERE api_client_credential_id=$1 AND revoked_at IS NULL",
    )
    .bind(credential_id)
    .bind(revoked_at)
    .execute(&mut **transaction)
    .await
    .map(|_| ())
}

async fn insert_credential_audit(
    transaction: &mut Transaction<'_, Postgres>,
    client: &ClientRow,
    actor_staff_user_id: Uuid,
    action: &str,
    credential_id: Uuid,
    before: Option<&CredentialMetadata>,
    after: &CredentialMetadata,
) -> Result<(), sqlx::Error> {
    let before_state = before.map(|_| client.audit_snapshot());
    let after_state = client.audit_snapshot();
    sqlx::query(
        "INSERT INTO api_client_management_audit (
             api_client_id,actor_staff_user_id,action,before_state,after_state,credential_id,created_at
         ) VALUES ($1,$2,$3,$4,$5,$6,clock_timestamp())",
    )
    .bind(client.id)
    .bind(actor_staff_user_id)
    .bind(action)
    .bind(before_state)
    .bind(after_state)
    .bind(credential_id)
    .execute(&mut **transaction)
    .await
    .map(|_| {
        let _ = after;
    })
}

fn postgres_timestamp(value: DateTime<Utc>) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp_micros(value.timestamp_micros())
        .expect("a valid UTC timestamp remains representable at PostgreSQL precision")
}

fn digest_matches(supplied: &[u8; 32], stored: &[u8]) -> bool {
    if stored.len() != 32 {
        return false;
    }
    supplied.as_slice().ct_eq(stored).into()
}

#[derive(FromRow)]
struct ClientRow {
    id: Uuid,
    #[allow(dead_code)]
    client_id: String,
    display_name: String,
    description: Option<String>,
    status: String,
    version: i64,
    scopes: Vec<String>,
}

impl ClientRow {
    fn status(&self) -> ApiClientStatus {
        ApiClientStatus::parse(&self.status).unwrap_or(ApiClientStatus::Revoked)
    }

    fn audit_snapshot(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.display_name,
            "description": self.description,
            "status": self.status,
            "allowedScopes": self.scopes,
        })
    }
}

#[derive(FromRow)]
struct ClientLockRow {
    id: Uuid,
    status: String,
}

impl ClientLockRow {
    fn status(&self) -> ApiClientStatus {
        ApiClientStatus::parse(&self.status).unwrap_or(ApiClientStatus::Revoked)
    }
}

#[derive(FromRow)]
struct CredentialRow {
    id: Uuid,
    issued_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
    revocation_reason: Option<String>,
}

impl CredentialRow {
    fn metadata(&self) -> Result<CredentialMetadata, ()> {
        Ok(CredentialMetadata {
            credential_id: self.id,
            issued_at: self.issued_at,
            revoked_at: self.revoked_at,
            revocation_reason: self
                .revocation_reason
                .as_deref()
                .map(parse_revocation_reason)
                .transpose()?,
        })
    }
}

#[derive(FromRow)]
struct CredentialVerifierRow {
    id: Uuid,
    secret_digest: Vec<u8>,
}

#[derive(FromRow)]
struct PrincipalRow {
    api_client_id: Uuid,
    client_id: String,
    scopes: Vec<String>,
}

impl PrincipalRow {
    fn principal(self) -> Result<ExternalPrincipal, ()> {
        let scopes = self
            .scopes
            .into_iter()
            .map(|scope| ApiClientScope::parse(&scope).ok_or(()))
            .collect::<Result<BTreeSet<_>, _>>()?;
        Ok(ExternalPrincipal::new(
            self.api_client_id,
            self.client_id,
            scopes,
        ))
    }
}

fn parse_revocation_reason(value: &str) -> Result<CredentialRevocationReason, ()> {
    match value {
        "ADMIN_REQUEST" => Ok(CredentialRevocationReason::AdminRequest),
        "CLIENT_SUSPENDED" => Ok(CredentialRevocationReason::ClientSuspended),
        "CLIENT_REVOKED" => Ok(CredentialRevocationReason::ClientRevoked),
        "REPLACED" => Ok(CredentialRevocationReason::Replaced),
        _ => Err(()),
    }
}
