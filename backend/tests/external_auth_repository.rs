mod common;

use std::{future::Future, sync::Arc, time::Duration};

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

#[derive(Clone)]
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
    let mut transaction = setup.begin().await.expect("begin fixture transaction");
    let actor_id: Uuid = sqlx::query_scalar(
        "INSERT INTO staff_users (email,password_hash) VALUES ($1,'external-auth-test') RETURNING id",
    )
    .bind(format!("external-auth-{}@test.invalid", Uuid::new_v4()))
    .fetch_one(&mut *transaction)
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
    .fetch_one(&mut *transaction)
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
        .execute(&mut *transaction)
        .await
        .expect("fixture scope");
    }
    transaction
        .commit()
        .await
        .expect("commit fixture transaction");
    Fixture {
        client_pk,
        client_id,
        actor_id,
    }
}

async fn cleanup(setup: &PgPool, fixture: &Fixture) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM api_client_management_audit WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(setup)
        .await?;
    sqlx::query(
        "DELETE FROM external_access_tokens WHERE api_client_credential_id IN
             (SELECT id FROM api_client_credentials WHERE api_client_id=$1)",
    )
    .bind(fixture.client_pk)
    .execute(setup)
    .await?;
    sqlx::query("DELETE FROM api_client_credentials WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(setup)
        .await?;
    sqlx::query("DELETE FROM api_client_allowed_scopes WHERE api_client_id=$1")
        .bind(fixture.client_pk)
        .execute(setup)
        .await?;
    sqlx::query("DELETE FROM api_clients WHERE id=$1")
        .bind(fixture.client_pk)
        .execute(setup)
        .await?;
    sqlx::query("DELETE FROM staff_users WHERE id=$1")
        .bind(fixture.actor_id)
        .execute(setup)
        .await?;
    Ok(())
}

async fn cleanup_actor_client(
    setup: &PgPool,
    actor_id: Uuid,
    client_pk: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM api_clients WHERE id=$1")
        .bind(client_pk)
        .execute(setup)
        .await?;
    sqlx::query("DELETE FROM staff_users WHERE id=$1")
        .bind(actor_id)
        .execute(setup)
        .await?;
    Ok(())
}

async fn cleanup_fixtures(setup: &PgPool, fixtures: &[&Fixture]) -> Result<(), String> {
    let mut errors = Vec::new();
    for fixture in fixtures {
        if let Err(error) = cleanup(setup, fixture).await {
            errors.push(error.to_string());
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

async fn run_fixture_test<F, Fut>(scopes: Vec<ApiClientScope>, status: &'static str, body: F)
where
    F: FnOnce(PgPool, PgPool, Fixture) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let (setup, runtime) = pools().await;
    let fixture = fixture(&setup, &scopes, status).await;
    let cleanup_fixture = fixture.clone();
    let cleanup_setup = setup.clone();
    common::run_fixture_body_with_cleanup(
        move || body(setup, runtime, fixture),
        move || async move { cleanup(&cleanup_setup, &cleanup_fixture).await },
    )
    .await;
}

async fn wait_for_fixture_lock_wait(setup: &PgPool, application_name: &str) {
    let observation = async {
        loop {
            let waiting: i64 = sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM pg_stat_activity AS waiter
                 WHERE waiter.datname=current_database()
                   AND waiter.application_name=$1
                   AND EXISTS (
                       SELECT 1
                       FROM pg_locks AS blocked
                       WHERE blocked.pid=waiter.pid
                         AND blocked.locktype='advisory'
                         AND NOT blocked.granted
                   )",
            )
            .bind(application_name)
            .fetch_one(setup)
            .await
            .expect("inspect TEST fixture lock wait");
            if waiting == 1 {
                break;
            }
            tokio::task::yield_now().await;
        }
    };
    tokio::time::timeout(Duration::from_secs(10), observation)
        .await
        .expect("fixture operation did not reach the expected advisory-lock wait");
}

fn crypto() -> Arc<HmacExternalCredentialCrypto> {
    Arc::new(HmacExternalCredentialCrypto::from_pepper(test_pepper()))
}

#[tokio::test]
async fn persists_only_credential_hmac_digest() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        |setup, runtime, fixture| async move {
            let secret = PlaintextClientSecret::parse_hex(&secret_text(0x31)).unwrap();
            let digest = crypto().credential_digest(&fixture.client_id, &secret);
            let repository = SqlxExternalAuthRepository::new(runtime);
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
        },
    )
    .await;
}

#[tokio::test]
async fn never_persists_plaintext_client_secret() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        |setup, runtime, fixture| async move {
            let secret = PlaintextClientSecret::parse_hex(&secret_text(0x41)).unwrap();
            let digest = crypto().credential_digest(&fixture.client_id, &secret);
            let repository = SqlxExternalAuthRepository::new(runtime);
            let metadata = repository
                .issue_credential(&fixture.client_id, fixture.actor_id, 1, &digest, Utc::now())
                .await
                .unwrap();
            let plaintext_match: bool = sqlx::query_scalar(
                "SELECT secret_digest = $2 FROM api_client_credentials WHERE id=$1",
            )
            .bind(metadata.credential_id)
            .bind(secret.as_bytes().to_vec())
            .fetch_one(&setup)
            .await
            .unwrap();
            assert!(!plaintext_match);
        },
    )
    .await;
}

#[tokio::test]
async fn persists_only_access_token_sha256() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        |setup, runtime, fixture| async move {
            let crypto = crypto();
            let secret = PlaintextClientSecret::parse_hex(&secret_text(0x51)).unwrap();
            let digest = crypto.credential_digest(&fixture.client_id, &secret);
            let repository = SqlxExternalAuthRepository::new(runtime);
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
        },
    )
    .await;
}

#[tokio::test]
async fn plaintext_digest_cannot_authenticate() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "ACTIVE",
        |setup, runtime, fixture| async move {
            let crypto = crypto();
            let correct = PlaintextClientSecret::parse_hex(&secret_text(0x61)).unwrap();
            let wrong = PlaintextClientSecret::parse_hex(&secret_text(0x62)).unwrap();
            let digest = crypto.credential_digest(&fixture.client_id, &correct);
            let wrong_digest = crypto.credential_digest(&fixture.client_id, &wrong);
            let repository = SqlxExternalAuthRepository::new(runtime);
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
        },
    )
    .await;
}

#[tokio::test]
async fn rejects_duplicate_live_credential() {
    run_fixture_test(vec![ApiClientScope::FlightsRead], "ACTIVE", |setup, runtime, fixture| async move {
        let crypto = crypto();
        let first = PlaintextClientSecret::parse_hex(&secret_text(0x71)).unwrap();
        let first_digest = crypto.credential_digest(&fixture.client_id, &first);
        let repository = SqlxExternalAuthRepository::new(runtime);
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
    })
    .await;
}

#[tokio::test]
async fn revokes_credential_and_live_tokens_atomically() {
    run_fixture_test(
        vec![ApiClientScope::AnalyticsRead],
        "ACTIVE",
        |setup, runtime, fixture| async move {
            let crypto = crypto();
            let secret = PlaintextClientSecret::parse_hex(&secret_text(0x81)).unwrap();
            let digest = crypto.credential_digest(&fixture.client_id, &secret);
            let repository = SqlxExternalAuthRepository::new(runtime);
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
            let token_revoked: Option<chrono::DateTime<Utc>> = sqlx::query_scalar(
                "SELECT revoked_at FROM external_access_tokens WHERE token_hash=$1",
            )
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
        },
    )
    .await;
}

#[tokio::test]
async fn zero_scope_token_returns_empty_principal() {
    run_fixture_test(
        Vec::new(),
        "ACTIVE",
        |_setup, runtime, fixture| async move {
            let crypto = crypto();
            let secret = PlaintextClientSecret::parse_hex(&secret_text(0x91)).unwrap();
            let digest = crypto.credential_digest(&fixture.client_id, &secret);
            let repository = SqlxExternalAuthRepository::new(runtime);
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
        },
    )
    .await;
}

#[tokio::test]
async fn mutation_materializes_before_commit() {
    run_fixture_test(
        vec![ApiClientScope::FlightsRead],
        "SUSPENDED",
        |setup, runtime, fixture| async move {
            let crypto = crypto();
            let secret = PlaintextClientSecret::parse_hex(&secret_text(0xa1)).unwrap();
            let digest = crypto.credential_digest(&fixture.client_id, &secret);
            let repository = SqlxExternalAuthRepository::new(runtime);
            let issued_at = Utc::now();
            let metadata = repository
                .issue_credential(&fixture.client_id, fixture.actor_id, 1, &digest, issued_at)
                .await
                .unwrap();
            let persisted: (Uuid, chrono::DateTime<Utc>, Option<chrono::DateTime<Utc>>) =
                sqlx::query_as(
                    "SELECT id,issued_at,revoked_at FROM api_client_credentials WHERE id=$1",
                )
                .bind(metadata.credential_id)
                .fetch_one(&setup)
                .await
                .unwrap();
            assert_eq!(persisted.0, metadata.credential_id);
            assert_eq!(persisted.1, metadata.issued_at);
            assert_eq!(persisted.2, metadata.revoked_at);
        },
    )
    .await;
}

#[tokio::test]
async fn external_fixture_setup_survives_staff_cleanup_interleaving() {
    let (setup, _runtime) = pools().await;
    let external_application_name = format!("b25-fixture-ext-{}", Uuid::new_v4());
    let staff_application_name = format!("b25-fixture-staff-{}", Uuid::new_v4());
    let (fixture_ready_tx, fixture_ready_rx) = tokio::sync::oneshot::channel();
    let (commit_tx, commit_rx) = tokio::sync::oneshot::channel();
    let (committed_tx, committed_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let external_setup = setup.clone();
    let external_application_name_for_task = external_application_name.clone();
    let external = tokio::spawn(async move {
        let _fixture_lock =
            common::acquire_test_fixture_lock_named(&external_application_name_for_task).await;
        let mut transaction = external_setup
            .begin()
            .await
            .expect("begin atomic external fixture transaction");
        let actor_id: Uuid = sqlx::query_scalar(
            "INSERT INTO staff_users (email,password_hash) VALUES ($1,'fixture-isolation-green') RETURNING id",
        )
        .bind(format!(
            "fixture-isolation-green-{}@test.invalid",
            Uuid::new_v4()
        ))
        .fetch_one(&mut *transaction)
        .await
        .expect("atomic fixture actor");
        fixture_ready_tx
            .send(actor_id)
            .expect("signal atomic fixture actor creation");
        commit_rx.await.expect("continue atomic fixture creation");
        let client_id = unique_client_id();
        let client_pk: Uuid = sqlx::query_scalar(
            "INSERT INTO api_clients (
                 client_id,display_name,description,status,
                 created_by_staff_user_id,updated_by_staff_user_id
             ) VALUES ($1,'Fixture isolation GREEN',NULL,'ACTIVE',$2,$2)
             RETURNING id",
        )
        .bind(&client_id)
        .bind(actor_id)
        .fetch_one(&mut *transaction)
        .await
        .expect("atomic fixture client");
        transaction
            .commit()
            .await
            .expect("commit atomic external fixture transaction");
        committed_tx.send(client_pk).expect("signal fixture commit");
        release_rx.await.expect("release external fixture lock");
    });
    let actor_id = fixture_ready_rx
        .await
        .expect("atomic fixture actor became ready");

    let staff_setup = setup.clone();
    let staff_application_name_for_task = staff_application_name.clone();
    let staff = tokio::spawn(async move {
        let _fixture_lock =
            common::acquire_test_fixture_lock_named(&staff_application_name_for_task).await;
        sqlx::query("DELETE FROM staff_users WHERE id=$1")
            .bind(actor_id)
            .execute(&staff_setup)
            .await
    });
    wait_for_fixture_lock_wait(&setup, &staff_application_name).await;
    commit_tx
        .send(())
        .expect("continue atomic external fixture");
    let client_pk = committed_rx.await.expect("external fixture committed");

    let body_setup = setup.clone();
    let cleanup_setup = setup.clone();
    common::run_fixture_body_with_cleanup(
        move || async move {
            let client_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM api_clients WHERE id=$1")
                    .bind(client_pk)
                    .fetch_one(&body_setup)
                    .await
                    .expect("inspect committed external fixture");
            assert_eq!(client_count, 1);
        },
        move || async move {
            let cleanup_result = cleanup_actor_client(&cleanup_setup, actor_id, client_pk).await;
            let release_result = release_tx
                .send(())
                .map_err(|_| "release external fixture lock channel closed".to_owned());
            match (cleanup_result, release_result) {
                (Ok(()), Ok(())) => Ok(()),
                (Err(error), Ok(())) => Err(error.to_string()),
                (Ok(()), Err(error)) => Err(error),
                (Err(cleanup_error), Err(release_error)) => {
                    Err(format!("{cleanup_error}; {release_error}"))
                }
            }
        },
    )
    .await;
    staff
        .await
        .expect("staff cleanup task")
        .expect("staff cleanup runs after owned fixture cleanup");
    external.await.expect("external fixture task");
}

#[tokio::test]
async fn committed_external_fixture_is_cleaned_before_staff_cleanup() {
    let (setup, _runtime) = pools().await;
    let _external_lock = common::acquire_test_fixture_lock_named("b25-fixture-committed").await;
    let fixture = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let (start_tx, start_rx) = tokio::sync::oneshot::channel();
    let staff_application_name = format!("b25-fixture-staff-{}", Uuid::new_v4());
    let staff_application_name_for_task = staff_application_name.clone();
    let actor_id = fixture.actor_id;
    let cleanup_setup = setup.clone();
    let cleanup_fixture = fixture.clone();
    let (staff_tx, staff_rx) = tokio::sync::oneshot::channel();
    let body_setup = setup.clone();
    let staff_setup = setup.clone();
    common::run_fixture_body_with_cleanup(
        move || async move {
            let staff_cleanup = tokio::spawn(async move {
                start_rx.await.expect("start staff cleanup");
                let _fixture_lock =
                    common::acquire_test_fixture_lock_named(&staff_application_name_for_task).await;
                sqlx::query("DELETE FROM staff_users WHERE id=$1")
                    .bind(actor_id)
                    .execute(&staff_setup)
                    .await
            });
            staff_tx
                .send(staff_cleanup)
                .expect("signal staff cleanup task");
            start_tx.send(()).expect("start staff cleanup task");
            wait_for_fixture_lock_wait(&body_setup, &staff_application_name).await;
        },
        move || async move {
            cleanup(&cleanup_setup, &cleanup_fixture)
                .await
                .map_err(|error| error.to_string())
        },
    )
    .await;
    drop(_external_lock);
    let staff_cleanup = staff_rx.await.expect("staff cleanup task handle");
    let cleanup_result = staff_cleanup.await.expect("staff cleanup task");
    cleanup_result.expect("staff cleanup must wait for owned fixture cleanup");
}

#[tokio::test]
async fn owned_fixture_cleanup_preserves_unrelated_fixture_rows() {
    let (setup, _runtime) = pools().await;
    let _fixture_lock = common::acquire_test_fixture_lock().await;
    let owned = fixture(&setup, &[ApiClientScope::FlightsRead], "ACTIVE").await;
    let unrelated = fixture(&setup, &[ApiClientScope::AnalyticsRead], "ACTIVE").await;

    let body_setup = setup.clone();
    let cleanup_setup = setup.clone();
    let body_owned = owned.clone();
    let body_unrelated = unrelated.clone();
    let cleanup_owned = owned.clone();
    let cleanup_unrelated = unrelated.clone();
    common::run_fixture_body_with_cleanup(
        move || async move {
            cleanup(&body_setup, &body_owned)
                .await
                .expect("owned fixture cleanup");
            let owned_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM api_clients WHERE id=$1")
                    .bind(body_owned.client_pk)
                    .fetch_one(&body_setup)
                    .await
                    .expect("inspect owned fixture");
            let unrelated_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM api_clients WHERE id=$1")
                    .bind(body_unrelated.client_pk)
                    .fetch_one(&body_setup)
                    .await
                    .expect("inspect unrelated fixture");
            assert_eq!(owned_count, 0);
            assert_eq!(unrelated_count, 1);
        },
        move || async move {
            cleanup_fixtures(&cleanup_setup, &[&cleanup_owned, &cleanup_unrelated]).await
        },
    )
    .await;
}
