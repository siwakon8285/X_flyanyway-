use async_trait::async_trait;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    application::api_client::{
        ApiClientAuditAction, ApiClientAuditEntry, ApiClientAuditSnapshot, ApiClientDetail,
        ApiClientListFilter, ApiClientManagementError, ApiClientPage, ApiClientRecord,
        ApiClientRepository, ApiScopeDefinition,
    },
    domain::api_client::{
        ApiClientScope, ApiClientStatus, ApiClientValidationError, UpdateApiClientCommand,
        ValidatedCreateApiClient,
    },
};

#[derive(Clone, Debug)]
pub struct SqlxApiClientRepository {
    pool: PgPool,
}

impl SqlxApiClientRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApiClientRepository for SqlxApiClientRepository {
    async fn list(
        &self,
        filter: ApiClientListFilter,
    ) -> Result<ApiClientPage, ApiClientManagementError> {
        let next_offset = filter
            .offset
            .checked_add(filter.limit)
            .ok_or(ApiClientManagementError::Validation)?;
        let search = filter.search.as_deref().map(search_pattern);
        let rows = sqlx::query_as::<_, ClientRow>(
            "SELECT client.id,client.client_id,client.display_name,client.description,
                    client.status,client.version,client.created_at,client.updated_at,
                    creator.email created_by,updater.email updated_by,
                    ARRAY(SELECT assignment.scope_code FROM api_client_allowed_scopes assignment
                          WHERE assignment.api_client_id=client.id ORDER BY assignment.scope_code) scopes
             FROM api_clients client
             JOIN staff_users creator ON creator.id=client.created_by_staff_user_id
             JOIN staff_users updater ON updater.id=client.updated_by_staff_user_id
             WHERE ($1::TEXT IS NULL OR lower(client.client_id) LIKE $1 ESCAPE '\\'
                    OR lower(client.display_name) LIKE $1 ESCAPE '\\')
               AND ($2::TEXT IS NULL OR client.status=$2)
               AND ($3::TEXT IS NULL OR EXISTS(
                    SELECT 1 FROM api_client_allowed_scopes scope_filter
                    WHERE scope_filter.api_client_id=client.id AND scope_filter.scope_code=$3))
             ORDER BY client.updated_at DESC,client.client_id
             LIMIT $4 OFFSET $5",
        )
        .bind(search.as_deref())
        .bind(filter.status.map(ApiClientStatus::as_str))
        .bind(filter.scope.map(ApiClientScope::as_str))
        .bind(filter.limit + 1)
        .bind(filter.offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| ApiClientManagementError::Infrastructure)?;
        let has_more = rows.len() > filter.limit as usize;
        let items = rows
            .into_iter()
            .take(filter.limit as usize)
            .map(ClientRow::domain)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ApiClientPage {
            items,
            next_offset: has_more.then_some(next_offset),
        })
    }

    async fn detail(&self, client_id: &str) -> Result<ApiClientDetail, ApiClientManagementError> {
        let client = load_client(&self.pool, client_id)
            .await?
            .ok_or(ApiClientManagementError::NotFound)?;
        let audit = sqlx::query_as::<_, AuditRow>(
            "SELECT staff.email actor_email,audit.action,audit.before_state,audit.after_state,
                    audit.created_at
             FROM api_client_management_audit audit
             JOIN staff_users staff ON staff.id=audit.actor_staff_user_id
             WHERE audit.api_client_id=$1
             ORDER BY audit.created_at DESC,audit.id DESC",
        )
        .bind(client.internal_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| ApiClientManagementError::Infrastructure)?
        .into_iter()
        .map(AuditRow::domain)
        .collect::<Result<Vec<_>, _>>()?;
        Ok(ApiClientDetail {
            client: client.public,
            audit,
        })
    }

    async fn scope_catalog(&self) -> Result<Vec<ApiScopeDefinition>, ApiClientManagementError> {
        sqlx::query_as::<_, ScopeRow>(
            "SELECT code,description FROM api_scope_catalog ORDER BY code",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| ApiClientManagementError::Infrastructure)?
        .into_iter()
        .map(ScopeRow::domain)
        .collect()
    }

    async fn create(
        &self,
        actor: Uuid,
        client_id: &str,
        command: &ValidatedCreateApiClient,
    ) -> Result<ApiClientRecord, ApiClientManagementError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ApiClientManagementError::Infrastructure)?;
        let id = match sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO api_clients (
                client_id,display_name,description,status,
                created_by_staff_user_id,updated_by_staff_user_id
             ) VALUES ($1,$2,$3,$4,$5,$5) RETURNING id",
        )
        .bind(client_id)
        .bind(&command.name)
        .bind(&command.description)
        .bind(command.status.as_str())
        .bind(actor)
        .fetch_one(&mut *transaction)
        .await
        {
            Ok(id) => id,
            Err(error)
                if error
                    .as_database_error()
                    .is_some_and(|db| db.is_unique_violation()) =>
            {
                return Err(ApiClientManagementError::IdentityGeneration)
            }
            Err(_) => return Err(ApiClientManagementError::Infrastructure),
        };
        replace_scopes(&mut transaction, id, actor, &command.allowed_scopes).await?;
        let snapshot = ApiClientAuditSnapshot {
            name: command.name.clone(),
            description: command.description.clone(),
            status: command.status,
            allowed_scopes: command.allowed_scopes.clone(),
        };
        insert_audit(
            &mut transaction,
            id,
            actor,
            ApiClientAuditAction::ClientCreated,
            None,
            &snapshot,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| ApiClientManagementError::Infrastructure)?;
        load_client(&self.pool, client_id)
            .await?
            .map(|value| value.public)
            .ok_or(ApiClientManagementError::Infrastructure)
    }

    async fn update(
        &self,
        actor: Uuid,
        client_id: &str,
        command: UpdateApiClientCommand,
    ) -> Result<ApiClientRecord, ApiClientManagementError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ApiClientManagementError::Infrastructure)?;
        let current = load_locked_client(&mut transaction, client_id).await?;
        if command.version < 1 {
            return Err(ApiClientManagementError::Validation);
        }
        if current.public.version != command.version {
            return Err(ApiClientManagementError::Conflict);
        }
        let validated = command.validate(current.public.status).map_err(|error| {
            if error == ApiClientValidationError::Revoked {
                ApiClientManagementError::InvalidStatus
            } else {
                ApiClientManagementError::Validation
            }
        })?;
        let metadata_changed = current.public.name != validated.name
            || current.public.description != validated.description;
        let scopes_changed = current.public.allowed_scopes != validated.allowed_scopes;
        if !metadata_changed && !scopes_changed {
            transaction
                .commit()
                .await
                .map_err(|_| ApiClientManagementError::Infrastructure)?;
            return Ok(current.public);
        }
        let before = ApiClientAuditSnapshot::from(&current.public);
        sqlx::query(
            "UPDATE api_clients SET display_name=$2,description=$3,version=version+1,
                    updated_by_staff_user_id=$4,updated_at=NOW()
             WHERE id=$1",
        )
        .bind(current.internal_id)
        .bind(&validated.name)
        .bind(&validated.description)
        .bind(actor)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiClientManagementError::Infrastructure)?;
        if scopes_changed {
            replace_scopes(
                &mut transaction,
                current.internal_id,
                actor,
                &validated.allowed_scopes,
            )
            .await?;
        }
        let after = ApiClientAuditSnapshot {
            name: validated.name,
            description: validated.description,
            status: current.public.status,
            allowed_scopes: validated.allowed_scopes,
        };
        if metadata_changed {
            insert_audit(
                &mut transaction,
                current.internal_id,
                actor,
                ApiClientAuditAction::ClientMetadataUpdated,
                Some(&before),
                &after,
            )
            .await?;
        }
        if scopes_changed {
            insert_audit(
                &mut transaction,
                current.internal_id,
                actor,
                ApiClientAuditAction::ClientScopesUpdated,
                Some(&before),
                &after,
            )
            .await?;
        }
        transaction
            .commit()
            .await
            .map_err(|_| ApiClientManagementError::Infrastructure)?;
        load_client(&self.pool, client_id)
            .await?
            .map(|value| value.public)
            .ok_or(ApiClientManagementError::Infrastructure)
    }

    async fn transition(
        &self,
        actor: Uuid,
        client_id: &str,
        version: i64,
        status: ApiClientStatus,
    ) -> Result<ApiClientRecord, ApiClientManagementError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| ApiClientManagementError::Infrastructure)?;
        let current = load_locked_client(&mut transaction, client_id).await?;
        if current.public.version != version {
            return Err(ApiClientManagementError::Conflict);
        }
        if !current
            .public
            .status
            .can_transition_to(status, !current.public.allowed_scopes.is_empty())
        {
            return Err(ApiClientManagementError::InvalidStatus);
        }
        sqlx::query(
            "UPDATE api_clients SET status=$2,version=version+1,
                    updated_by_staff_user_id=$3,updated_at=NOW() WHERE id=$1",
        )
        .bind(current.internal_id)
        .bind(status.as_str())
        .bind(actor)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiClientManagementError::Infrastructure)?;
        let before = ApiClientAuditSnapshot::from(&current.public);
        let mut after = before.clone();
        after.status = status;
        let action = match status {
            ApiClientStatus::Active => ApiClientAuditAction::ClientActivated,
            ApiClientStatus::Suspended => ApiClientAuditAction::ClientSuspended,
            ApiClientStatus::Revoked => ApiClientAuditAction::ClientRevoked,
        };
        insert_audit(
            &mut transaction,
            current.internal_id,
            actor,
            action,
            Some(&before),
            &after,
        )
        .await?;
        transaction
            .commit()
            .await
            .map_err(|_| ApiClientManagementError::Infrastructure)?;
        load_client(&self.pool, client_id)
            .await?
            .map(|value| value.public)
            .ok_or(ApiClientManagementError::Infrastructure)
    }
}

fn search_pattern(value: &str) -> String {
    format!(
        "%{}%",
        value
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}

async fn load_client(
    pool: &PgPool,
    client_id: &str,
) -> Result<Option<InternalClient>, ApiClientManagementError> {
    sqlx::query_as::<_, ClientRow>(CLIENT_SELECT)
        .bind(client_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiClientManagementError::Infrastructure)?
        .map(ClientRow::internal)
        .transpose()
}

async fn load_locked_client(
    transaction: &mut Transaction<'_, Postgres>,
    client_id: &str,
) -> Result<InternalClient, ApiClientManagementError> {
    sqlx::query_as::<_, ClientRow>(CLIENT_SELECT_LOCKED)
        .bind(client_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(|_| ApiClientManagementError::Infrastructure)?
        .ok_or(ApiClientManagementError::NotFound)?
        .internal()
}

const CLIENT_SELECT: &str =
    "SELECT client.id,client.client_id,client.display_name,client.description,
            client.status,client.version,client.created_at,client.updated_at,
            creator.email created_by,updater.email updated_by,
            ARRAY(SELECT assignment.scope_code FROM api_client_allowed_scopes assignment
                  WHERE assignment.api_client_id=client.id ORDER BY assignment.scope_code) scopes
     FROM api_clients client
     JOIN staff_users creator ON creator.id=client.created_by_staff_user_id
     JOIN staff_users updater ON updater.id=client.updated_by_staff_user_id
     WHERE client.client_id=$1";

const CLIENT_SELECT_LOCKED: &str =
    "SELECT client.id,client.client_id,client.display_name,client.description,
            client.status,client.version,client.created_at,client.updated_at,
            creator.email created_by,updater.email updated_by,
            ARRAY(SELECT assignment.scope_code FROM api_client_allowed_scopes assignment
                  WHERE assignment.api_client_id=client.id ORDER BY assignment.scope_code) scopes
     FROM api_clients client
     JOIN staff_users creator ON creator.id=client.created_by_staff_user_id
     JOIN staff_users updater ON updater.id=client.updated_by_staff_user_id
     WHERE client.client_id=$1 FOR UPDATE OF client";

async fn replace_scopes(
    transaction: &mut Transaction<'_, Postgres>,
    client_id: Uuid,
    actor: Uuid,
    scopes: &[ApiClientScope],
) -> Result<(), ApiClientManagementError> {
    sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1")
        .bind(client_id)
        .execute(&mut **transaction)
        .await
        .map_err(|_| ApiClientManagementError::Infrastructure)?;
    let values: Vec<&str> = scopes.iter().map(|scope| scope.as_str()).collect();
    if !values.is_empty() {
        sqlx::query(
            "INSERT INTO api_client_allowed_scopes (
                api_client_id,scope_code,assigned_by_staff_user_id
             ) SELECT $1,scope_code,$3 FROM unnest($2::TEXT[]) scope_code",
        )
        .bind(client_id)
        .bind(values)
        .bind(actor)
        .execute(&mut **transaction)
        .await
        .map_err(|_| ApiClientManagementError::Infrastructure)?;
    }
    Ok(())
}

async fn insert_audit(
    transaction: &mut Transaction<'_, Postgres>,
    client_id: Uuid,
    actor: Uuid,
    action: ApiClientAuditAction,
    before: Option<&ApiClientAuditSnapshot>,
    after: &ApiClientAuditSnapshot,
) -> Result<(), ApiClientManagementError> {
    sqlx::query(
        "INSERT INTO api_client_management_audit (
            api_client_id,actor_staff_user_id,action,before_state,after_state,created_at
         ) VALUES ($1,$2,$3,$4,$5,clock_timestamp())",
    )
    .bind(client_id)
    .bind(actor)
    .bind(action.as_str())
    .bind(
        before
            .map(serde_json::to_value)
            .transpose()
            .map_err(|_| ApiClientManagementError::Infrastructure)?,
    )
    .bind(serde_json::to_value(after).map_err(|_| ApiClientManagementError::Infrastructure)?)
    .execute(&mut **transaction)
    .await
    .map_err(|_| ApiClientManagementError::Infrastructure)?;
    Ok(())
}

#[derive(FromRow)]
struct ClientRow {
    id: Uuid,
    client_id: String,
    display_name: String,
    description: Option<String>,
    status: String,
    version: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    created_by: String,
    updated_by: String,
    scopes: Vec<String>,
}

impl ClientRow {
    fn domain(self) -> Result<ApiClientRecord, ApiClientManagementError> {
        self.internal().map(|value| value.public)
    }

    fn internal(self) -> Result<InternalClient, ApiClientManagementError> {
        let scopes = self
            .scopes
            .iter()
            .map(|value| ApiClientScope::parse(value))
            .collect::<Option<Vec<_>>>()
            .ok_or(ApiClientManagementError::Infrastructure)?;
        let status =
            ApiClientStatus::parse(&self.status).ok_or(ApiClientManagementError::Infrastructure)?;
        Ok(InternalClient {
            internal_id: self.id,
            public: ApiClientRecord {
                client_id: self.client_id,
                name: self.display_name,
                description: self.description,
                status,
                allowed_scopes: scopes,
                version: self.version,
                created_at: self.created_at,
                updated_at: self.updated_at,
                created_by: self.created_by,
                updated_by: self.updated_by,
            },
        })
    }
}

struct InternalClient {
    internal_id: Uuid,
    public: ApiClientRecord,
}

#[derive(FromRow)]
struct AuditRow {
    actor_email: String,
    action: String,
    before_state: Option<serde_json::Value>,
    after_state: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl AuditRow {
    fn domain(self) -> Result<ApiClientAuditEntry, ApiClientManagementError> {
        Ok(ApiClientAuditEntry {
            actor_email: self.actor_email,
            action: ApiClientAuditAction::parse(&self.action)
                .ok_or(ApiClientManagementError::Infrastructure)?,
            before: self
                .before_state
                .map(serde_json::from_value)
                .transpose()
                .map_err(|_| ApiClientManagementError::Infrastructure)?,
            after: serde_json::from_value(self.after_state)
                .map_err(|_| ApiClientManagementError::Infrastructure)?,
            created_at: self.created_at,
        })
    }
}

#[derive(FromRow)]
struct ScopeRow {
    code: String,
    description: String,
}

impl ScopeRow {
    fn domain(self) -> Result<ApiScopeDefinition, ApiClientManagementError> {
        Ok(ApiScopeDefinition {
            code: ApiClientScope::parse(&self.code)
                .ok_or(ApiClientManagementError::Infrastructure)?,
            description: self.description,
        })
    }
}
