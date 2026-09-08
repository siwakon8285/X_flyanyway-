mod common;

use common::{test_database_url, validate_test_database_url};

#[test]
fn resolves_an_explicit_safe_test_database_url() {
    const SAFE_URL: &str =
        "postgresql://test_user:test_password@127.0.0.1:5434/x_fly_concurrency_test";
    let previous = std::env::var_os("TEST_DATABASE_URL");
    std::env::set_var("TEST_DATABASE_URL", SAFE_URL);

    assert_eq!(test_database_url(), SAFE_URL);

    if let Some(previous) = previous {
        std::env::set_var("TEST_DATABASE_URL", previous);
    } else {
        std::env::remove_var("TEST_DATABASE_URL");
    }
}

#[test]
fn accepts_the_dedicated_local_test_database() {
    assert!(validate_test_database_url(
        "postgresql://test_user:test_password@127.0.0.1:5434/x_fly_concurrency_test"
    )
    .is_ok());
}

#[test]
fn rejects_the_known_dev_port() {
    assert!(validate_test_database_url(
        "postgresql://test_user:test_password@127.0.0.1:5433/x_fly_concurrency_test"
    )
    .is_err());
}

#[test]
fn rejects_the_dev_database_name() {
    assert!(validate_test_database_url(
        "postgresql://test_user:test_password@127.0.0.1:5434/x_fly"
    )
    .is_err());
}

#[test]
fn rejects_a_missing_database_name() {
    assert!(
        validate_test_database_url("postgresql://test_user:test_password@127.0.0.1:5434").is_err()
    );
}

#[test]
fn rejects_a_clearly_non_test_database_name() {
    assert!(validate_test_database_url(
        "postgresql://test_user:test_password@database.internal:5432/x_fly_production"
    )
    .is_err());
}
