use sqlx::{PgPool, Row};
use thiserror::Error;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatabaseCommand {
    Migrate,
    SeedDemo,
}

impl DatabaseCommand {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "migrate" => Some(Self::Migrate),
            "seed-demo" => Some(Self::SeedDemo),
            _ => None,
        }
    }
}

#[derive(Debug, Error)]
pub enum DatabaseLifecycleError {
    #[error("database migration failed")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("database schema is not ready")]
    NotReady,
    #[error("demo seed requires an empty demo inventory")]
    SeedTargetPopulated,
    #[error("demo seed database confirmation does not match the connected database")]
    SeedTargetMismatch,
    #[error("database lifecycle operation failed")]
    Database(#[from] sqlx::Error),
}

pub async fn migrate_database(pool: &PgPool) -> Result<(), DatabaseLifecycleError> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

pub async fn verify_database_ready(pool: &PgPool) -> Result<(), DatabaseLifecycleError> {
    let rows = sqlx::query("SELECT version, checksum, success FROM _sqlx_migrations")
        .fetch_all(pool)
        .await
        .map_err(|_| DatabaseLifecycleError::NotReady)?;
    if rows.len() != MIGRATOR.iter().count() {
        return Err(DatabaseLifecycleError::NotReady);
    }
    for migration in MIGRATOR.iter() {
        let Some(row) = rows
            .iter()
            .find(|row| row.get::<i64, _>("version") == migration.version)
        else {
            return Err(DatabaseLifecycleError::NotReady);
        };
        if !row.get::<bool, _>("success")
            || row.get::<Vec<u8>, _>("checksum").as_slice() != migration.checksum.as_ref()
        {
            return Err(DatabaseLifecycleError::NotReady);
        }
    }
    Ok(())
}

pub async fn seed_demo_database(
    pool: &PgPool,
    confirmed_database: &str,
) -> Result<(), DatabaseLifecycleError> {
    let mut transaction = pool.begin().await?;
    let current_database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&mut *transaction)
        .await?;
    if confirmed_database.is_empty() || current_database != confirmed_database {
        return Err(DatabaseLifecycleError::SeedTargetMismatch);
    }
    let populated: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM flight_services)
             OR EXISTS(SELECT 1 FROM flight_service_cabins)
             OR EXISTS(SELECT 1 FROM aircraft_seat_templates)
             OR EXISTS(SELECT 1 FROM flight_service_seat_templates)",
    )
    .fetch_one(&mut *transaction)
    .await?;
    if populated {
        return Err(DatabaseLifecycleError::SeedTargetPopulated);
    }
    sqlx::raw_sql(include_str!("../../../seeds/demo_flight_inventory.sql"))
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn prepare_test_database(pool: &PgPool) -> Result<(), DatabaseLifecycleError> {
    let database_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(pool)
        .await?;
    if database_name == "x_fly" || !database_name.ends_with("_test") {
        return Err(DatabaseLifecycleError::SeedTargetMismatch);
    }
    migrate_database(pool).await?;
    let populated: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM flight_services)")
        .fetch_one(pool)
        .await?;
    if !populated {
        seed_demo_database(pool, &database_name).await?;
    }
    Ok(())
}
