use std::{collections::BTreeSet, env, net::SocketAddr, sync::Mutex, time::Duration};

use uuid::Uuid;
use x_fly_api::{
    application::external_auth::ACCESS_TOKEN_TTL,
    config::AppConfig,
    domain::{
        api_client::ApiClientScope,
        external_api::{
            CredentialAdministrationError, CredentialFormatError, CredentialRevocationReason,
            ExternalApiCredentialPepper, ExternalAuthenticationError, ExternalPrincipal,
            ExternalTokenExchangeError, PlaintextAccessToken, PlaintextClientSecret,
        },
    },
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn valid_secret_text() -> String {
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_owned()
}

fn valid_token_text() -> String {
    format!("xfa_v1_{}", "0123456789abcdef".repeat(4))
}

fn set_required_config_environment() {
    env::set_var(
        "DATABASE_URL",
        "postgresql://runtime:runtime@127.0.0.1:5434/x_fly_concurrency_test",
    );
    env::set_var(
        "TICKET_QR_SIGNING_SECRET",
        "ticket-signing-secret-sentinel-with-required-length",
    );
    env::set_var(
        "MANAGE_BOOKING_SIGNING_SECRET",
        "manage-booking-signing-secret-sentinel-with-required-length",
    );
}

fn with_config_environment<T>(operation: impl FnOnce() -> T) -> T {
    let _guard = ENV_LOCK.lock().expect("configuration environment lock");
    let names = [
        "DATABASE_URL",
        "BACKEND_BIND_ADDRESS",
        "SEAT_HOLD_TTL_SECONDS",
        "APP_ENV",
        "FRONTEND_ORIGIN",
        "STRIPE_SECRET_KEY",
        "STRIPE_WEBHOOK_SECRET",
        "TICKET_QR_SIGNING_SECRET",
        "MANAGE_BOOKING_SIGNING_SECRET",
        "EXTERNAL_API_CREDENTIAL_PEPPER_V1",
    ];
    let previous: Vec<_> = names
        .iter()
        .map(|name| (*name, env::var_os(name)))
        .collect();
    for name in names {
        env::remove_var(name);
    }
    set_required_config_environment();
    let result = operation();
    for (name, value) in previous {
        match value {
            Some(value) => env::set_var(name, value),
            None => env::remove_var(name),
        }
    }
    result
}

#[test]
fn client_secret_parser_enforces_32_byte_lowercase_hex() {
    let encoded = valid_secret_text();
    let parsed = PlaintextClientSecret::parse_hex(&encoded).expect("valid secret representation");
    assert_eq!(parsed.as_bytes().len(), 32);
    assert!(PlaintextClientSecret::parse_hex(&encoded.to_uppercase()).is_err());
    assert!(matches!(
        PlaintextClientSecret::parse_hex(&encoded[..encoded.len() - 1]),
        Err(CredentialFormatError::InvalidLength)
    ));
    let mut whitespace_bytes = encoded.into_bytes();
    whitespace_bytes[0] = b' ';
    let whitespace = String::from_utf8(whitespace_bytes).expect("ASCII test input");
    assert!(matches!(
        PlaintextClientSecret::parse_hex(&whitespace),
        Err(CredentialFormatError::InvalidEncoding)
    ));
}

#[test]
fn access_token_parser_accepts_only_xfa_v1_hex() {
    let valid = valid_token_text();
    let parsed = PlaintextAccessToken::parse_bearer_text(&valid).expect("valid token text");
    assert_eq!(parsed.as_bytes().len(), 32);
    assert!(PlaintextAccessToken::parse_bearer_text("Bearer ".to_owned().as_str()).is_err());
    assert!(PlaintextAccessToken::parse_bearer_text(
        "xfa_v2_0000000000000000000000000000000000000000000000000000000000000000"
    )
    .is_err());
    assert!(PlaintextAccessToken::parse_bearer_text(&valid.to_uppercase()).is_err());
    assert!(PlaintextAccessToken::parse_bearer_text(&valid[..valid.len() - 1]).is_err());
}

#[test]
fn token_ttl_is_900_seconds() {
    assert_eq!(ACCESS_TOKEN_TTL, Duration::from_secs(900));
}

#[test]
fn credential_revocation_reason_matches_locked_receiver_signature() {
    let as_str: fn(&CredentialRevocationReason) -> &'static str =
        CredentialRevocationReason::as_str;
    assert_eq!(
        as_str(&CredentialRevocationReason::AdminRequest),
        "ADMIN_REQUEST"
    );
}

#[test]
fn external_principal_contains_current_scopes_without_serializing_internal_uuid() {
    let internal_id = Uuid::new_v4();
    let scopes = BTreeSet::from([ApiClientScope::FlightsRead]);
    let principal =
        ExternalPrincipal::new(internal_id, "XFCABCDEFGHJKLMNP2".to_owned(), scopes.clone());

    assert_eq!(principal.api_client_id(), internal_id);
    assert_eq!(principal.client_id(), "XFCABCDEFGHJKLMNP2");
    assert_eq!(principal.scopes(), &scopes);
}

#[test]
fn config_rejects_missing_or_malformed_external_pepper() {
    with_config_environment(|| {
        env::remove_var("EXTERNAL_API_CREDENTIAL_PEPPER_V1");
        assert!(AppConfig::from_env().is_err());

        env::set_var("EXTERNAL_API_CREDENTIAL_PEPPER_V1", "not-a-pepper");
        assert!(AppConfig::from_env().is_err());

        env::set_var(
            "EXTERNAL_API_CREDENTIAL_PEPPER_V1",
            "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
        );
        let config = AppConfig::from_env().expect("valid external pepper configuration");
        assert_eq!(config.external_api_credential_pepper.as_bytes().len(), 32);
    });
}

#[test]
fn sentinel_secret_material_never_appears_in_formatted_config_crypto_or_auth_types() {
    let pepper_bytes = *b"task1-pepper-sentinel-32-bytes!!";
    let pepper_sentinel = std::str::from_utf8(&pepper_bytes)
        .expect("test-only pepper sentinel is ASCII")
        .to_owned();
    let pepper_hex = hex::encode(pepper_bytes);
    let secret_sentinel = "client-secret-sentinel";
    let token_sentinel = "access-token-sentinel";
    let digest_sentinel = "credential-digest-sentinel";
    let hash_sentinel = "access-token-hash-sentinel";
    let config = AppConfig {
        database_url: format!("postgresql://{digest_sentinel}"),
        bind_address: "127.0.0.1:8080"
            .parse::<SocketAddr>()
            .expect("valid address"),
        frontend_origin: "https://example.test".to_owned(),
        seat_hold_ttl: Duration::from_secs(600),
        secure_cookies: true,
        stripe_secret_key: Some(secret_sentinel.to_owned()),
        stripe_webhook_secret: Some(token_sentinel.to_owned()),
        ticket_qr_signing_secret: digest_sentinel.to_owned(),
        manage_booking_signing_secret: hash_sentinel.to_owned(),
        external_api_credential_pepper: ExternalApiCredentialPepper::parse_hex(&pepper_hex)
            .expect("sentinel pepper representation has 32 bytes"),
    };

    let formatted = format!("{config:?}");
    for sentinel in [
        pepper_sentinel.as_str(),
        pepper_hex.as_str(),
        secret_sentinel,
        token_sentinel,
        digest_sentinel,
        hash_sentinel,
    ] {
        assert!(
            !formatted.contains(sentinel),
            "formatted config leaked secret material"
        );
    }
    for error in [
        format!("{:?}", CredentialAdministrationError::Infrastructure),
        format!("{:?}", ExternalAuthenticationError::Unavailable),
        format!("{:?}", ExternalTokenExchangeError::Unavailable),
        format!("{:?}", CredentialRevocationReason::AdminRequest),
    ] {
        for sentinel in [
            pepper_sentinel.as_str(),
            secret_sentinel,
            token_sentinel,
            digest_sentinel,
            hash_sentinel,
        ] {
            assert!(
                !error.contains(sentinel),
                "formatted auth value leaked secret material"
            );
        }
    }
}
