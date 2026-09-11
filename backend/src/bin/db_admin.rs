use std::env;

use sqlx::postgres::PgPoolOptions;
use x_fly_api::infrastructure::database::{migrate_database, seed_demo_database, DatabaseCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let command = env::args()
        .nth(1)
        .as_deref()
        .and_then(DatabaseCommand::parse)
        .ok_or_else(usage)?;
    if env::args().nth(2).is_some() {
        return Err(usage().into());
    }
    let database_url = env::var("MIGRATION_DATABASE_URL")
        .map_err(|_| "MIGRATION_DATABASE_URL must be configured")?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;
    match command {
        DatabaseCommand::Migrate => migrate_database(&pool).await?,
        DatabaseCommand::SeedDemo => {
            let confirmed_database = env::var("DEMO_SEED_DATABASE")
                .map_err(|_| "DEMO_SEED_DATABASE must confirm the target database")?;
            seed_demo_database(&pool, &confirmed_database).await?
        }
    }
    Ok(())
}

fn usage() -> &'static str {
    "usage: db_admin <migrate|seed-demo>"
}
