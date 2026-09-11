use async_trait::async_trait;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[cfg(test)]
use std::sync::atomic::{AtomicBool, Ordering};

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

#[cfg(test)]
static FAIL_NEXT_CLIENT_RELOAD: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
fn fail_next_client_reload_for_test() {
    FAIL_NEXT_CLIENT_RELOAD.store(true, Ordering::SeqCst);
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
        let response = load_client_in_transaction(&mut transaction, client_id)
            .await?
            .map(|value| value.public)
            .ok_or(ApiClientManagementError::Infrastructure)?;
        transaction
            .commit()
            .await
            .map_err(|_| ApiClientManagementError::Infrastructure)?;
        Ok(response)
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
        let response = load_client_in_transaction(&mut transaction, client_id)
            .await?
            .map(|value| value.public)
            .ok_or(ApiClientManagementError::Infrastructure)?;
        transaction
            .commit()
            .await
            .map_err(|_| ApiClientManagementError::Infrastructure)?;
        Ok(response)
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
        let response = load_client_in_transaction(&mut transaction, client_id)
            .await?
            .map(|value| value.public)
            .ok_or(ApiClientManagementError::Infrastructure)?;
        transaction
            .commit()
            .await
            .map_err(|_| ApiClientManagementError::Infrastructure)?;
        Ok(response)
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
    #[cfg(test)]
    if FAIL_NEXT_CLIENT_RELOAD.swap(false, Ordering::SeqCst) {
        return Err(ApiClientManagementError::Infrastructure);
    }
    sqlx::query_as::<_, ClientRow>(CLIENT_SELECT)
        .bind(client_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiClientManagementError::Infrastructure)?
        .map(ClientRow::internal)
        .transpose()
}

async fn load_client_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    client_id: &str,
) -> Result<Option<InternalClient>, ApiClientManagementError> {
    sqlx::query_as::<_, ClientRow>(CLIENT_SELECT)
        .bind(client_id)
        .fetch_optional(&mut **transaction)
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

#[cfg(test)]
mod tests {
    use std::{env, path::Path, str::FromStr, sync::OnceLock};

    use sqlx::{
        postgres::{PgConnectOptions, PgPoolOptions},
        PgPool,
    };

    use super::*;
    use crate::infrastructure::database::prepare_test_database;

    async fn test_guard() -> tokio::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
            .lock()
            .await
    }

    async fn test_pool() -> PgPool {
        let _ = dotenvy::from_path(Path::new(env!("CARGO_MANIFEST_DIR")).join(".env"));
        let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL is required");
        let options = PgConnectOptions::from_str(&database_url).expect("valid TEST URL");
        assert_eq!(options.get_port(), 5434);
        assert_eq!(options.get_database(), Some("x_fly_concurrency_test"));
        let pool = PgPoolOptions::new()
            .max_connections(3)
            .connect_with(options)
            .await
            .expect("TEST database is reachable");
        prepare_test_database(&pool)
            .await
            .expect("TEST database is prepared");
        pool
    }

    async fn cleanup(pool: &PgPool, actor: Uuid) {
        let client_ids: Vec<Uuid> =
            sqlx::query_scalar("SELECT id FROM api_clients WHERE created_by_staff_user_id=$1")
                .bind(actor)
                .fetch_all(pool)
                .await
                .expect("fixture query");
        for client_id in client_ids {
            sqlx::query("DELETE FROM api_client_management_audit WHERE api_client_id=$1")
                .bind(client_id)
                .execute(pool)
                .await
                .expect("audit cleanup");
            sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1")
                .bind(client_id)
                .execute(pool)
                .await
                .expect("scope cleanup");
            sqlx::query("DELETE FROM api_clients WHERE id=$1")
                .bind(client_id)
                .execute(pool)
                .await
                .expect("client cleanup");
        }
        sqlx::query("DELETE FROM staff_users WHERE id=$1")
            .bind(actor)
            .execute(pool)
            .await
            .expect("actor cleanup");
    }

    async fn fixture_actor(pool: &PgPool, label: &str) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO staff_users (email,password_hash) VALUES ($1,'hash') RETURNING id",
        )
        .bind(format!("{label}-{}@api-client-red.test", Uuid::new_v4()))
        .fetch_one(pool)
        .await
        .expect("actor fixture")
    }

    #[tokio::test]
    async fn create_materializes_response_before_commit() {
        let _guard = test_guard().await;
        let pool = test_pool().await;
        let marker = Uuid::new_v4();
        let actor = fixture_actor(&pool, "f04-create").await;
        let repository = SqlxApiClientRepository::new(pool.clone());
        const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let suffix: String = marker
            .as_bytes()
            .iter()
            .map(|byte| ALPHABET[(*byte & 31) as usize] as char)
            .collect();
        let client_id = format!("XFC{suffix}");
        let command = ValidatedCreateApiClient {
            name: "Response boundary test".to_owned(),
            description: Some("must not report a committed mutation as failed".to_owned()),
            status: ApiClientStatus::Active,
            allowed_scopes: vec![ApiClientScope::AnalyticsRead],
        };

        fail_next_client_reload_for_test();
        let result = repository.create(actor, &client_id, &command).await;
        let persisted_clients: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM api_clients WHERE created_by_staff_user_id=$1",
        )
        .bind(actor)
        .fetch_one(&pool)
        .await
        .expect("client count");
        let persisted_scopes: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM api_client_allowed_scopes scope
             JOIN api_clients client ON client.id=scope.api_client_id
             WHERE client.created_by_staff_user_id=$1",
        )
        .bind(actor)
        .fetch_one(&pool)
        .await
        .expect("scope count");
        let persisted_audits: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM api_client_management_audit audit
             JOIN api_clients client ON client.id=audit.api_client_id
             WHERE client.created_by_staff_user_id=$1",
        )
        .bind(actor)
        .fetch_one(&pool)
        .await
        .expect("audit count");
        cleanup(&pool, actor).await;

        assert!(result.is_ok());
        assert_eq!(persisted_clients, 1);
        assert_eq!(persisted_scopes, 1);
        assert_eq!(persisted_audits, 1);
    }

    #[tokio::test]
    async fn update_materializes_response_before_commit() {
        let _guard = test_guard().await;
        let pool = test_pool().await;
        let actor = fixture_actor(&pool, "f04-update").await;
        let repository = SqlxApiClientRepository::new(pool.clone());
        let client_id = {
            const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
            let suffix: String = Uuid::new_v4()
                .as_bytes()
                .iter()
                .map(|byte| ALPHABET[(*byte & 31) as usize] as char)
                .collect();
            format!("XFC{suffix}")
        };
        let created = repository
            .create(
                actor,
                &client_id,
                &ValidatedCreateApiClient {
                    name: "Update boundary test".to_owned(),
                    description: Some("before".to_owned()),
                    status: ApiClientStatus::Suspended,
                    allowed_scopes: vec![ApiClientScope::FlightsRead],
                },
            )
            .await
            .expect("create fixture");
        fail_next_client_reload_for_test();
        let result = repository
            .update(
                actor,
                &client_id,
                UpdateApiClientCommand {
                    name: "Updated boundary test".to_owned(),
                    description: Some("after".to_owned()),
                    allowed_scopes: vec![ApiClientScope::AnalyticsRead],
                    version: created.version,
                },
            )
            .await;
        cleanup(&pool, actor).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn transition_materializes_response_before_commit() {
        let _guard = test_guard().await;
        let pool = test_pool().await;
        let actor = fixture_actor(&pool, "f04-transition").await;
        let repository = SqlxApiClientRepository::new(pool.clone());
        let client_id = {
            const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
            let suffix: String = Uuid::new_v4()
                .as_bytes()
                .iter()
                .map(|byte| ALPHABET[(*byte & 31) as usize] as char)
                .collect();
            format!("XFC{suffix}")
        };
        let created = repository
            .create(
                actor,
                &client_id,
                &ValidatedCreateApiClient {
                    name: "Transition boundary test".to_owned(),
                    description: None,
                    status: ApiClientStatus::Suspended,
                    allowed_scopes: vec![ApiClientScope::FlightsRead],
                },
            )
            .await
            .expect("create fixture");
        fail_next_client_reload_for_test();
        let result = repository
            .transition(actor, &client_id, created.version, ApiClientStatus::Active)
            .await;
        cleanup(&pool, actor).await;
        assert!(result.is_ok());
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
