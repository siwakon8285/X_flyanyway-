mod common;

use std::{sync::Arc, sync::OnceLock};

use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;
use x_fly_api::{
    application::api_client::{
        ApiClientListFilter, ApiClientManagement, ApiClientManagementError, ApiClientRepository,
    },
    domain::api_client::{
        ApiClientScope, ApiClientStatus, CreateApiClientCommand, UpdateApiClientCommand,
    },
    infrastructure::database::{prepare_test_database, SqlxApiClientRepository},
};

async fn fixture_guard() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn test_pool() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&common::test_database_url())
        .await
        .unwrap();
    prepare_test_database(&pool).await.unwrap();
    clean_api_client_fixtures(&pool).await;
    pool
}

async fn clean_api_client_fixtures(pool: &PgPool) {
    sqlx::raw_sql(
        "DELETE FROM api_client_management_audit;
         DELETE FROM api_client_allowed_scopes;
         DELETE FROM api_clients;
         DELETE FROM staff_sessions;
         DELETE FROM staff_user_roles;
         DELETE FROM staff_users WHERE email LIKE '%@api-client-repo.test';",
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn actor(pool: &PgPool, label: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash) VALUES ($1,'hash') RETURNING id",
    )
    .bind(format!("{label}@api-client-repo.test"))
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn persisted_scope_codes(pool: &PgPool, client_id: &str) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT ARRAY(
            SELECT assignment.scope_code
            FROM api_client_allowed_scopes assignment
            WHERE assignment.api_client_id=client.id
            ORDER BY assignment.scope_code
         )
         FROM api_clients client WHERE client.client_id=$1",
    )
    .bind(client_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

fn create(
    name: &str,
    status: ApiClientStatus,
    scopes: Vec<ApiClientScope>,
) -> CreateApiClientCommand {
    CreateApiClientCommand {
        name: name.to_owned(),
        description: Some("Approved aggregate integration".to_owned()),
        status,
        allowed_scopes: scopes,
    }
}

#[tokio::test]
async fn creates_unique_public_clients_and_returns_only_public_dto_fields() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let actor = actor(&pool, "creator").await;
    let service = ApiClientManagement::new(Arc::new(SqlxApiClientRepository::new(pool.clone())));

    let first = service
        .create(
            actor,
            create(
                "Marketing Insights",
                ApiClientStatus::Active,
                vec![ApiClientScope::AnalyticsRead],
            ),
        )
        .await
        .unwrap();
    let second = service
        .create(
            actor,
            create(
                "Flight Display",
                ApiClientStatus::Suspended,
                vec![ApiClientScope::FlightsRead],
            ),
        )
        .await
        .unwrap();

    assert!(first.client_id.starts_with("XFC"));
    assert_eq!(first.client_id.len(), 19);
    assert_ne!(first.client_id, second.client_id);
    assert_eq!(first.created_by, "creator@api-client-repo.test");
    assert_eq!(first.allowed_scopes, vec![ApiClientScope::AnalyticsRead]);
    assert_eq!(second.allowed_scopes, vec![ApiClientScope::FlightsRead]);
    let json = serde_json::to_value(&first).unwrap();
    assert!(json.get("id").is_none());
    assert!(json.get("secret").is_none());
    assert!(json.get("token").is_none());

    let scope_catalog = service.scope_catalog().await.unwrap();
    assert_eq!(scope_catalog.len(), 2);
    assert_eq!(scope_catalog[0].code, ApiClientScope::AnalyticsRead);
    assert_eq!(scope_catalog[1].code, ApiClientScope::FlightsRead);
    clean_api_client_fixtures(&pool).await;
}

#[tokio::test]
async fn canonical_scope_codes_survive_create_detail_persistence_audit_and_edit() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let actor = actor(&pool, "scope-integrity").await;
    let service = ApiClientManagement::new(Arc::new(SqlxApiClientRepository::new(pool.clone())));

    let analytics = service
        .create(
            actor,
            create(
                "Analytics only",
                ApiClientStatus::Active,
                vec![ApiClientScope::AnalyticsRead],
            ),
        )
        .await
        .unwrap();
    let flights = service
        .create(
            actor,
            create(
                "Flights only",
                ApiClientStatus::Active,
                vec![ApiClientScope::FlightsRead],
            ),
        )
        .await
        .unwrap();
    let both = service
        .create(
            actor,
            create(
                "Both reversed",
                ApiClientStatus::Active,
                vec![ApiClientScope::FlightsRead, ApiClientScope::AnalyticsRead],
            ),
        )
        .await
        .unwrap();

    for (created, expected_domain, expected_codes) in [
        (
            &analytics,
            vec![ApiClientScope::AnalyticsRead],
            vec!["analytics:read".to_owned()],
        ),
        (
            &flights,
            vec![ApiClientScope::FlightsRead],
            vec!["flights:read".to_owned()],
        ),
        (
            &both,
            vec![ApiClientScope::AnalyticsRead, ApiClientScope::FlightsRead],
            vec!["analytics:read".to_owned(), "flights:read".to_owned()],
        ),
    ] {
        assert_eq!(created.allowed_scopes, expected_domain);
        assert_eq!(
            persisted_scope_codes(&pool, &created.client_id).await,
            expected_codes
        );
        let detail = service.detail(&created.client_id).await.unwrap();
        assert_eq!(detail.client.allowed_scopes, expected_domain);
        assert_eq!(detail.audit[0].after.allowed_scopes, expected_domain);
    }

    let updated = service
        .update(
            actor,
            &analytics.client_id,
            UpdateApiClientCommand {
                name: analytics.name.clone(),
                description: analytics.description.clone(),
                allowed_scopes: vec![ApiClientScope::FlightsRead],
                version: analytics.version,
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.allowed_scopes, vec![ApiClientScope::FlightsRead]);
    assert_eq!(
        persisted_scope_codes(&pool, &analytics.client_id).await,
        vec!["flights:read"]
    );
    let detail = service.detail(&analytics.client_id).await.unwrap();
    assert_eq!(
        detail.client.allowed_scopes,
        vec![ApiClientScope::FlightsRead]
    );
    assert_eq!(
        detail.audit[0].before.as_ref().unwrap().allowed_scopes,
        vec![ApiClientScope::AnalyticsRead]
    );
    assert_eq!(
        detail.audit[0].after.allowed_scopes,
        vec![ApiClientScope::FlightsRead]
    );
    clean_api_client_fixtures(&pool).await;
}

#[tokio::test]
async fn lists_with_server_filters_and_bounded_next_offset() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let actor = actor(&pool, "filter").await;
    let service = ApiClientManagement::new(Arc::new(SqlxApiClientRepository::new(pool.clone())));
    for (name, status, scope) in [
        (
            "Alpha Analytics",
            ApiClientStatus::Active,
            ApiClientScope::AnalyticsRead,
        ),
        (
            "Beta Flight Board",
            ApiClientStatus::Suspended,
            ApiClientScope::FlightsRead,
        ),
        (
            "Gamma Analytics",
            ApiClientStatus::Suspended,
            ApiClientScope::AnalyticsRead,
        ),
    ] {
        service
            .create(actor, create(name, status, vec![scope]))
            .await
            .unwrap();
    }

    let page = service
        .list(ApiClientListFilter {
            search: Some("analytics".to_owned()),
            status: None,
            scope: Some(ApiClientScope::AnalyticsRead),
            limit: 1,
            offset: 0,
        })
        .await
        .unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.next_offset, Some(1));
    assert!(page.items[0].name.contains("Analytics"));

    let injection = service
        .list(ApiClientListFilter {
            search: Some("%' OR TRUE --".to_owned()),
            status: None,
            scope: None,
            limit: 50,
            offset: 0,
        })
        .await
        .unwrap();
    assert!(injection.items.is_empty());
    clean_api_client_fixtures(&pool).await;
}

#[tokio::test]
async fn repository_rejects_pagination_offsets_that_cannot_advance() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let repository = SqlxApiClientRepository::new(pool.clone());

    let result = repository
        .list(ApiClientListFilter {
            search: None,
            status: None,
            scope: None,
            limit: 50,
            offset: i64::MAX,
        })
        .await;
    assert!(matches!(result, Err(ApiClientManagementError::Validation)));

    clean_api_client_fixtures(&pool).await;
}

#[tokio::test]
async fn updates_metadata_and_scopes_without_changing_public_identity() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let creator = actor(&pool, "update-creator").await;
    let editor = actor(&pool, "update-editor").await;
    let service = ApiClientManagement::new(Arc::new(SqlxApiClientRepository::new(pool.clone())));
    let created = service
        .create(
            creator,
            create("Original", ApiClientStatus::Suspended, vec![]),
        )
        .await
        .unwrap();

    let updated = service
        .update(
            editor,
            &created.client_id,
            UpdateApiClientCommand {
                name: "Updated Client".to_owned(),
                description: None,
                allowed_scopes: vec![ApiClientScope::FlightsRead],
                version: created.version,
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.client_id, created.client_id);
    assert_eq!(updated.name, "Updated Client");
    assert_eq!(updated.updated_by, "update-editor@api-client-repo.test");
    assert_eq!(updated.version, 2);
    assert!(service
        .update(
            editor,
            &created.client_id,
            UpdateApiClientCommand {
                name: "Stale".to_owned(),
                description: None,
                allowed_scopes: vec![ApiClientScope::FlightsRead],
                version: 1,
            },
        )
        .await
        .is_err());

    let detail = service.detail(&created.client_id).await.unwrap();
    let actions: Vec<_> = detail
        .audit
        .iter()
        .map(|entry| entry.action.as_str())
        .collect();
    assert_eq!(
        actions,
        vec![
            "CLIENT_SCOPES_UPDATED",
            "CLIENT_METADATA_UPDATED",
            "CLIENT_CREATED"
        ]
    );
    clean_api_client_fixtures(&pool).await;
}

#[tokio::test]
async fn lifecycle_is_explicit_and_revocation_preserves_terminal_history() {
    let _guard = fixture_guard().await;
    let pool = test_pool().await;
    let actor = actor(&pool, "lifecycle").await;
    let service = ApiClientManagement::new(Arc::new(SqlxApiClientRepository::new(pool.clone())));
    let created = service
        .create(
            actor,
            create(
                "Lifecycle",
                ApiClientStatus::Suspended,
                vec![ApiClientScope::FlightsRead],
            ),
        )
        .await
        .unwrap();
    let active = service
        .transition(
            actor,
            &created.client_id,
            created.version,
            ApiClientStatus::Active,
        )
        .await
        .unwrap();
    let revoked = service
        .transition(
            actor,
            &created.client_id,
            active.version,
            ApiClientStatus::Revoked,
        )
        .await
        .unwrap();
    assert_eq!(revoked.status, ApiClientStatus::Revoked);
    assert!(service
        .transition(
            actor,
            &created.client_id,
            revoked.version,
            ApiClientStatus::Active
        )
        .await
        .is_err());
    assert!(service
        .update(
            actor,
            &created.client_id,
            UpdateApiClientCommand {
                name: "Cannot rewrite history".to_owned(),
                description: None,
                allowed_scopes: vec![ApiClientScope::FlightsRead],
                version: revoked.version,
            },
        )
        .await
        .is_err());

    let row_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_clients WHERE client_id=$1")
        .bind(&created.client_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row_count, 1);
    let detail = service.detail(&created.client_id).await.unwrap();
    assert_eq!(detail.client.status, ApiClientStatus::Revoked);
    assert_eq!(detail.audit[0].action.as_str(), "CLIENT_REVOKED");
    clean_api_client_fixtures(&pool).await;
}
