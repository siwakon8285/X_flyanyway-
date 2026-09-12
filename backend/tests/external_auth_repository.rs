mod common;

use std::{sync::Arc, time::Duration};

use chrono::{Duration as ChronoDuration, Utc};
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

use x_fly_api::{
    application::external_auth::{ExternalAuthRepository, ExternalCredentialCrypto},
    domain::{
        api_client::ApiClientScope,
        external_api::{
            AccessTokenHash, CredentialAdministrationError, CredentialDigest,
            CredentialRevocationReason, ExternalApiCredentialPepper, ExternalTokenExchangeError,
            PlaintextAccessToken, PlaintextClientSecret,
        },
    },
    infrastructure::database::{
        migrate_database, verify_database_ready, SqlxExternalAuthRepository,
    },
    infrastructure::external_auth_crypto::HmacExternalCredentialCrypto,
};

fn test_pepper() -> ExternalApiCredentialPepper {
    ExternalApiCredentialPepper::parse_hex(
        "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
    )
    .expect("test pepper is valid")
}

fn secret_text(byte: u8) -> String {
    hex::encode([byte; 32])
}

#[test]
fn generated_client_secret_has_canonical_32_byte_shape() {
    let crypto = HmacExternalCredentialCrypto::from_pepper(test_pepper());
    let secret = crypto.generate_client_secret();
    assert_eq!(secret.as_bytes().len(), 32);
    let encoded = hex::encode(secret.as_bytes());
    assert_eq!(encoded.len(), 64);
    assert!(
        PlaintextClientSecret::parse_hex(&encoded)
            .unwrap()
            .as_bytes()
            == secret.as_bytes()
    );
}

#[test]
fn generated_client_secrets_are_independent() {
    let crypto = HmacExternalCredentialCrypto::from_pepper(test_pepper());
    let first = crypto.generate_client_secret();
    let second = crypto.generate_client_secret();

    assert_eq!(first.as_bytes().len(), 32);
    assert_eq!(second.as_bytes().len(), 32);
    assert!(
        first.as_bytes() != second.as_bytes(),
        "independent client-secret generations unexpectedly matched"
    );

    let first_encoded = hex::encode(first.as_bytes());
    let second_encoded = hex::encode(second.as_bytes());
    assert!(
        PlaintextClientSecret::parse_hex(&first_encoded)
            .unwrap()
            .as_bytes()
            == first.as_bytes()
    );
    assert!(
        PlaintextClientSecret::parse_hex(&second_encoded)
            .unwrap()
            .as_bytes()
            == second.as_bytes()
    );
}

#[test]
fn credential_digest_is_deterministic_and_bound_to_client_and_secret() {
    let crypto = HmacExternalCredentialCrypto::from_pepper(test_pepper());
    let secret = PlaintextClientSecret::parse_hex(&secret_text(0x11)).unwrap();
    let same = crypto.credential_digest("XFCABCDEFGHJKLMNP2", &secret);
    let repeated = crypto.credential_digest("XFCABCDEFGHJKLMNP2", &secret);
    let different_secret = PlaintextClientSecret::parse_hex(&secret_text(0x22)).unwrap();
    let different_client = crypto.credential_digest("XFCABCDEFGHJKLMNP3", &secret);

    assert!(same.as_bytes() == repeated.as_bytes());
    assert!(
        same.as_bytes()
            != crypto
                .credential_digest("XFCABCDEFGHJKLMNP2", &different_secret)
                .as_bytes()
    );
    assert!(same.as_bytes() != different_client.as_bytes());
    assert!(crypto.digest_matches(&same, &repeated));
    assert!(!crypto.digest_matches(&same, &different_client));
}

#[test]
fn dummy_digest_is_a_valid_distinct_hmac_path() {
    let crypto = HmacExternalCredentialCrypto::from_pepper(test_pepper());
    let dummy = crypto.dummy_credential_digest("XFCABCDEFGHJKLMNP2");
    let real = crypto.credential_digest(
        "XFCABCDEFGHJKLMNP2",
        &PlaintextClientSecret::parse_hex(&secret_text(0x11)).unwrap(),
    );
    assert_eq!(dummy.as_bytes().len(), 32);
    assert!(dummy.as_bytes() != real.as_bytes());
}

#[test]
fn generated_access_token_and_hash_are_canonical_and_deterministic() {
    let crypto = HmacExternalCredentialCrypto::from_pepper(test_pepper());
    let token = crypto.generate_access_token();
    let second_token = crypto.generate_access_token();
    let encoded = format!("xfa_v1_{}", hex::encode(token.as_bytes()));
    let parsed = PlaintextAccessToken::parse_bearer_text(&encoded).unwrap();
    assert!(parsed.as_bytes() == token.as_bytes());
    assert_eq!(token.as_bytes().len(), 32);

    let hash = crypto.access_token_hash(&token);
    let repeated = crypto.access_token_hash(&parsed);
    assert!(hash.as_bytes() == repeated.as_bytes());
    assert_eq!(hash.as_bytes().len(), 32);
    assert!(
        token.as_bytes() != second_token.as_bytes(),
        "independent access-token generations unexpectedly matched"
    );
    assert!(
        hash.as_bytes() != crypto.access_token_hash(&second_token).as_bytes(),
        "different access tokens unexpectedly produced the same hash"
    );
}

#[test]
fn token_ttl_contract_remains_fifteen_minutes() {
    assert_eq!(
        x_fly_api::application::external_auth::ACCESS_TOKEN_TTL,
        Duration::from_secs(900)
    );
}

#[allow(dead_code)]
fn _value_types_remain_fixed_length(digest: CredentialDigest, hash: AccessTokenHash) {
    assert_eq!(digest.as_bytes().len(), 32);
    assert_eq!(hash.as_bytes().len(), 32);
}

struct Fixture {
    client_pk: Uuid,
    client_id: String,
    actor_id: Uuid,
}

async fn pools() -> (PgPool, PgPool) {
    let setup = PgPoolOptions::new()
        .max_connections(5)
        .connect(&common::test_database_url())
        .await
        .expect("connect to TEST as migrator");
    migrate_database(&setup)
        .await
        .expect("TEST migration chain is current");
    let runtime = PgPoolOptions::new()
        .max_connections(5)
        .connect(&common::test_runtime_database_url())
        .await
        .expect("connect to TEST as runtime");
    verify_database_ready(&runtime)
        .await
        .expect("runtime sees ready TEST schema");
    (setup, runtime)
}

fn unique_client_id() -> String {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let suffix: String = Uuid::new_v4()
        .as_bytes()
        .iter()
        .map(|byte| ALPHABET[(*byte & 31) as usize] as char)
        .collect();
    format!("XFC{suffix}")
}

async fn fixture(setup: &PgPool, scopes: &[ApiClientScope], status: &str) -> Fixture {
    let actor_id: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash) VALUES ($1,'external-auth-test') RETURNING id",
    )
    .bind(format!("external-auth-{}@test.invalid", Uuid::new_v4()))
    .fetch_one(setup)
    .await
    .expect("fixture actor");
    let client_id = unique_client_id();
    let client_pk: Uuid = sqlx::query_scalar(
        "INSERT INTO api_clients (
             client_id,display_name,description,status,
             created_by_staff_user_id,updated_by_staff_user_id
         ) VALUES ($1,'External auth test',NULL,$2,$3,$3) RETURNING id",
    )
    .bind(&client_id)
    .bind(status)
    .bind(actor_id)
    .fetch_one(setup)
    .await
    .expect("fixture client");
    for scope in scopes {
        sqlx::query(
            "INSERT INTO api_client_allowed_scopes (api_client_id,scope_code,assigned_by_staff_user_id)
             VALUES ($1,$2,$3)",
        )
        .bind(client_pk)
        .bind(scope.as_str())
        .bind(actor_id)
        .execute(setup)
        .await
        .expect("fixture scope");
    }
    Fixture {
        client_pk,
        client_id,
        actor_id,
    }
}

async fn cleanup(setup: &PgPool, fixture: &Fixture) {
    sqlx::query("DELETE FROM api_client_management_audit WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(setup)
        .await
        .expect("audit cleanup");
    sqlx::query(
        "DELETE FROM external_access_tokens WHERE api_client_credential_id IN
             (SELECT id FROM api_client_credentials WHERE api_client_id=$1)",
    )
    .bind(fixture.client_pk)
    .execute(setup)
    .await
    .expect("token cleanup");
    sqlx::query("DELETE FROM api_client_credentials WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(setup)
        .await
        .expect("credential cleanup");
    sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(setup)
        .await
        .expect("scope cleanup");
    sqlx::query("DELETE FROM api_clients WHERE id=$1")
        .bind(fixture.client_pk)
        .execute(setup)
        .await
        .expect("client cleanup");
    sqlx::query("DELETE FROM staff_users WHERE id=$1")
        .bind(fixture.actor_id)
        .execute(setup)
        .await
        .expect("actor cleanup");
}

fn crypto() -> Arc<HmacExternalCredentialCrypto> {
    Arc::new(HmacExternalCredentialCrypto::from_pepper(test_pepper()))
}

#[tokio::test]
async fn persists_only_credential_hmac_digest() {
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let secret = PlaintextClientSecret::parse_hex(&secret_text(0x31)).unwrap();
    let digest = crypto().credential_digest(&fixture.client_id, &secret);
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    let metadata = repository
        .issue_credential(&fixture.client_id, fixture.actor_id, 1, &digest, Utc::now())
        .await
        .expect("runtime may issue credential");
    let stored: Vec<u8> =
        sqlx::query_scalar("SELECT secret_digest FROM api_client_credentials WHERE id=$1")
            .bind(metadata.credential_id)
            .fetch_one(&setup)
            .await
            .expect("read persisted verifier");
    assert!(stored.as_slice() == digest.as_bytes());
    assert!(stored.as_slice() != secret.as_bytes());
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn never_persists_plaintext_client_secret() {
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let secret = PlaintextClientSecret::parse_hex(&secret_text(0x41)).unwrap();
    let digest = crypto().credential_digest(&fixture.client_id, &secret);
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    let metadata = repository
        .issue_credential(&fixture.client_id, fixture.actor_id, 1, &digest, Utc::now())
        .await
        .unwrap();
    let plaintext_match: bool =
        sqlx::query_scalar("SELECT secret_digest = $2 FROM api_client_credentials WHERE id=$1")
            .bind(metadata.credential_id)
            .bind(secret.as_bytes().to_vec())
            .fetch_one(&setup)
            .await
            .unwrap();
    assert!(!plaintext_match);
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn persists_only_access_token_sha256() {
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let crypto = crypto();
    let secret = PlaintextClientSecret::parse_hex(&secret_text(0x51)).unwrap();
    let digest = crypto.credential_digest(&fixture.client_id, &secret);
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    repository
        .issue_credential(&fixture.client_id, fixture.actor_id, 1, &digest, Utc::now())
        .await
        .unwrap();
    let token = crypto.generate_access_token();
    let token_hash = crypto.access_token_hash(&token);
    let issued_at = Utc::now();
    repository
        .issue_access_token(
            &fixture.client_id,
            &digest,
            &token_hash,
            issued_at,
            issued_at + ChronoDuration::minutes(15),
        )
        .await
        .expect("runtime may persist token hash");
    let stored: Vec<u8> = sqlx::query_scalar(
        "SELECT token_hash FROM external_access_tokens token
         JOIN api_client_credentials credential ON credential.id=token.api_client_credential_id
         WHERE credential.api_client_id=$1",
    )
    .bind(fixture.client_pk)
    .fetch_one(&setup)
    .await
    .unwrap();
    assert!(stored.as_slice() == token_hash.as_bytes());
    assert!(stored.as_slice() != token.as_bytes());
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn plaintext_digest_cannot_authenticate() {
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let crypto = crypto();
    let correct = PlaintextClientSecret::parse_hex(&secret_text(0x61)).unwrap();
    let wrong = PlaintextClientSecret::parse_hex(&secret_text(0x62)).unwrap();
    let digest = crypto.credential_digest(&fixture.client_id, &correct);
    let wrong_digest = crypto.credential_digest(&fixture.client_id, &wrong);
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    repository
        .issue_credential(&fixture.client_id, fixture.actor_id, 1, &digest, Utc::now())
        .await
        .unwrap();
    let token = crypto.generate_access_token();
    let token_hash = crypto.access_token_hash(&token);
    let now = Utc::now();
    let result = repository
        .issue_access_token(
            &fixture.client_id,
            &wrong_digest,
            &token_hash,
            now,
            now + ChronoDuration::minutes(15),
        )
        .await;
    assert_eq!(result, Err(ExternalTokenExchangeError::InvalidCredential));
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM external_access_tokens token
         JOIN api_client_credentials credential ON credential.id=token.api_client_credential_id
         WHERE credential.api_client_id=$1",
    )
    .bind(fixture.client_pk)
    .fetch_one(&setup)
    .await
    .unwrap();
    assert_eq!(count, 0);
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn rejects_duplicate_live_credential() {
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let crypto = crypto();
    let first = PlaintextClientSecret::parse_hex(&secret_text(0x71)).unwrap();
    let first_digest = crypto.credential_digest(&fixture.client_id, &first);
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    repository
        .issue_credential(
            &fixture.client_id,
            fixture.actor_id,
            1,
            &first_digest,
            Utc::now(),
        )
        .await
        .unwrap();
    let second = PlaintextClientSecret::parse_hex(&secret_text(0x72)).unwrap();
    let second_digest = crypto.credential_digest(&fixture.client_id, &second);
    let result = repository
        .issue_credential(
            &fixture.client_id,
            fixture.actor_id,
            2,
            &second_digest,
            Utc::now(),
        )
        .await;
    assert!(matches!(
        result,
        Err(CredentialAdministrationError::LiveCredential)
    ));
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_client_credentials WHERE api_client_id=$1 AND revoked_at IS NULL",
    )
    .bind(fixture.client_pk)
    .fetch_one(&setup)
    .await
    .unwrap();
    assert_eq!(count, 1);
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn revokes_credential_and_live_tokens_atomically() {
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::AnalyticsRead], "ACTIVE").await;
    let crypto = crypto();
    let secret = PlaintextClientSecret::parse_hex(&secret_text(0x81)).unwrap();
    let digest = crypto.credential_digest(&fixture.client_id, &secret);
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    let issued = repository
        .issue_credential(&fixture.client_id, fixture.actor_id, 1, &digest, Utc::now())
        .await
        .unwrap();
    let token = crypto.generate_access_token();
    let token_hash = crypto.access_token_hash(&token);
    let now = Utc::now();
    repository
        .issue_access_token(
            &fixture.client_id,
            &digest,
            &token_hash,
            now,
            now + ChronoDuration::minutes(15),
        )
        .await
        .unwrap();
    let revoked_at = now + ChronoDuration::seconds(1);
    let revoked = repository
        .revoke_credential(
            &fixture.client_id,
            fixture.actor_id,
            2,
            CredentialRevocationReason::AdminRequest,
            revoked_at,
        )
        .await
        .unwrap();
    assert_eq!(revoked.credential_id, issued.credential_id);
    assert_eq!(revoked.revoked_at, Some(revoked_at));
    assert_eq!(
        revoked.revocation_reason,
        Some(CredentialRevocationReason::AdminRequest)
    );
    let credential_state: (Option<chrono::DateTime<Utc>>, Option<String>) = sqlx::query_as(
        "SELECT revoked_at,revocation_reason FROM api_client_credentials WHERE id=$1",
    )
    .bind(issued.credential_id)
    .fetch_one(&setup)
    .await
    .unwrap();
    assert_eq!(credential_state.0, Some(revoked_at));
    assert_eq!(credential_state.1.as_deref(), Some("ADMIN_REQUEST"));
    let token_revoked: Option<chrono::DateTime<Utc>> =
        sqlx::query_scalar("SELECT revoked_at FROM external_access_tokens WHERE token_hash=$1")
            .bind(token_hash.as_bytes().to_vec())
            .fetch_one(&setup)
            .await
            .unwrap();
    assert_eq!(token_revoked, Some(revoked_at));
    let audit_action: String = sqlx::query_scalar(
        "SELECT action FROM api_client_management_audit
         WHERE credential_id=$1 ORDER BY created_at DESC,id DESC LIMIT 1",
    )
    .bind(issued.credential_id)
    .fetch_one(&setup)
    .await
    .unwrap();
    assert_eq!(audit_action, "CREDENTIAL_REVOKED");
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn zero_scope_token_returns_empty_principal() {
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &[], "ACTIVE").await;
    let crypto = crypto();
    let secret = PlaintextClientSecret::parse_hex(&secret_text(0x91)).unwrap();
    let digest = crypto.credential_digest(&fixture.client_id, &secret);
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    repository
        .issue_credential(&fixture.client_id, fixture.actor_id, 1, &digest, Utc::now())
        .await
        .unwrap();
    let token = crypto.generate_access_token();
    let token_hash = crypto.access_token_hash(&token);
    let now = Utc::now();
    repository
        .issue_access_token(
            &fixture.client_id,
            &digest,
            &token_hash,
            now,
            now + ChronoDuration::minutes(15),
        )
        .await
        .unwrap();
    let principal = repository
        .authenticate_access_token(&token_hash, now + ChronoDuration::seconds(1))
        .await
        .unwrap();
    assert!(principal.scopes().is_empty());
    assert_eq!(principal.client_id(), fixture.client_id);
    cleanup(&setup, &fixture).await;
}

#[tokio::test]
async fn mutation_materializes_before_commit() {
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "SUSPENDED").await;
    let crypto = crypto();
    let secret = PlaintextClientSecret::parse_hex(&secret_text(0xa1)).unwrap();
    let digest = crypto.credential_digest(&fixture.client_id, &secret);
    let repository = SqlxExternalAuthRepository::new(runtime.clone());
    let issued_at = Utc::now();
    let metadata = repository
        .issue_credential(&fixture.client_id, fixture.actor_id, 1, &digest, issued_at)
        .await
        .unwrap();
    let persisted: (Uuid, chrono::DateTime<Utc>, Option<chrono::DateTime<Utc>>) =
        sqlx::query_as("SELECT id,issued_at,revoked_at FROM api_client_credentials WHERE id=$1")
            .bind(metadata.credential_id)
            .fetch_one(&setup)
            .await
            .unwrap();
    assert_eq!(persisted.0, metadata.credential_id);
    assert_eq!(persisted.1, metadata.issued_at);
    assert_eq!(persisted.2, metadata.revoked_at);
    cleanup(&setup, &fixture).await;
}
