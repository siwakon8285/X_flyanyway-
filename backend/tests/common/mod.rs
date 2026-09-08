use std::{env, path::Path, str::FromStr};

use sqlx::postgres::PgConnectOptions;

const CONFIGURATION_HELP: &str =
    "Configure TEST_DATABASE_URL for a dedicated PostgreSQL database whose name ends in _test; X-Fly tests never use DATABASE_URL or the DEV database";

pub fn validate_test_database_url(database_url: &str) -> Result<(), &'static str> {
    let options = PgConnectOptions::from_str(database_url)
        .map_err(|_| "TEST_DATABASE_URL is not a valid PostgreSQL URL")?;

    if options.get_port() == 5433 {
        return Err("port 5433 is reserved for the X-Fly DEV database");
    }

    let database_name = options
        .get_database()
        .filter(|name| !name.is_empty())
        .ok_or("TEST_DATABASE_URL must include a database name")?;
    if database_name == "x_fly" || !database_name.ends_with("_test") {
        return Err("the database name must end in _test and must not be x_fly");
    }

    Ok(())
}

pub fn test_database_url() -> String {
    let database_url = match env::var("TEST_DATABASE_URL") {
        Ok(database_url) => database_url,
        Err(env::VarError::NotPresent) => {
            let environment_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
            let _ = dotenvy::from_path(environment_path);
            env::var("TEST_DATABASE_URL").unwrap_or_else(|_| panic!("{CONFIGURATION_HELP}"))
        }
        Err(env::VarError::NotUnicode(_)) => panic!("{CONFIGURATION_HELP}"),
    };

    if let Err(reason) = validate_test_database_url(&database_url) {
        panic!("Refusing unsafe TEST_DATABASE_URL: {reason}. {CONFIGURATION_HELP}");
    }

    database_url
}
