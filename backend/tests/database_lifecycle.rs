mod common;

use sqlx::{PgPool, Row};
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
