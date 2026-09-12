mod common;

use sqlx::{PgPool, Postgres, Row, Transaction};
use x_fly_api::infrastructure::database::{
    migrate_database, seed_demo_database, verify_database_ready, DatabaseCommand,
};

#[test]
fn database_operator_accepts_only_explicit_lifecycle_commands() {
    assert_eq!(
        DatabaseCommand::parse("migrate"),
        Some(DatabaseCommand::Migrate)
    );
    assert_eq!(
        DatabaseCommand::parse("seed-demo"),
        Some(DatabaseCommand::SeedDemo)
    );
    assert_eq!(DatabaseCommand::parse("start"), None);
}

#[tokio::test]
async fn migrations_and_demo_seed_are_independent_and_seed_refuses_unsafe_targets() {
    let setup_url = common::test_database_url();
    let pool = PgPool::connect(&setup_url).await.unwrap();

    migrate_database(&pool).await.unwrap();
    assert!(seed_demo_database(&pool, "not_the_test_database")
        .await
        .is_err());
    let database_name: String = sqlx::query("SELECT current_database() AS database_name")
        .fetch_one(&pool)
        .await
        .unwrap()
        .get("database_name");
    assert!(seed_demo_database(&pool, &database_name).await.is_err());
    verify_database_ready(&pool).await.unwrap();
}

#[tokio::test]
async fn migrator_owns_application_objects_without_cluster_admin_privileges() {
    let setup_url = common::test_database_url();
    let pool = PgPool::connect(&setup_url).await.unwrap();
    migrate_database(&pool).await.unwrap();

    let role = sqlx::query(
        "SELECT current_user AS role_name, rolsuper, rolcreatedb, rolcreaterole,
                rolreplication, rolbypassrls
         FROM pg_roles WHERE rolname = current_user",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(role.get::<String, _>("role_name"), "x_fly_migrator");
    assert!(!role.get::<bool, _>("rolsuper"));
    assert!(!role.get::<bool, _>("rolcreatedb"));
    assert!(!role.get::<bool, _>("rolcreaterole"));
    assert!(!role.get::<bool, _>("rolreplication"));
    assert!(!role.get::<bool, _>("rolbypassrls"));

    let foreign_table_owners: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pg_tables
         WHERE schemaname = 'public' AND tableowner <> current_user",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(foreign_table_owners, 0);
    assert!(sqlx::query("CREATE ROLE migrator_must_not_create")
        .execute(&pool)
        .await
        .is_err());
    assert!(sqlx::query("CREATE DATABASE migrator_must_not_create")
        .execute(&pool)
        .await
        .is_err());
}

async fn migrated_test_pool() -> PgPool {
    let pool = PgPool::connect(&common::test_database_url()).await.unwrap();
    migrate_database(&pool).await.unwrap();
    pool
}

fn constraint_fixture_public_client_id() -> String {
    let suffix: String = uuid::Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(16)
        .map(|value| match value {
            '0' => '2',
            '1' => '3',
            value => value.to_ascii_uppercase(),
        })
        .collect();
    format!("XFC{suffix}")
}

async fn insert_constraint_fixture(
    transaction: &mut Transaction<'_, Postgres>,
) -> (uuid::Uuid, uuid::Uuid) {
    let staff_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email, password_hash)
         VALUES ($1, 'task2-constraint-fixture') RETURNING id",
    )
    .bind(format!(
        "task2-constraint-{}@x-fly.test",
        uuid::Uuid::new_v4().simple()
    ))
    .fetch_one(&mut **transaction)
    .await
    .unwrap();
    let client_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO api_clients (
             client_id, display_name, description, status,
             created_by_staff_user_id, updated_by_staff_user_id
         ) VALUES ($1, 'Task 2 constraint fixture', NULL, 'ACTIVE', $2, $2)
         RETURNING id",
    )
    .bind(constraint_fixture_public_client_id())
    .bind(staff_id)
    .fetch_one(&mut **transaction)
    .await
    .unwrap();
    (client_id, staff_id)
}

async fn insert_constraint_credential(
    transaction: &mut Transaction<'_, Postgres>,
    client_id: uuid::Uuid,
    staff_id: uuid::Uuid,
    digest: [u8; 32],
) -> uuid::Uuid {
    sqlx::query_scalar(
        "INSERT INTO api_client_credentials (
             api_client_id, secret_digest, digest_version, issued_at,
             issued_by_staff_user_id
         ) VALUES ($1, $2, 1, NOW(), $3)
         RETURNING id",
    )
    .bind(client_id)
    .bind(digest.to_vec())
    .bind(staff_id)
    .fetch_one(&mut **transaction)
    .await
    .unwrap()
}

fn assert_constraint_error(error: sqlx::Error, code: &str, constraint: &str) {
    let database_error = error
        .as_database_error()
        .expect("expected a PostgreSQL constraint error");
    assert!(database_error.code().is_some_and(|value| value == code));
    assert_eq!(database_error.constraint(), Some(constraint));
}

#[tokio::test]
async fn fresh_schema_counts_35_application_tables() {
    let pool = migrated_test_pool().await;
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM pg_tables
         WHERE schemaname = 'public' AND tablename <> '_sqlx_migrations'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 35);
}

#[tokio::test]
async fn fresh_schema_counts_36_total_public_tables() {
    let pool = migrated_test_pool().await;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM pg_tables WHERE schemaname = 'public'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 36);
}

#[tokio::test]
async fn fresh_schema_counts_36_migrator_owned_public_tables() {
    let pool = migrated_test_pool().await;
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM pg_tables
         WHERE schemaname = 'public' AND tableowner = 'x_fly_migrator'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 36);
}

#[tokio::test]
async fn fresh_schema_has_27_successful_migrations() {
    let pool = migrated_test_pool().await;
    let status: String = sqlx::query_scalar(
        "SELECT COUNT(*) FILTER (WHERE success)::text || '|' || COUNT(*)::text
         FROM _sqlx_migrations",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(status, "27|27");
}

#[tokio::test]
async fn branch24_audit_actions_remain_valid() {
    let pool = migrated_test_pool().await;
    let definition: String = sqlx::query_scalar(
        "SELECT pg_get_constraintdef(oid)
         FROM pg_constraint
         WHERE conrelid = 'public.api_client_management_audit'::regclass
           AND conname = 'api_client_management_audit_action_check'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    for action in [
        "CLIENT_CREATED",
        "CLIENT_METADATA_UPDATED",
        "CLIENT_SCOPES_UPDATED",
        "CLIENT_ACTIVATED",
        "CLIENT_SUSPENDED",
        "CLIENT_REVOKED",
    ] {
        assert!(definition.contains(action), "missing Branch 24 action");
    }
}

#[tokio::test]
async fn credential_actions_require_credential_id() {
    let pool = migrated_test_pool().await;
    let definition: String = sqlx::query_scalar(
        "SELECT pg_get_constraintdef(oid)
         FROM pg_constraint
         WHERE conrelid = 'public.api_client_management_audit'::regclass
           AND conname = 'api_client_management_audit_credential_context_check'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(definition.contains("CREDENTIAL_ISSUED"));
    assert!(definition.contains("CREDENTIAL_REVOKED"));
    assert!(definition.contains("credential_id IS NOT NULL"));
    assert!(definition.contains("credential_id IS NULL"));
}

#[tokio::test]
async fn credential_correlation_fk_is_restrict() {
    let pool = migrated_test_pool().await;
    let delete_action: String = sqlx::query_scalar(
        "SELECT confdeltype::text
         FROM pg_constraint
         WHERE conrelid = 'public.api_client_management_audit'::regclass
           AND conname = 'api_client_management_audit_credential_id_fkey'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(delete_action, "r");
}

#[tokio::test]
async fn external_auth_tables_have_expected_columns_and_nullability() {
    let pool = migrated_test_pool().await;
    let missing_columns: i64 = sqlx::query_scalar(
        "WITH expected(table_name, column_name, data_type, is_nullable) AS (
             VALUES
                 ('api_client_credentials', 'id', 'uuid', 'NO'),
                 ('api_client_credentials', 'api_client_id', 'uuid', 'NO'),
                 ('api_client_credentials', 'secret_digest', 'bytea', 'NO'),
                 ('api_client_credentials', 'digest_version', 'smallint', 'NO'),
                 ('api_client_credentials', 'issued_at', 'timestamp with time zone', 'NO'),
                 ('api_client_credentials', 'issued_by_staff_user_id', 'uuid', 'NO'),
                 ('api_client_credentials', 'revoked_at', 'timestamp with time zone', 'YES'),
                 ('api_client_credentials', 'revoked_by_staff_user_id', 'uuid', 'YES'),
                 ('api_client_credentials', 'revocation_reason', 'text', 'YES'),
                 ('external_access_tokens', 'id', 'uuid', 'NO'),
                 ('external_access_tokens', 'api_client_credential_id', 'uuid', 'NO'),
                 ('external_access_tokens', 'token_hash', 'bytea', 'NO'),
                 ('external_access_tokens', 'issued_at', 'timestamp with time zone', 'NO'),
                 ('external_access_tokens', 'expires_at', 'timestamp with time zone', 'NO'),
                 ('external_access_tokens', 'revoked_at', 'timestamp with time zone', 'YES'),
                 ('api_client_management_audit', 'credential_id', 'uuid', 'YES')
         )
         SELECT COUNT(*)
         FROM expected
         LEFT JOIN information_schema.columns actual
           ON actual.table_schema = 'public'
          AND actual.table_name = expected.table_name
          AND actual.column_name = expected.column_name
          AND actual.data_type = expected.data_type
          AND actual.is_nullable = expected.is_nullable
         WHERE actual.column_name IS NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(missing_columns, 0);
}

#[tokio::test]
async fn external_auth_schema_has_required_constraints_and_indexes() {
    let pool = migrated_test_pool().await;
    let missing_objects: i64 = sqlx::query_scalar(
        "WITH expected(name) AS (
             VALUES
                 ('api_client_credentials_api_client_id_fkey'),
                 ('api_client_credentials_issued_by_staff_user_id_fkey'),
                 ('api_client_credentials_revoked_by_staff_user_id_fkey'),
                 ('api_client_credentials_secret_digest_check'),
                 ('api_client_credentials_digest_version_check'),
                 ('api_client_credentials_revocation_reason_check'),
                 ('api_client_credentials_one_live_idx'),
                 ('external_access_tokens_api_client_credential_id_fkey'),
                 ('external_access_tokens_token_hash_key'),
                 ('external_access_tokens_token_hash_length_check'),
                 ('external_access_tokens_expiry_check'),
                 ('external_access_tokens_credential_live_idx'),
                 ('api_client_management_audit_credential_id_fkey'),
                 ('api_client_management_audit_credential_idx'),
                 ('api_client_management_audit_action_check'),
                 ('api_client_management_audit_credential_context_check')
         )
         SELECT COUNT(*)
         FROM expected
         LEFT JOIN (
             SELECT conname AS name FROM pg_constraint
             UNION ALL
             SELECT indexname AS name FROM pg_indexes WHERE schemaname = 'public'
         ) actual ON actual.name = expected.name
         WHERE actual.name IS NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(missing_objects, 0);

    let restrict_fks: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM pg_constraint
         WHERE conname IN (
             'api_client_credentials_api_client_id_fkey',
             'api_client_credentials_issued_by_staff_user_id_fkey',
             'api_client_credentials_revoked_by_staff_user_id_fkey',
             'external_access_tokens_api_client_credential_id_fkey',
             'api_client_management_audit_credential_id_fkey'
         )
           AND contype = 'f'
           AND confdeltype = 'r'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(restrict_fks, 5);

    let digest_check: String = sqlx::query_scalar(
        "SELECT pg_get_constraintdef(oid)
         FROM pg_constraint
         WHERE conname = 'api_client_credentials_secret_digest_check'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(digest_check.contains("octet_length(secret_digest) = 32"));

    let digest_version_check: String = sqlx::query_scalar(
        "SELECT pg_get_constraintdef(oid)
         FROM pg_constraint
         WHERE conname = 'api_client_credentials_digest_version_check'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(digest_version_check.contains("digest_version = 1"));

    let reason_check: String = sqlx::query_scalar(
        "SELECT pg_get_constraintdef(oid)
         FROM pg_constraint
         WHERE conname = 'api_client_credentials_revocation_reason_check'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    for reason in [
        "ADMIN_REQUEST",
        "CLIENT_SUSPENDED",
        "CLIENT_REVOKED",
        "REPLACED",
    ] {
        assert!(reason_check.contains(reason));
    }

    let token_hash_check: String = sqlx::query_scalar(
        "SELECT pg_get_constraintdef(oid)
         FROM pg_constraint
         WHERE conname = 'external_access_tokens_token_hash_length_check'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(token_hash_check.contains("octet_length(token_hash) = 32"));

    let expiry_check: String = sqlx::query_scalar(
        "SELECT pg_get_constraintdef(oid)
         FROM pg_constraint
         WHERE conname = 'external_access_tokens_expiry_check'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(expiry_check.contains("expires_at > issued_at"));

    let credential_index: String = sqlx::query_scalar(
        "SELECT indexdef FROM pg_indexes
         WHERE schemaname = 'public'
           AND indexname = 'api_client_credentials_one_live_idx'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(credential_index.contains("revoked_at IS NULL"));

    let token_index: String = sqlx::query_scalar(
        "SELECT indexdef FROM pg_indexes
         WHERE schemaname = 'public'
           AND indexname = 'external_access_tokens_credential_live_idx'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(token_index.contains("revoked_at IS NULL"));
}

#[tokio::test]
async fn database_rejects_short_credential_digest() {
    let pool = migrated_test_pool().await;
    let mut transaction = pool.begin().await.unwrap();
    let (client_id, staff_id) = insert_constraint_fixture(&mut transaction).await;
    let error = sqlx::query(
        "INSERT INTO api_client_credentials (
             api_client_id, secret_digest, digest_version, issued_at,
             issued_by_staff_user_id
         ) VALUES ($1, $2, 1, NOW(), $3)",
    )
    .bind(client_id)
    .bind(vec![0_u8; 31])
    .bind(staff_id)
    .execute(&mut *transaction)
    .await
    .unwrap_err();
    assert_constraint_error(error, "23514", "api_client_credentials_secret_digest_check");
    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn database_rejects_unsupported_credential_digest_version() {
    let pool = migrated_test_pool().await;
    let mut transaction = pool.begin().await.unwrap();
    let (client_id, staff_id) = insert_constraint_fixture(&mut transaction).await;
    let error = sqlx::query(
        "INSERT INTO api_client_credentials (
             api_client_id, secret_digest, digest_version, issued_at,
             issued_by_staff_user_id
         ) VALUES ($1, $2, 2, NOW(), $3)",
    )
    .bind(client_id)
    .bind(vec![0_u8; 32])
    .bind(staff_id)
    .execute(&mut *transaction)
    .await
    .unwrap_err();
    assert_constraint_error(
        error,
        "23514",
        "api_client_credentials_digest_version_check",
    );
    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn database_rejects_second_live_credential() {
    let pool = migrated_test_pool().await;
    let mut transaction = pool.begin().await.unwrap();
    let (client_id, staff_id) = insert_constraint_fixture(&mut transaction).await;
    insert_constraint_credential(&mut transaction, client_id, staff_id, [0x01; 32]).await;
    let error = sqlx::query(
        "INSERT INTO api_client_credentials (
             api_client_id, secret_digest, digest_version, issued_at,
             issued_by_staff_user_id
         ) VALUES ($1, $2, 1, NOW(), $3)",
    )
    .bind(client_id)
    .bind(vec![0x02_u8; 32])
    .bind(staff_id)
    .execute(&mut *transaction)
    .await
    .unwrap_err();
    assert_constraint_error(error, "23505", "api_client_credentials_one_live_idx");
    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn database_rejects_short_access_token_hash() {
    let pool = migrated_test_pool().await;
    let mut transaction = pool.begin().await.unwrap();
    let (client_id, staff_id) = insert_constraint_fixture(&mut transaction).await;
    let credential_id =
        insert_constraint_credential(&mut transaction, client_id, staff_id, [0x03; 32]).await;
    let error = sqlx::query(
        "INSERT INTO external_access_tokens (
             api_client_credential_id, token_hash, issued_at, expires_at
         ) VALUES ($1, $2, NOW(), NOW() + INTERVAL '15 minutes')",
    )
    .bind(credential_id)
    .bind(vec![0_u8; 31])
    .execute(&mut *transaction)
    .await
    .unwrap_err();
    assert_constraint_error(
        error,
        "23514",
        "external_access_tokens_token_hash_length_check",
    );
    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn database_rejects_non_increasing_access_token_expiry() {
    let pool = migrated_test_pool().await;
    let mut transaction = pool.begin().await.unwrap();
    let (client_id, staff_id) = insert_constraint_fixture(&mut transaction).await;
    let credential_id =
        insert_constraint_credential(&mut transaction, client_id, staff_id, [0x04; 32]).await;
    let error = sqlx::query(
        "INSERT INTO external_access_tokens (
             api_client_credential_id, token_hash, issued_at, expires_at
         ) VALUES ($1, $2, NOW(), NOW())",
    )
    .bind(credential_id)
    .bind(vec![0x05_u8; 32])
    .execute(&mut *transaction)
    .await
    .unwrap_err();
    assert_constraint_error(error, "23514", "external_access_tokens_expiry_check");
    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn database_rejects_credential_audit_without_credential_id() {
    let pool = migrated_test_pool().await;
    let mut transaction = pool.begin().await.unwrap();
    let (client_id, staff_id) = insert_constraint_fixture(&mut transaction).await;
    let error = sqlx::query(
        "INSERT INTO api_client_management_audit (
             api_client_id, actor_staff_user_id, action,
             before_state, after_state, created_at
         ) VALUES ($1, $2, 'CREDENTIAL_ISSUED', NULL, '{}'::jsonb, NOW())",
    )
    .bind(client_id)
    .bind(staff_id)
    .execute(&mut *transaction)
    .await
    .unwrap_err();
    assert_constraint_error(
        error,
        "23514",
        "api_client_management_audit_credential_context_check",
    );
    transaction.rollback().await.unwrap();
}

#[tokio::test]
async fn database_rejects_unknown_audit_action() {
    let pool = migrated_test_pool().await;
    let mut transaction = pool.begin().await.unwrap();
    let (client_id, staff_id) = insert_constraint_fixture(&mut transaction).await;
    let error = sqlx::query(
        "INSERT INTO api_client_management_audit (
             api_client_id, actor_staff_user_id, action,
             before_state, after_state, created_at
         ) VALUES ($1, $2, 'NOT_A_REAL_ACTION', NULL, '{}'::jsonb, NOW())",
    )
    .bind(client_id)
    .bind(staff_id)
    .execute(&mut *transaction)
    .await
    .unwrap_err();
    assert_constraint_error(error, "23514", "api_client_management_audit_action_check");
    transaction.rollback().await.unwrap();
}
