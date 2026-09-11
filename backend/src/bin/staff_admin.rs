use std::{env, sync::Arc, time::Duration};

use sqlx::postgres::PgPoolOptions;
use x_fly_api::{
    application::staff_auth::{StaffAdminCommand, StaffAuthService},
    infrastructure::{
        database::{verify_database_ready, SqlxStaffAuthRepository},
        password::Argon2PasswordService,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let command = StaffAdminCommand::parse(env::args().skip(1)).map_err(|_| usage())?;
    let password = rpassword::prompt_password("Staff password: ")?;
    let confirmation = rpassword::prompt_password("Confirm staff password: ")?;
    if password != confirmation {
        return Err("password confirmation does not match".into());
    }

    let database_url = env::var("MIGRATION_DATABASE_URL")
        .map_err(|_| "MIGRATION_DATABASE_URL must be configured")?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;
    verify_database_ready(&pool).await?;
    let service = StaffAuthService::new(
        Arc::new(SqlxStaffAuthRepository::new(pool)),
        Argon2PasswordService::default(),
        Duration::from_secs(60 * 60),
    )?;
    service
        .provision(command.mode(), command.email(), &password, command.roles())
        .await?;
    println!("Staff account provisioned for {}.", command.email());
    Ok(())
}

fn usage() -> &'static str {
    "usage: staff_admin <bootstrap|create> --email <address> --role <CANONICAL_ROLE> [--role <CANONICAL_ROLE> ...]"
}
