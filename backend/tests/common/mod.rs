use std::{env, path::Path, str::FromStr};

use chrono::NaiveDate;
use sqlx::{postgres::PgConnectOptions, Connection, PgConnection};

#[allow(dead_code)]
const FIXTURE_NAMESPACE_LOCK: i64 = 0x5846_4c59_5445_5354;

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

#[allow(dead_code)]
pub fn validate_test_database_pair(setup_url: &str, runtime_url: &str) -> Result<(), &'static str> {
    validate_test_database_url(setup_url)?;
    validate_test_database_url(runtime_url)?;
    let setup = PgConnectOptions::from_str(setup_url)
        .map_err(|_| "TEST_DATABASE_URL is not a valid PostgreSQL URL")?;
    let runtime = PgConnectOptions::from_str(runtime_url)
        .map_err(|_| "TEST_RUNTIME_DATABASE_URL is not a valid PostgreSQL URL")?;
    if setup.get_host() != runtime.get_host()
        || setup.get_port() != runtime.get_port()
        || setup.get_database() != runtime.get_database()
    {
        return Err("TEST setup and runtime credentials must target the same database");
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

#[allow(dead_code)]
pub fn test_runtime_database_url() -> String {
    let setup_url = test_database_url();
    let runtime_url = env::var("TEST_RUNTIME_DATABASE_URL").unwrap_or_else(|_| {
        panic!(
            "Configure TEST_RUNTIME_DATABASE_URL for the restricted runtime role; it must target the same TEST database as TEST_DATABASE_URL"
        )
    });
    if let Err(reason) = validate_test_database_pair(&setup_url, &runtime_url) {
        panic!("Refusing unsafe TEST runtime configuration: {reason}");
    }
    runtime_url
}

#[allow(dead_code)]
pub async fn allocate_test_departure_date(
    flight_public_id: &str,
    earliest: NaiveDate,
    latest: NaiveDate,
) -> Result<NaiveDate, String> {
    let database_url = test_database_url();
    let mut connection = PgConnection::connect(&database_url)
        .await
        .expect("connect to TEST fixture namespace allocator");
    let mut transaction = connection
        .begin()
        .await
        .expect("begin TEST fixture namespace allocation");

    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(FIXTURE_NAMESPACE_LOCK)
        .execute(&mut *transaction)
        .await
        .expect("lock TEST fixture namespace allocation");

    let departure_date: Option<NaiveDate> = sqlx::query_scalar(
        "SELECT candidate::date
         FROM generate_series($2::date, $3::date, INTERVAL '1 day') AS candidate
         WHERE NOT EXISTS (
             SELECT 1
             FROM flight_instances AS instance
             JOIN flight_services AS service ON service.id = instance.flight_service_id
             WHERE service.public_id = $1
               AND instance.departure_date = candidate::date
         )
         ORDER BY candidate
         LIMIT 1",
    )
    .bind(flight_public_id)
    .bind(earliest)
    .bind(latest)
    .fetch_optional(&mut *transaction)
    .await
    .expect("inspect available TEST flight instance namespaces");
    let Some(departure_date) = departure_date else {
        return Err(format!(
            "TEST flight instance namespace exhausted: flight={flight_public_id} earliest={earliest} latest={latest}"
        ));
    };

    let claimed: NaiveDate = sqlx::query_scalar(
        "INSERT INTO flight_instances (flight_service_id, departure_date)
         SELECT id, $2 FROM flight_services WHERE public_id = $1
         RETURNING departure_date",
    )
    .bind(flight_public_id)
    .bind(departure_date)
    .fetch_one(&mut *transaction)
    .await
    .expect("claim TEST flight instance namespace");

    transaction
        .commit()
        .await
        .expect("commit TEST fixture namespace allocation");
    Ok(claimed)
}
