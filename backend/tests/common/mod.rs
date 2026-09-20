use std::{env, fmt::Display, future::Future, path::Path, str::FromStr};

use chrono::{Duration as ChronoDuration, NaiveDate};
use sqlx::{postgres::PgConnectOptions, Connection, PgConnection};

#[allow(dead_code)]
const FIXTURE_NAMESPACE_LOCK: i64 = 0x5846_4c59_5445_5354;
#[allow(dead_code)]
const AUTH_FIXTURE_LOCK: i64 = 0x5846_4c59_4155_5448;

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
pub struct TestFixtureLock {
    connection: PgConnection,
}

#[allow(dead_code)]
pub async fn run_fixture_body_with_cleanup<F, Fut, C, CF, E>(body: F, cleanup: C)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
    C: FnOnce() -> CF + Send + 'static,
    CF: Future<Output = Result<(), E>> + Send + 'static,
    E: Display + Send + 'static,
{
    let body_result = tokio::spawn(async move { body().await }).await;
    let cleanup_result = tokio::spawn(async move { cleanup().await }).await;
    finish_fixture_body(body_result, cleanup_result);
}

#[allow(dead_code)]
pub fn finish_fixture_body<E: Display>(
    body_result: Result<(), tokio::task::JoinError>,
    cleanup_result: Result<Result<(), E>, tokio::task::JoinError>,
) {
    match body_result {
        Ok(()) => match cleanup_result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => panic!("fixture cleanup failed: {error}"),
            Err(join_error) => std::panic::resume_unwind(join_error.into_panic()),
        },
        Err(join_error) => match cleanup_result {
            Ok(Ok(())) => std::panic::resume_unwind(join_error.into_panic()),
            Ok(Err(error)) => {
                eprintln!("fixture cleanup failed after primary test failure: {error}");
                std::panic::resume_unwind(join_error.into_panic());
            }
            Err(cleanup_join_error) => {
                eprintln!(
                    "fixture cleanup panicked after primary test failure: {cleanup_join_error}"
                );
                std::panic::resume_unwind(join_error.into_panic());
            }
        },
    }
}

#[allow(dead_code)]
pub async fn acquire_test_fixture_lock() -> TestFixtureLock {
    acquire_test_fixture_lock_named("x-fly-test-fixture").await
}

#[allow(dead_code)]
pub async fn acquire_test_fixture_lock_named(application_name: &str) -> TestFixtureLock {
    assert!(
        application_name.len() <= 63,
        "TEST fixture lock application name exceeds PostgreSQL's 63-byte limit"
    );
    let options = PgConnectOptions::from_str(&test_database_url())
        .expect("valid TEST fixture coordination connection options")
        .application_name(application_name);
    let mut connection = PgConnection::connect_with(&options)
        .await
        .expect("connect to TEST fixture coordination database");
    sqlx::query("BEGIN")
        .execute(&mut connection)
        .await
        .expect("begin TEST fixture coordination transaction");
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(AUTH_FIXTURE_LOCK)
        .execute(&mut connection)
        .await
        .expect("acquire TEST fixture coordination lock");
    TestFixtureLock { connection }
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

    let mut candidate = earliest;
    while candidate <= latest {
        let claimed: Option<NaiveDate> = sqlx::query_scalar(
            "INSERT INTO flight_instances (flight_service_id, departure_date)
             SELECT id, $2 FROM flight_services WHERE public_id = $1
             ON CONFLICT (flight_service_id, departure_date) DO NOTHING
             RETURNING departure_date",
        )
        .bind(flight_public_id)
        .bind(candidate)
        .fetch_optional(&mut *transaction)
        .await
        .expect("claim TEST flight instance namespace");

        if let Some(claimed) = claimed {
            transaction
                .commit()
                .await
                .expect("commit TEST fixture namespace allocation");
            return Ok(claimed);
        }

        if candidate == latest {
            break;
        }
        candidate += ChronoDuration::days(1);
    }

    Err(format!(
        "TEST flight instance namespace exhausted: flight={flight_public_id} earliest={earliest} latest={latest}"
    ))
}
