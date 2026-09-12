# Branch 25 External REST API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect the Branch 24 API-client registry to one-time client-secret issuance, short-lived opaque bearer tokens, current relational scope enforcement, and the approved read-only External REST API.

**Architecture:** Keep staff sessions and external authentication as separate security domains. A Rust application service owns credential/token lifecycle contracts; SQLx repositories persist only HMAC credential verifiers and SHA-256 token hashes; Axum authenticates external callers before typed scope guards and dedicated DTOs. The external namespace is `/api/v1/external`, with no Next.js BFF or browser CORS allowance.

**Tech Stack:** Rust 1.98 / Axum 0.8 / Tokio / SQLx 0.8 / PostgreSQL 18 / `rand` / `hmac` / `sha2` / `hex` / `subtle`; Next.js 16 / React 19 / TypeScript / Jest and Testing Library; existing Cloudflare/Nginx edge for deployment controls.

**Spec:** `docs/superpowers/specs/2026-09-11-branch-25-external-rest-api-design.md`

## Global Constraints

- Stay on `feat/25-external-rest-api`; expected base is `2f7628099f11c8b4fe8dd11d24cf766df07f6282`.
- During this planning turn, modify only the spec and this plan; do not implement product code, migrations, dependencies, CI, or documentation outside these two files.
- During implementation, do not stage, commit, push, merge, switch branches, deploy, or automatically perform Git operations; each task ends with `STOP — ready for user review/staging`.
- Staff authentication remains the PostgreSQL-backed opaque `x_fly_staff_session` cookie; external routes never accept staff or customer cookies.
- Use Option B: long-lived client credential exchanged for a 15-minute opaque access token; do not implement JWT or OAuth2 authorization-server compliance.
- Generate 32-byte client secrets and 256-bit access tokens with cryptographically secure randomness.
- Persist only HMAC-SHA-256 credential verifiers and SHA-256 access-token hashes; never persist plaintext secret/token material.
- Use distinct non-observable `ExternalApiCredentialPepper([u8; 32])`, `PlaintextClientSecret([u8; 32])`, `PlaintextAccessToken([u8; 32])`, `CredentialDigest([u8; 32])`, and `AccessTokenHash([u8; 32])` types. None implements `Debug`, `Display`, or `Serialize`; bytes are available only through narrow internal accessors.
- `EXTERNAL_API_CREDENTIAL_PEPPER_V1` is required by normal DEV, TEST, and production application startup, exactly 32 decoded random bytes represented by 64 hexadecimal characters, and identical across API instances. Only isolated unit tests may inject explicit bytes into the crypto constructor; no application/test configuration path may default or bypass validation.
- Parse the pepper into its secret wrapper rather than retaining a raw `String`; replace automatic `AppConfig: Debug` with a manual redacted implementation that redacts every credential-bearing field (database URL, existing signing/payment secrets, and the pepper). Formatting config/crypto/auth structures with sentinel values must never render pepper, secret, token, digest, or hash material.
- Token responses contain only `accessToken`, `tokenType`, and `expiresIn`; they contain no scope snapshot.
- The token request accepts exactly `clientId` and `clientSecret` through strict deserialization. Malformed JSON, missing fields, wrong JSON types, unknown fields (including `scope`/`scopes`), non-canonical field representations, and values over approved structural bounds return `400 EXTERNAL_REQUEST_INVALID` before authentication. Only a representation-valid request enters authentication; unknown well-formed clients, wrong secrets, suspended/revoked clients, revoked credentials, and other credential-state failures return the same generic `401 EXTERNAL_CLIENT_AUTHENTICATION_FAILED`.
- Use current client state, credential state, token expiry/revocation, and relational scopes for every protected request.
- Enforce at most one unrevoked credential per client; support revoke-then-replace, not overlapping credentials.
- Use `/api/v1/external/token`, `/api/v1/external/flights`, `/api/v1/external/flights/{flightPublicId}`, and `/api/v1/external/analytics/summary` exactly.
- Analytics initially contains only aggregate non-financial operational metrics: total bookings, tickets issued, cancelled bookings, booked seats, sellable seats, and occupancy percentage. Its cohort is authoritative flight departure converted to `Asia/Bangkok`, with inclusive `[from,to]` dates, all supported successful providers, no provider filter, current Business/First inventory only, and `0.0` occupancy when sellable capacity is zero.
- No application-level `429 EXTERNAL_RATE_LIMITED` contract exists in Branch 25; token-endpoint abuse limiting is a documented Cloudflare/Nginx/equivalent go-live dependency.
- Never expose internal UUIDs, passenger/contact PII, booking references, ticket numbers, payment identifiers, revenue, refunds, staff data, or audit history through external DTOs. External flight DTOs expose `aircraftCode`, not an unavailable aircraft description.
- Preserve the pre-25 cross-process-safe TEST fixture isolation model; do not reintroduce random/UUID-modulo dates, process-local collision protection, global test serialization, or broad table clearing.
- DEV is `127.0.0.1:5433`, database `x_fly`, and must not be mutated by tests or setup. TEST is `127.0.0.1:5434`, database `x_fly_concurrency_test`.
- Before every database-backed implementation task, prove `TEST_DATABASE_URL` targets the TEST database through `x_fly_migrator` and `TEST_RUNTIME_DATABASE_URL` targets the same host/port/database through `x_fly_runtime`; `DATABASE_URL` is the normal Axum runtime URL for `x_fly_runtime` in an application process, but integration-test commands deliberately override it with an invalid/non-DEV value for safety. `MIGRATION_DATABASE_URL` is the operator/local `x_fly_migrator` variable used by `db_admin`, not the normal Axum URL.
- Never run `docker compose down -v`; never edit an already-applied migration; add the new migration append-only.
- Preserve the existing `next typegen` CI correction and RustSec reachability guard.
- Application-table terminology is authoritative: `33 → 35` application tables; `34 → 36` total public tables including `_sqlx_migrations`; migration count `26 → 27`.
- CI must generate a masked ephemeral `EXTERNAL_API_CREDENTIAL_PEPPER_V1=$(openssl rand -hex 32)` and export it before any backend command that constructs application configuration; local TEST guidance is `export EXTERNAL_API_CREDENTIAL_PEPPER_V1="$(openssl rand -hex 32)"` in a private ignored environment (never a checked-in `.env`), while DEV/production remain operator-managed.

---

### Task 1: Domain credential/token types and crypto/config contract

**Files:**
- Create: `backend/src/domain/external_api.rs`
- Create: `backend/src/application/external_auth.rs`
- Modify: `backend/src/domain/mod.rs`
- Modify: `backend/src/application/mod.rs`
- Modify: `backend/src/config.rs`
- Modify: `backend/.env.example`
- Test: `backend/tests/external_auth_rules.rs`

**Interfaces:**
- Consumes: `ApiClientScope`, `ApiClientStatus`, `PermissionCode`, existing `Arc<dyn ...>` repository conventions, UUID/chrono conventions, and `AppConfig::from_env`.
- Produces these non-observable value types in `backend/src/domain/external_api.rs`: `ExternalApiCredentialPepper([u8; 32])`, `PlaintextClientSecret([u8; 32])`, `PlaintextAccessToken([u8; 32])`, `CredentialDigest([u8; 32])`, and `AccessTokenHash([u8; 32])`. None implements `Debug`, `Display`, or `Serialize`; each exposes only `as_bytes(&self) -> &[u8; 32]` to internal crypto/repository code. `PlaintextClientSecret::parse_hex(&str) -> Result<Self, CredentialFormatError>` and `PlaintextAccessToken::parse_bearer_text(&str) -> Result<Self, CredentialFormatError>` enforce the approved encodings. `ExternalApiCredentialPepper::parse_hex(&str) -> Result<Self, PepperConfigError>` enforces exactly 64 hexadecimal characters and 32 decoded bytes.
- Produces: `ExternalPrincipal { api_client_id: Uuid, client_id: String, scopes: BTreeSet<ApiClientScope> }` with `api_client_id(&self) -> Uuid`, `client_id(&self) -> &str`, and `scopes(&self) -> &BTreeSet<ApiClientScope>` accessors; `CredentialMetadata { credential_id: Uuid, issued_at: DateTime<Utc>, revoked_at: Option<DateTime<Utc>>, revocation_reason: Option<CredentialRevocationReason> }`; `IssuedCredential { secret: PlaintextClientSecret, metadata: CredentialMetadata }`; `IssuedAccessToken { token: PlaintextAccessToken, expires_at: DateTime<Utc> }`; and `ACCESS_TOKEN_TTL: Duration = Duration::from_secs(900)`. The credential/token result types and principal are never directly serialized.
- Produces `CredentialRevocationReason::{AdminRequest, ClientSuspended, ClientRevoked, Replaced}` with `as_str(&self) -> &'static str`; error enums with no secret-bearing fields: `CredentialFormatError`, `PepperConfigError`, `CredentialAdministrationError::{NotFound, VersionConflict, RevokedClient, LiveCredential, NoLiveCredential, Infrastructure}`, `ExternalTokenExchangeError::{InvalidCredential, Unavailable}`, and `ExternalAuthenticationError::{InvalidToken, Unavailable}`.
- Produces `ExternalCredentialCrypto: Send + Sync` with exact methods `generate_client_secret(&self) -> PlaintextClientSecret`, `credential_digest(&self, public_client_id: &str, secret: &PlaintextClientSecret) -> CredentialDigest`, `dummy_credential_digest(&self, public_client_id: &str) -> CredentialDigest`, `digest_matches(&self, supplied: &CredentialDigest, stored: &CredentialDigest) -> bool`, `generate_access_token(&self) -> PlaintextAccessToken`, and `access_token_hash(&self, token: &PlaintextAccessToken) -> AccessTokenHash`. `ExternalAuthService::exchange` computes both the candidate and dummy HMAC before the repository transaction; the repository uses the candidate for a live credential and the unknown-client path discards both after equivalent dummy work without issuing a token.
- Produces `ExternalAuthRepository: Send + Sync` with exact async methods: `issue_credential(&self, client_id: &str, actor_staff_user_id: Uuid, expected_client_version: i64, digest: &CredentialDigest, issued_at: DateTime<Utc>) -> Result<CredentialMetadata, CredentialAdministrationError>`; `revoke_credential(&self, client_id: &str, actor_staff_user_id: Uuid, expected_client_version: i64, reason: CredentialRevocationReason, revoked_at: DateTime<Utc>) -> Result<CredentialMetadata, CredentialAdministrationError>`; `issue_access_token(&self, client_id: &str, digest: &CredentialDigest, token_hash: &AccessTokenHash, issued_at: DateTime<Utc>, expires_at: DateTime<Utc>) -> Result<(), ExternalTokenExchangeError>`; and `authenticate_access_token(&self, token_hash: &AccessTokenHash, now: DateTime<Utc>) -> Result<ExternalPrincipal, ExternalAuthenticationError>`. `issue_access_token` locks `api_clients FOR UPDATE`, loads the live credential and current client state under that lock, compares the digest, inserts the token hash before commit, and loads current scopes without an inner-join-created authentication failure.
- Produces constructors `ApiClientCredentialService::new(repository: Arc<dyn ExternalAuthRepository>, crypto: Arc<dyn ExternalCredentialCrypto>) -> ApiClientCredentialService` and `ExternalAuthService::new(repository: Arc<dyn ExternalAuthRepository>, crypto: Arc<dyn ExternalCredentialCrypto>) -> ExternalAuthService`; `ApiClientCredentialService::issue(&self, actor_staff_user_id: Uuid, client_id: &str, expected_client_version: i64, issued_at: DateTime<Utc>) -> Result<IssuedCredential, CredentialAdministrationError>` and `revoke(&self, actor_staff_user_id: Uuid, client_id: &str, expected_client_version: i64, reason: CredentialRevocationReason, revoked_at: DateTime<Utc>) -> Result<CredentialMetadata, CredentialAdministrationError>`; and `ExternalAuthService::exchange(&self, client_id: &str, client_secret_text: &str, now: DateTime<Utc>) -> Result<IssuedAccessToken, ExternalTokenExchangeError>` plus `authenticate_bearer(&self, authorization_header: &str, now: DateTime<Utc>) -> Result<ExternalPrincipal, ExternalAuthenticationError>`.

- [ ] **Write the failing test:** Add `client_secret_is_exactly_32_bytes_and_has_no_observable_traits`, `access_token_parser_accepts_only_xfa_v1_hex`, `credential_digest_binds_client_id`, `wrong_plaintext_digest_does_not_verify`, `token_ttl_is_900_seconds`, `external_principal_contains_current_scopes_without_serializing_internal_uuid`, `config_rejects_missing_or_malformed_external_pepper`, and `sentinel_secret_material_never_appears_in_formatted_config_crypto_or_auth_types` to `backend/tests/external_auth_rules.rs`.
- [ ] **Run the failing test and state expected failure:** Run `cargo test --test external_auth_rules`; expect compilation failures for the absent module/types and assertions for the missing pepper contract.
- [ ] **Implement the minimum production change:** Add strict constructors for 64-character lowercase hexadecimal secrets/tokens, define all five non-observable secret/verifier types and exact interfaces above, parse the pepper wrapper in `AppConfig`, replace automatic `AppConfig: Debug` with a manual redacted implementation, add the crypto trait and fixed 900-second constant, and validate `EXTERNAL_API_CREDENTIAL_PEPPER_V1` without a default or test bypass.
- [ ] **Run the focused test:** Run `cargo test --test external_auth_rules`; expect all format, non-observability, digest-binding, TTL, and configuration tests to pass.
- [ ] **Run the relevant regression set:** Run `cargo test --test api_client_rules --test staff_auth_rules --test test_database_config`; expect existing Branch 24/staff/test-safety behavior to remain green.
- [ ] **Review the diff:** Confirm no pepper, plaintext secret/token, digest, hash, principal, or issued-result type derives `Debug`, `Display`, or `Serialize`; confirm manual config formatting is redacted; confirm no staff session type is imported by external-auth contracts; confirm the pepper is not defaulted or silently bypassed in tests.
- [ ] **STOP — ready for user review/staging.**

### Task 2: Append-only migration, ownership, grants, and inventory assertions

**Files:**
- Create: `backend/migrations/20260911000100_create_external_api_auth.sql`
- Modify: `backend/provisioning/runtime_grants.sql`
- Modify: `backend/provisioning/existing_dev_ownership.sql`
- Modify: `.github/workflows/ci.yml`
- Modify: `backend/tests/database_lifecycle.rs`
- Modify: `backend/tests/runtime_database_permissions.rs`
- Test: `backend/tests/database_lifecycle.rs`, `backend/tests/runtime_database_permissions.rs`

**Interfaces:**
- Consumes: Task 1 `ExternalApiCredentialPepper`, `CredentialDigest`, `AccessTokenHash`, `CredentialMetadata`, `CredentialRevocationReason`, and exact field constraints; existing `x_fly_migrator` ownership and runtime grant conventions.
- Produces: `api_client_credentials (id, api_client_id, secret_digest, digest_version, issued_at, issued_by_staff_user_id, revoked_at, revoked_by_staff_user_id, revocation_reason)` with `secret_digest` `BYTEA` length 32, `digest_version SMALLINT CHECK (digest_version = 1)`, the four-value revocation-reason check, `ON DELETE RESTRICT` staff/client FKs, and partial unique index `api_client_credentials_one_live_idx` on `(api_client_id) WHERE revoked_at IS NULL`.
- Produces: `external_access_tokens (id, api_client_credential_id, token_hash, issued_at, expires_at, revoked_at)` with unique 32-byte hash constraint/index `external_access_tokens_token_hash_key`, `expires_at > issued_at`, `ON DELETE RESTRICT` credential FK, and partial live-token revocation index `external_access_tokens_credential_live_idx` on `(api_client_credential_id) WHERE revoked_at IS NULL`.
- Produces an additive alteration of `api_client_management_audit`: nullable `credential_id UUID REFERENCES api_client_credentials(id) ON DELETE RESTRICT`, a partial `(credential_id, created_at DESC, id DESC)` index, an action check containing exactly the six Branch 24 actions (`CLIENT_CREATED`, `CLIENT_METADATA_UPDATED`, `CLIENT_SCOPES_UPDATED`, `CLIENT_ACTIVATED`, `CLIENT_SUSPENDED`, and `CLIENT_REVOKED`) plus exactly `CREDENTIAL_ISSUED` and `CREDENTIAL_REVOKED`, and a context check requiring correlation for only the two credential actions. Existing `before_state`/`after_state` remain the `ApiClientAuditSnapshot` JSON shape.
- Produces exact runtime grant intent. PostgreSQL table-level and column-level `INSERT` privileges are cumulative, so provisioning executes these statements before any replacement grant:

  ```sql
  REVOKE ALL PRIVILEGES ON TABLE public.api_client_management_audit
      FROM x_fly_runtime, PUBLIC;
  REVOKE INSERT ON TABLE public.api_client_management_audit
      FROM x_fly_runtime, PUBLIC;
  ```

  It removes `public.api_client_management_audit` from the existing table-level `GRANT INSERT` list, restores only the approved table-level `SELECT` for `x_fly_runtime`, and then grants the exact audit column list. Existing Branch 24 `insert_audit` writes `(api_client_id, actor_staff_user_id, action, before_state, after_state, created_at)` with `credential_id` omitted/NULL; `CREDENTIAL_ISSUED` and `CREDENTIAL_REVOKED` writes use the same columns plus `credential_id`. The resulting union grant is `INSERT (api_client_id, actor_staff_user_id, action, before_state, after_state, credential_id, created_at)` to `x_fly_runtime`, with no audit write privilege for `PUBLIC` and generated `id` protected. For the two new auth tables, first `REVOKE ALL PRIVILEGES ON TABLE public.api_client_credentials, public.external_access_tokens FROM x_fly_runtime, PUBLIC`, remove both new tables from any existing table-level SELECT/INSERT/UPDATE lists, then grant credentials `SELECT (id, api_client_id, secret_digest, digest_version, issued_at, revoked_at, revocation_reason)` only for verification, safe metadata, and mutation predicates/`RETURNING`, credentials `INSERT (api_client_id, secret_digest, digest_version, issued_at, issued_by_staff_user_id)`, and credentials `UPDATE (revoked_at, revoked_by_staff_user_id, revocation_reason)`; `issued_by_staff_user_id` and `revoked_by_staff_user_id` are write-only attribution fields and application SQL must use explicit projections, never `SELECT *`; grant tokens `SELECT (id, api_client_credential_id, token_hash, issued_at, expires_at, revoked_at)`, tokens `INSERT (api_client_credential_id, token_hash, issued_at, expires_at)`, and tokens `UPDATE (revoked_at)`. No DELETE/TRUNCATE/DDL/ownership/migration-ledger rights. SQL must omit generated `id` columns so these grants execute as written.

**Database safety (required before any database command):**

DEV is `127.0.0.1:5433` / `x_fly`; do not mutate it. TEST is `127.0.0.1:5434` / `x_fly_concurrency_test`. Prove `TEST_DATABASE_URL` is the TEST `x_fly_migrator` setup credential and `TEST_RUNTIME_DATABASE_URL` is the same TEST target using `x_fly_runtime`; keep `DATABASE_URL` invalid/non-DEV. Never run `docker compose down -v`.

- [ ] **Write the failing test:** Extend `database_lifecycle.rs` with separate `fresh_schema_counts_35_application_tables`, `fresh_schema_counts_36_total_public_tables`, `fresh_schema_counts_36_migrator_owned_public_tables`, `fresh_schema_has_27_successful_migrations`, `branch24_audit_actions_remain_valid`, `credential_actions_require_credential_id`, and `credential_correlation_fk_is_restrict`; extend `runtime_database_permissions.rs` with `runtime_has_no_table_level_audit_insert`, `runtime_has_audit_insert_on_approved_columns`, `runtime_has_no_audit_insert_on_protected_columns`, `public_has_no_audit_write_privilege`, `branch24_audit_insert_succeeds_as_runtime`, `credential_issued_audit_insert_succeeds_as_runtime`, `credential_revoked_audit_insert_succeeds_as_runtime`, `runtime_cannot_update_historical_audit_rows`, `runtime_cannot_delete_audit_history`, `runtime_cannot_truncate_audit_history`, `runtime_cannot_alter_or_transfer_audit_table`, `runtime_cannot_update_credential_identity_fields`, `runtime_cannot_update_credential_verifier_fields`, `runtime_cannot_update_credential_issuance_fields`, `runtime_cannot_update_token_identity_fields`, `runtime_cannot_update_token_hash`, `runtime_cannot_update_token_issuance_fields`, `runtime_cannot_delete_or_truncate_auth_history`, `runtime_cannot_alter_or_transfer_auth_tables`, and `runtime_cannot_mutate_migration_ledger`, plus `runtime_can_insert_credential_with_issuance_columns`, `runtime_can_insert_token_with_issuance_columns`, `runtime_can_revoke_credential`, and `runtime_can_revoke_token` proving the required statements execute. The privilege assertions use `has_table_privilege` for table-level `INSERT`, `has_column_privilege` for each approved/protected audit column, and an ACL inspection for any `PUBLIC` write grant.
- [ ] **Run the failing test and state expected failure:** Prove TEST identity, run the focused lifecycle/permission tests against the current schema, and expect the old counts and absent tables/columns to fail the new assertions.
- [ ] **Implement the minimum production change:** Add one append-only migration that creates both auth tables before adding the audit FK, uses `ON DELETE RESTRICT`, exact byte-length/reason/context checks, one-live-credential partial uniqueness, and the live-token index; preserve all historical Branch 24 audit actions and snapshot JSON. In `backend/provisioning/runtime_grants.sql`, explicitly revoke all audit privileges from `x_fly_runtime` and `PUBLIC`, remove `public.api_client_management_audit` from the existing table-level `GRANT INSERT` list, then grant only `INSERT (api_client_id, actor_staff_user_id, action, before_state, after_state, credential_id, created_at)` to `x_fly_runtime`; retain table-level audit `SELECT` only. Apply the exact new-table column grants above and keep generated IDs/protected columns ungranted for insert/update. Update migrator ownership and the exact permission assertions. Update the existing CI inventory step narrowly: retain `SELECT COUNT(*) FILTER (WHERE success) || '|' || COUNT(*) FROM _sqlx_migrations` and assert `27|27`; add/assert `SELECT COUNT(*) FROM pg_tables WHERE schemaname = 'public' AND tablename <> '_sqlx_migrations'` equals `35`; retain/assert total `SELECT COUNT(*) FROM pg_tables WHERE schemaname = 'public'` equals `36`; retain/assert owner `... AND tableowner = 'x_fly_migrator'` equals `36`; do not alter workflow structure.
- [ ] **Run the focused test:** Prove `TEST_DATABASE_URL` is the TEST `x_fly_migrator` URL and `TEST_RUNTIME_DATABASE_URL` is the same TEST target as `x_fly_runtime`; use `MIGRATION_DATABASE_URL` only for the operator-style `db_admin` migration command; apply reviewed grants, then run `cargo test --test database_lifecycle --test runtime_database_permissions`; expect PASS and zero runtime ownership/privilege regressions.
- [ ] **Run the relevant regression set:** Run `cargo test --test test_database_config --test database_lifecycle --test runtime_database_permissions`; expect the existing migration-ledger, role-boundary, and TEST-safety tests to remain green.
- [ ] **Review the diff:** Check every new table remains migrator-owned; verify the SQL uses only columns covered by the exact grants; verify runtime can INSERT credentials/tokens/audit rows and revoke only through allowed columns but cannot UPDATE digest/hash/identity/expiry, DELETE/TRUNCATE history, ALTER/own objects, or mutate `_sqlx_migrations`; confirm CI still preserves `next typegen` and the RustSec checks.
- [ ] **STOP — ready for user review/staging.**

### Task 3: Credential/token repository and serialization-safe transactions

**Files:**
- Create: `backend/src/infrastructure/external_auth_crypto.rs`
- Create: `backend/src/infrastructure/database/external_auth.rs`
- Create: `backend/tests/external_auth_repository.rs`
- Create: `backend/tests/external_auth_concurrency.rs`
- Modify: `backend/src/infrastructure/mod.rs`
- Modify: `backend/src/infrastructure/database/mod.rs`
- Modify: `backend/src/application/external_auth.rs`
- Modify: `backend/src/state.rs`
- Modify: `backend/src/main.rs`

**Interfaces:**
- Consumes: Task 1's exact `ExternalCredentialCrypto`, `ExternalAuthRepository`, `ExternalApiCredentialPepper`, `CredentialDigest`, `AccessTokenHash`, `CredentialMetadata`, and error types, plus Task 2 schema/grants.
- Produces: `HmacExternalCredentialCrypto::from_pepper(pepper: ExternalApiCredentialPepper) -> HmacExternalCredentialCrypto` and `SqlxExternalAuthRepository::new(pool: PgPool) -> SqlxExternalAuthRepository`, implementing every Task 1 repository signature.
- Produces: safe `CredentialMetadata` materialized inside credential mutation transactions and `ExternalPrincipal` materialized inside token-authentication queries; no raw secret/token values leave crypto/repository boundaries except the one-time application result.
- Produces: row-mutation helper `revoke_client_credentials_and_tokens(tx: &mut Transaction<'_, Postgres>, client_id: Uuid, reason: CredentialRevocationReason, revoked_at: DateTime<Utc>) -> Result<Option<CredentialMetadata>, CredentialAdministrationError>` for the existing client suspend/revoke paths. It returns the revoked credential metadata when one existed but does not write audit rows; the caller supplies `actor_staff_user_id` and before/after client snapshots to append the correlated `CREDENTIAL_REVOKED` event in the same transaction. Every lifecycle path locks `api_clients` first, then credential/token rows in the same order.

**Database safety (required before any database command):**

DEV is `127.0.0.1:5433` / `x_fly`; do not mutate it. TEST is `127.0.0.1:5434` / `x_fly_concurrency_test`. Prove both TEST URLs target that exact database with `x_fly_migrator` setup and `x_fly_runtime` application credentials; keep `DATABASE_URL` invalid/non-DEV. Never run `docker compose down -v`.

- [ ] **Write the failing test:** Add `persists_only_credential_hmac_digest`, `never_persists_plaintext_client_secret`, `persists_only_access_token_sha256`, `plaintext_digest_cannot_authenticate`, `rejects_duplicate_live_credential`, `revokes_credential_and_live_tokens_atomically`, `zero_scope_token_returns_empty_principal`, and `mutation_materializes_before_commit` to `external_auth_repository.rs`; add barrier-coordinated `simultaneous_issue_has_one_success`, `token_issue_serializes_after_suspend`, `token_issue_serializes_after_client_revoke`, `token_issue_serializes_after_credential_revoke`, `credential_issue_vs_suspend_has_one_final_credential_and_no_usable_token`, `credential_issue_vs_client_revoke_has_no_live_credential_or_token`, `credential_issue_vs_credential_revoke_allows_replacement_after_revoke`, `scope_removal_before_auth_is_denied`, and `scope_removal_after_guard_allows_in_flight` to `external_auth_concurrency.rs`. Do not use sleeps.
- [ ] **Run the failing test and state expected failure:** Prove TEST identity and run `cargo test --test external_auth_repository --test external_auth_concurrency`; expect absent repository implementation/schema behavior to fail.
- [ ] **Implement the minimum production change:** Implement HMAC crypto with `subtle::ConstantTimeEq`; have the exchange service compute candidate and dummy HMACs before the repository call; use transactions that lock `api_clients FOR UPDATE`, select the live credential under that lock, compare the candidate fixed digest, require ACTIVE/live state, insert the token hash before commit, and revoke live tokens in the same client-row transaction. An unknown client still completes the dummy HMAC path and returns the generic exchange failure without inserting a token. Use `LEFT JOIN`/typed aggregation (or an equivalent two-step read) so zero scope rows still return an authenticated empty `ExternalPrincipal`. Make credential issuance, client suspension/revocation, and credential revocation/replacement acquire the client lock first and implement the exact race outcomes in the spec. Use `RETURNING`/in-transaction materialization and no post-commit reload.
- [ ] **Run the focused test:** With the proven TEST setup and runtime grants, run `cargo test --test external_auth_repository --test external_auth_concurrency`; expect all persistence, digest secrecy, rollback, and serialization tests to pass.
- [ ] **Run the relevant regression set:** Run `cargo test --test api_client_repository --test api_client_rules --test database_lifecycle --test runtime_database_permissions`; expect Branch 24 optimistic/concurrency and grant behavior to remain green.
- [ ] **Review the diff:** Confirm the token issuance linearization point is the locked-client transaction; prove no token can be inserted after a committed suspend/revoke; prove issuance/revocation races leave the stated credential invariants; confirm scope-removal barriers implement the permitted in-flight rule; confirm secret/token-bearing structs cannot be logged or serialized generically.
- [ ] **STOP — ready for user review/staging.**

### Task 4: Staff API-client credential administration HTTP

**Files:**
- Modify: `backend/src/application/api_client.rs`
- Modify: `backend/src/infrastructure/database/api_client.rs`
- Modify: `backend/src/infrastructure/http/admin/mod.rs`
- Modify: `backend/src/state.rs`
- Modify: `backend/src/main.rs`
- Create: `backend/tests/admin_external_credentials_http.rs`
- Modify: `backend/tests/admin_api_clients_http.rs`

**Interfaces:**
- Consumes: Task 1 exact `ApiClientCredentialService::issue(&self, actor_staff_user_id: Uuid, client_id: &str, expected_client_version: i64, issued_at: DateTime<Utc>) -> Result<IssuedCredential, CredentialAdministrationError>` and `revoke(&self, actor_staff_user_id: Uuid, client_id: &str, expected_client_version: i64, reason: CredentialRevocationReason, revoked_at: DateTime<Utc>) -> Result<CredentialMetadata, CredentialAdministrationError>`; Task 3 `revoke_client_credentials_and_tokens(tx: &mut Transaction<'_, Postgres>, client_id: Uuid, reason: CredentialRevocationReason, revoked_at: DateTime<Utc>) -> Result<Option<CredentialMetadata>, CredentialAdministrationError>`; existing `AuthenticatedStaff`, `PermissionCode::ApiClientsManage`, `AdminApiError`, and optimistic client versioning.
- Produces strict request DTOs `IssueCredentialRequest { version: i64 }` and `RevokeCredentialRequest { version: i64 }`; the admin route always supplies the system-owned `CredentialRevocationReason::AdminRequest`, while `ClientSuspended`, `ClientRevoked`, and `Replaced` are supplied only by internal lifecycle transactions. It also produces safe `CredentialMetadataResponse { has_live_credential: bool, issued_at: Option<DateTime<Utc>>, revoked_at: Option<DateTime<Utc>> }` with `serde(rename_all = "camelCase")` output (`hasLiveCredential`, `issuedAt`, `revokedAt`); timestamps describe the newest credential row by `issued_at`, whether currently live or most recently revoked, and no internal ID/digest/hash is included.
- Produces routes `POST /api/v1/admin/api-clients/{client_id}/credentials` and `POST /api/v1/admin/api-clients/{client_id}/credentials/revoke`, each extracting `AuthenticatedStaff`, requiring `PermissionCode::ApiClientsManage`, validating the body version, and calling the exact service methods above.
- Modifies `ApiClientDetail` to add a `credential_metadata: CredentialMetadataResponse` field serialized as `credentialMetadata`; the existing flattened client record, audit list, and six Branch 24 snapshot/action fields remain unchanged.
- Produces one adapter-only response DTO `CredentialIssuanceResponse { client_id: String, client_secret: String, issued_at: DateTime<Utc> }` with `serde(rename_all = "camelCase")` output (`clientId`, `clientSecret`, `issuedAt`). The adapter obtains the secret bytes from `IssuedCredential.secret` exactly once, encodes lowercase hex, constructs `Json<CredentialIssuanceResponse>`, and never returns the secret-bearing application type directly.
- Produces `CREDENTIAL_ISSUED`/`CREDENTIAL_REVOKED` audit actions using the nullable internal `credential_id` correlation while preserving the existing `ApiClientAuditSnapshot` JSON. Public/admin detail exposes only `CredentialMetadataResponse`; generic safe mappings cover not-found, stale-version, revoked-client, no-live-credential, and already-live-credential errors.
- Modifies `ApiClientAuditAction` and its parser/`as_str` mapping to support exactly the six existing Branch 24 values plus `CredentialIssued`/`CredentialRevoked`; `ApiClientAuditEntry` and `ApiClientAuditSnapshot` remain backward-compatible, and the internal `credential_id` column is not added to either browser DTO.

**Database safety (required before any database command):**

DEV is `127.0.0.1:5433` / `x_fly`; do not mutate it. TEST is `127.0.0.1:5434` / `x_fly_concurrency_test`. Prove TEST setup/runtime URL identity and role separation before DB-backed HTTP tests; keep `DATABASE_URL` invalid/non-DEV. Never run `docker compose down -v`.

- [ ] **Write the failing test:** Add `issue_requires_api_clients_manage_and_csrf_origin`, `issue_returns_secret_once_and_safe_metadata`, `detail_never_returns_digest_or_secret`, `revoke_requires_current_version`, `admin_revoke_cannot_supply_system_reason`, `revoke_is_audited_with_credential_context`, `client_suspend_and_revoke_audit_correlates_credential_when_present`, `revoked_client_cannot_issue`, and `existing_branch24_audit_actions_still_render` to `admin_external_credentials_http.rs`; add regression assertions to `admin_api_clients_http.rs` that ordinary client DTOs remain secret-free. Leave transport-loss/no-retry behavior to Task 9's frontend/BFF tests.
- [ ] **Run the failing test and state expected failure:** Prove TEST identity, run `cargo test --test admin_external_credentials_http --test admin_api_clients_http`; expect missing routes/service methods and absent credential metadata.
- [ ] **Implement the minimum production change:** Add the two staff-protected routes, require effective `api_clients:manage`, enforce exact Origin/CSRF for mutations, pass the current version, and return private/no-store responses. Add detail metadata without verifier fields, map the one-time secret only in the HTTP adapter, preserve all Branch 24 audit actions/snapshot parsing, call `revoke_client_credentials_and_tokens` from client suspension/revocation while the client row is already locked, and when it returns metadata append the correlated `CREDENTIAL_REVOKED` audit row with the existing actor/snapshot helper before the same transaction commits.
- [ ] **Run the focused test:** Run the two focused HTTP suites against TEST; expect authorization, one-time DTO, audit, lifecycle, no-store, and optimistic-version tests to pass.
- [ ] **Run the relevant regression set:** Run `cargo test --test admin_api_clients_http --test staff_auth_http --test staff_auth_rules`; expect SYSTEM_ADMIN and unrelated roles to remain denied through permission mapping.
- [ ] **Review the diff:** Confirm no role-name string bypass, staff session reuse outside the existing extractor, secret in errors/logs/DTOs, credential UUID in browser output, or post-commit database reload; confirm `CREDENTIAL_ISSUED`/`CREDENTIAL_REVOKED` rows have non-null correlation and REVOKED remains terminal.
- [ ] **STOP — ready for user review/staging.**

### Task 5: External token endpoint and bearer authentication

**Files:**
- Create: `backend/src/infrastructure/http/external.rs`
- Create: `backend/tests/external_token_http.rs`
- Modify: `backend/src/infrastructure/http/mod.rs`
- Modify: `backend/src/state.rs`
- Modify: `backend/src/main.rs`
- Modify: `backend/src/infrastructure/http/request_tracing.rs`

**Interfaces:**
- Consumes: Task 3 exact `ExternalAuthService::exchange(&self, client_id: &str, client_secret_text: &str, now: DateTime<Utc>) -> Result<IssuedAccessToken, ExternalTokenExchangeError>` and `authenticate_bearer(&self, authorization_header: &str, now: DateTime<Utc>) -> Result<ExternalPrincipal, ExternalAuthenticationError>`, `ExternalPrincipal`, `PlaintextAccessToken`, and `ACCESS_TOKEN_TTL`.
- Produces strict `TokenExchangeRequest { client_id: String, client_secret: String }` with required fields and unknown-field rejection. The adapter validates JSON shape, canonical Client ID representation, 64-character lowercase-hex secret representation, and structural bounds before calling `ExternalAuthService::exchange`; those failures map to `400 EXTERNAL_REQUEST_INVALID`. Only representation-valid requests enter authentication. It also produces adapter-only `TokenExchangeResponse { access_token: String, token_type: &'static str, expires_in: u64 }` with `serde(rename_all = "camelCase")` output exactly `accessToken`, `tokenType`, and `expiresIn`. The response is built from `IssuedAccessToken` exactly once and contains no scopes; exchange authentication failures map to generic `401 EXTERNAL_CLIENT_AUTHENTICATION_FAILED`.
- Produces `ExternalBearerAuthLayer::new(state: AppState) -> ExternalBearerAuthLayer` and extractor behavior with exact function `authenticate_external_request(state: &AppState, authorization_header: &str, now: DateTime<Utc>) -> Result<ExternalPrincipal, ExternalAuthenticationError>`. It accepts exactly `Authorization: Bearer xfa_v1_<64 lowercase hex>` and inserts `ExternalPrincipal` into request extensions; staff/customer cookies are never consulted.
- Produces generic `EXTERNAL_CLIENT_AUTHENTICATION_FAILED` for token-exchange failures and `EXTERNAL_AUTHENTICATION_FAILED` plus `WWW-Authenticate: Bearer` for protected-route failures. The external router is returned by `external_router(state: AppState) -> Router` without a CORS layer.

**Database safety (required before any database command):**

DEV is `127.0.0.1:5433` / `x_fly`; do not mutate it. TEST is `127.0.0.1:5434` / `x_fly_concurrency_test`. Prove both TEST URLs target the exact TEST database with migrator/runtime roles before token HTTP tests; keep `DATABASE_URL` invalid/non-DEV. Never run `docker compose down -v`.

- [ ] **Write the failing test:** Add `token_exchange_returns_no_scope_snapshot`, `token_exchange_sets_no_store_and_pragma`, `malformed_json_returns_400_external_request_invalid`, `missing_client_id_returns_400_external_request_invalid`, `missing_client_secret_returns_400_external_request_invalid`, `wrong_client_id_type_returns_400_external_request_invalid`, `wrong_client_secret_type_returns_400_external_request_invalid`, `unknown_token_request_field_returns_400_external_request_invalid`, `caller_scope_field_returns_400_external_request_invalid`, `noncanonical_client_id_returns_400_external_request_invalid`, `malformed_client_secret_representation_returns_400_external_request_invalid`, `oversized_token_request_value_returns_400_external_request_invalid`, `unknown_well_formed_client_returns_generic_401`, `wrong_secret_returns_generic_401`, `suspended_client_returns_generic_401`, `revoked_client_returns_generic_401`, `revoked_credential_returns_generic_401`, `reactivation_does_not_revive_revoked_token`, `valid_token_authenticates_principal`, `expired_token_is_generic_401`, `malformed_bearer_is_generic_401`, `duplicate_authorization_headers_fail`, `staff_or_customer_cookies_do_not_authenticate_external_routes`, `external_token_origin_has_no_browser_cors_headers`, and `external_protected_origin_has_no_browser_cors_headers` to `external_token_http.rs`. The 400 tests stop before `ExternalAuthService::exchange`; the 401 tests use representation-valid fields and assert the same generic public envelope.
- [ ] **Run the failing test and state expected failure:** Prove TEST identity and run `cargo test --test external_token_http`; expect the external router, token endpoint, extractor, strict 400/401 split, and CORS isolation to be absent.
- [ ] **Implement the minimum production change:** Add strict body/header parsing with `deny_unknown_fields`, required `clientId`/`clientSecret` strings, structural validation before authentication, one-time response mapping, no-store/private plus `Pragma` headers, generic public errors, `WWW-Authenticate`, bearer token hashing, and the authenticated-principal extension. Map malformed JSON, missing/wrong-type fields, unknown/scope fields, non-canonical representations, and oversized values to `400 EXTERNAL_REQUEST_INVALID` without calling credential authentication; invoke `ExternalAuthService::exchange` only after those checks and map all resulting credential/state failures to generic `401 EXTERNAL_CLIENT_AUTHENTICATION_FAILED`. In `build_router`, move every existing non-external route (health, public/customer, and admin) into a `browser_router` and apply the existing credentialed `CorsLayer` only there; merge a separately constructed `external_router` with no CORS layer, then apply request tracing outside the merged router. Do not add a second CORS subsystem. Keep credential/token values out of tracing fields and error bodies.
- [ ] **Run the focused test:** Run `cargo test --test external_token_http`; expect all token response, state, expiry, header, cookie-separation, and generic-error tests to pass.
- [ ] **Run the relevant regression set:** Run `cargo test --test public_flights_http --test staff_auth_http --test admin_api_clients_http`; expect customer/admin route behavior and CORS/security boundaries to remain unchanged.
- [ ] **Review the diff:** Confirm token JSON has no `scopes` property; malformed JSON, missing/wrong-type fields, unknown/scope fields, non-canonical representations, and oversized values are all `400 EXTERNAL_REQUEST_INVALID` before authentication; representation-valid unknown/wrong/suspended/revoked credentials are all the same generic `401 EXTERNAL_CLIENT_AUTHENTICATION_FAILED`; external routes do not use staff/customer cookies; an `Origin: https://example.test` receives no `Access-Control-Allow-Origin`, `Access-Control-Allow-Credentials`, `Access-Control-Allow-Headers`, or `Access-Control-Allow-Methods` on token/protected responses; invalid states are not distinguishable publicly; and request tracing still excludes headers, bodies, query strings, and cookies.
- [ ] **STOP — ready for user review/staging.**

### Task 6: Typed scope enforcement and external error contract

**Files:**
- Modify: `backend/src/infrastructure/http/external.rs`
- Modify: `backend/src/application/external_auth.rs`
- Create: `backend/tests/external_scope_error_http.rs`
- Modify: `backend/src/infrastructure/http/mod.rs`

**Interfaces:**
- Consumes: Task 5 authenticated `ExternalPrincipal` extension and Task 1 `ApiClientScope`, `ExternalAuthenticationError`, and request DTO rules.
- Produces exact `require_scope(principal: &ExternalPrincipal, required: ApiClientScope) -> Result<(), ExternalScopeError>` guard/layer used by route subrouters. An empty principal scope set is valid authentication and always yields `ExternalScopeError::Missing` (HTTP 403); no caller-provided scope input is accepted.
- Produces `ExternalErrorEnvelope { error: ExternalErrorBody { code: ExternalErrorCode, message: &'static str, request_id: Uuid } }` with `serde(rename_all = "camelCase")` output (`error.requestId`) and `ExternalErrorCode` serialized with `serde(rename_all = "SCREAMING_SNAKE_CASE")`; variants are `ExternalAuthenticationFailed`, `ExternalClientAuthenticationFailed`, `ExternalScopeDenied`, `ExternalRequestInvalid`, `ExternalResourceNotFound`, `ExternalAuthUnavailable`, `ExternalServiceUnavailable`, and `ExternalInternalError`, with `IntoResponse` mappings for 401/403/400/404/503/500 classes.
- Produces: no application-level `429 EXTERNAL_RATE_LIMITED`; edge limiting is documented only.

- [ ] **Write the failing test:** Add `flights_scope_allows_only_flights_read`, `analytics_scope_allows_only_analytics_read`, `zero_scope_principal_is_authenticated_then_denied_403`, `scope_removal_is_seen_on_next_request`, `missing_scope_is_403`, `invalid_request_is_400`, `not_found_is_404`, `dependency_failure_is_503`, `error_contains_request_id_only`, and `no_branch_25_rate_limit_code_exists` to `external_scope_error_http.rs`.
- [ ] **Run the failing test and state expected failure:** Run `cargo test --test external_scope_error_http`; expect absent typed scope layers and error envelope mappings.
- [ ] **Implement the minimum production change:** Create authentication-first route layers, typed route scope checks, empty-scope 403 behavior, strict unknown-field rejection, error serialization with server request ID, no-store headers, safe `WWW-Authenticate`, and generic state/credential failure mapping. Keep the scope decision based only on the `ExternalPrincipal` loaded from current relational rows.
- [ ] **Run the focused test:** Run `cargo test --test external_scope_error_http`; expect all positive/negative scope and stable error contract tests to pass.
- [ ] **Run the relevant regression set:** Run `cargo test --test external_token_http --test public_flights_http --test admin_api_clients_http`; expect no route boundary regression.
- [ ] **Review the diff:** Search the external HTTP module for `429`, caller scope fields, role-name authorization, raw database errors, and direct serialization of `ExternalPrincipal`; each must be absent. Confirm an empty current scope set reaches the typed guard and becomes 403, while a scope removal observed before authentication also becomes 403.
- [ ] **STOP — ready for user review/staging.**

### Task 7: `flights:read` external endpoints and DTOs

**Files:**
- Create: `backend/src/application/external_flights.rs`
- Modify: `backend/src/application/mod.rs`
- Modify: `backend/src/infrastructure/http/external.rs`
- Modify: `backend/src/state.rs`
- Modify: `backend/src/main.rs`
- Create: `backend/tests/external_flights_http.rs`

**Interfaces:**
- Consumes: Task 6 exact `require_scope(principal: &ExternalPrincipal, required: ApiClientScope) -> Result<(), ExternalScopeError>`, existing `FlightManagement::search_public(&self, filter: PublicFlightFilter) -> Result<Vec<PublicFlight>, FlightManagementError>`, `public_detail(&self, public_id: &str, departure: NaiveDate, cabin: CabinClass) -> Result<PublicFlight, FlightManagementError>`, and existing public flight validation rules.
- Produces dedicated non-persistence DTOs `ExternalFlight { flight_public_id: String, flight_number: String, origin_code: String, destination_code: String, departure_time: String, arrival_time: String, arrival_day_offset: u8, duration_minutes: u16, stops: String, aircraft_code: String, status: String, cabin_prices: Vec<ExternalCabinPrice> }`, `ExternalCabinPrice { amount_thb: i64, cabin: String }`, and `ExternalFlightPage { items: Vec<ExternalFlight> }`. There is no aircraft-description field.
- Produces constructor `ExternalFlightService::new(flights: FlightManagement) -> ExternalFlightService`; `ExternalFlightService::search(&self, filter: PublicFlightFilter) -> Result<ExternalFlightPage, FlightManagementError>` and `detail(&self, public_id: &str, departure: NaiveDate, cabin: CabinClass) -> Result<ExternalFlight, FlightManagementError>`, mapping only approved `PublicFlight` fields and never exposing `FlightRecord` or persistence models. DTOs use explicit `serde(rename_all = "camelCase")` mapping, including `aircraftCode`.
- Produces: `GET /api/v1/external/flights` and `GET /api/v1/external/flights/{flightPublicId}` only; no booking, seat-hold, inventory, or reference-data endpoint.

**Database safety (required before any database command):**

DEV is `127.0.0.1:5433` / `x_fly`; do not mutate it. TEST is `127.0.0.1:5434` / `x_fly_concurrency_test`. Prove TEST URL/host/port/database and role identity before external flight tests; use the existing bounded temporal fixture allocator for any required inventory setup; keep `DATABASE_URL` invalid/non-DEV. Never run `docker compose down -v`.

- [ ] **Write the failing test:** Add `flights_search_requires_flights_read`, `flights_search_returns_only_external_fields`, `flight_detail_requires_flights_read`, `flight_detail_uses_public_id_not_internal_uuid`, `flight_dto_exposes_aircraft_code_only`, `invalid_flight_filters_are_400`, `missing_flight_is_404`, and `analytics_scope_cannot_read_flights` to `external_flights_http.rs`.
- [ ] **Run the failing test and state expected failure:** Prove TEST identity and run `cargo test --test external_flights_http`; expect the external flight routes/DTO mapper to be absent.
- [ ] **Implement the minimum production change:** Add the application mapper and handlers, preserve current public validation, use the existing public repository query, map `PublicFlight.aircraft` to external `aircraftCode`, and omit UUID/version/audit/hold/capacity/inventory/staff fields from every external response.
- [ ] **Run the focused test:** Run `cargo test --test external_flights_http`; expect scope, DTO-minimization, public-ID, validation, and not-found tests to pass.
- [ ] **Run the relevant regression set:** Run `cargo test --test public_flights_http --test flight_management_repository --test external_scope_error_http`; expect public customer and staff flight paths to remain green.
- [ ] **Review the diff:** Compare serialized keys against the approved DTO list; confirm `aircraftCode` is sourced from the existing flight-service code and no description model is introduced; confirm no `FlightRecord`, internal UUID, `version`, audit, seat, hold, or capacity field can cross the external adapter.
- [ ] **STOP — ready for user review/staging.**

### Task 8: `analytics:read` aggregate repository and endpoint

**Files:**
- Create: `backend/src/application/external_analytics.rs`
- Create: `backend/src/infrastructure/database/external_analytics.rs`
- Modify: `backend/src/application/mod.rs`
- Modify: `backend/src/infrastructure/database/mod.rs`
- Modify: `backend/src/infrastructure/http/external.rs`
- Modify: `backend/src/state.rs`
- Modify: `backend/src/main.rs`
- Create: `backend/tests/external_analytics_http.rs`
- Create: `backend/tests/external_analytics_repository.rs`

**Interfaces:**
- Consumes: Task 6 exact `require_scope(principal: &ExternalPrincipal, required: ApiClientScope) -> Result<(), ExternalScopeError>`, current date/route/cabin validation conventions, and authoritative `flight_instances`, `flight_services`, `airports`, `seat_holds`, `payment_attempts`, `tickets`, `payment_attempt_seats`, and `flight_seats` tables. It does not consume `DashboardReport` or `AnalyticsFilter.provider`.
- Produces `ExternalAnalyticsFilter { from: NaiveDate, to: NaiveDate, route: Option<String>, cabin: Option<CabinClass> }` and `ExternalAnalyticsFilter::parse(from: Option<&str>, to: Option<&str>, route: Option<&str>, cabin: Option<&str>, now: DateTime<Utc>) -> Result<ExternalAnalyticsFilter, ExternalAnalyticsFilterError>`; provider is intentionally absent.
- Produces `ExternalAnalyticsFilterError::{InvalidDate, InvalidRange, InvalidRoute, InvalidCabin}` and `ExternalAnalyticsRepositoryError::{Infrastructure, InconsistentAggregate}`; neither error type contains sensitive row data.
- Produces `ExternalAnalyticsPeriod { from: NaiveDate, to: NaiveDate }` and `ExternalAnalyticsSummary { period: ExternalAnalyticsPeriod, generated_at: DateTime<Utc>, total_bookings: i64, tickets_issued: i64, cancelled_bookings: i64, booked_seats: i64, sellable_seats: i64, occupancy_percent: f64 }` with `serde(rename_all = "camelCase")` output (`generatedAt`, `totalBookings`, `ticketsIssued`, `cancelledBookings`, `bookedSeats`, `sellableSeats`, `occupancyPercent`), plus constructor `SqlxExternalAnalyticsRepository::new(pool: PgPool) -> SqlxExternalAnalyticsRepository`, constructor `ExternalAnalyticsService::new(repository: Arc<dyn ExternalAnalyticsRepository>) -> ExternalAnalyticsService`, `ExternalAnalyticsRepository::summary(&self, filter: &ExternalAnalyticsFilter) -> Result<ExternalAnalyticsSummary, ExternalAnalyticsRepositoryError>`, and `ExternalAnalyticsService::summary(&self, filter: ExternalAnalyticsFilter, now: DateTime<Utc>) -> Result<ExternalAnalyticsSummary, ExternalAnalyticsRepositoryError>`.
- Produces `GET /api/v1/external/analytics/summary` with inclusive Bangkok dates, maximum 366-day range, optional validated public route and Business/First cabin filters, all supported successful providers, and `0.0` occupancy when sellable capacity is zero. A specified cabin restricts every booking/seat/inventory aggregate to that cabin; no cabin defaults every aggregate to the current `BUSINESS`/`FIRST` set and excludes legacy cabins. The query must not select `DashboardReport`.

**Database safety (required before any database command):**

DEV is `127.0.0.1:5433` / `x_fly`; do not mutate it. TEST is `127.0.0.1:5434` / `x_fly_concurrency_test`. Prove TEST setup/runtime URL identity and roles before aggregate repository tests; use only TEST data; keep `DATABASE_URL` invalid/non-DEV. Never run `docker compose down -v`.

- [ ] **Write the failing test:** Add `summary_returns_only_nonfinancial_aggregate_keys`, `summary_uses_flight_departure_bangkok_cohort_not_payment_date`, `summary_uses_inclusive_bangkok_dates`, `summary_includes_all_supported_successful_providers`, `summary_counts_all_cohort_bookings_including_cancelled`, `summary_counts_issued_tickets_even_when_booking_cancelled`, `summary_counts_cancelled_booking_subset`, `summary_excludes_cancelled_bookings_from_booked_seats`, `summary_computes_occupancy_and_zero_denominator_is_zero_point_zero`, `route_and_cabin_filters_bound_both_numerators_and_denominator`, `analytics_requires_analytics_read`, `flights_scope_cannot_read_analytics`, and `dashboard_report_fields_never_leak` to the two external analytics test files.
- [ ] **Run the failing test and state expected failure:** Prove TEST identity and run `cargo test --test external_analytics_repository --test external_analytics_http`; expect missing aggregate repository/service/route failures.
- [ ] **Implement the minimum production change:** Add one read-only aggregate query rooted in a `flight_cohort` CTE. Inner-join the origin airport, require a non-null `flight_services.departure_time`, derive `departure_at` from `flight_instances.departure_date + flight_services.departure_time` interpreted in `COALESCE(flight_services.origin_time_zone, airports.time_zone)`, convert it to `Asia/Bangkok`, and filter its local date inclusively; apply the validated route filter to the cohort's origin/destination codes. Apply a specified normalized cabin to `seat_holds.cabin` for bookings and `flight_seats.cabin` for both seat and inventory aggregates; when absent, apply the `BUSINESS`/`FIRST` set to all aggregates and exclude legacy cabins. Join `payment_attempts` with `status = 'SUCCEEDED'` (the current `STRIPE` and `MOCK_BITCOIN` providers, without a provider predicate) through `seat_holds` to the cohort for `COUNT(DISTINCT payment_attempts.id) AS total_bookings`; left-join `tickets` for current cancellation state and compute `COUNT(DISTINCT payment_attempts.id) FILTER (WHERE ticket.status = 'CANCELLED') AS cancelled_bookings` plus `COUNT(DISTINCT tickets.id) FILTER (WHERE tickets.issued_at IS NOT NULL) AS tickets_issued`; use separate seat and sellable-inventory aggregates joined back to the cohort so joins cannot multiply counts. The seat aggregate must require `payment_attempt_seats.released_at IS NULL`, `flight_seats.booking_status = 'BOOKED'`, and `ticket.status IS DISTINCT FROM 'CANCELLED'`. The inventory aggregate must require `flight_seats.sellable = TRUE`, current `BUSINESS`/`FIRST` cabins, and `flight_services.status = 'SCHEDULED'`. Include both supported providers, never add a provider filter, count cancelled bookings in total/tickets-issued, exclude them from currently booked seats, count only active Business/First sellable capacity, and return `0.0` for zero denominator. Do not fetch, serialize, or post-filter `DashboardReport`; omit provider references, refunds, revenue, PII, staff, audit, booking references, ticket numbers, and per-flight data.
- [ ] **Run the focused test:** Run the two focused analytics suites against TEST; expect metric semantics, filters, scope, and field-exclusion tests to pass.
- [ ] **Run the relevant regression set:** Run `cargo test --test admin_dashboard_http --test external_scope_error_http --test external_analytics_http`; expect executive dashboard behavior and external boundaries to remain separate.
- [ ] **Review the diff:** Inspect the SQL projection and serialized DTO keys; confirm the flight-departure/Bangkok cohort is the only date basis, all supported successful providers are included without a filter, numerator/denominator filters match, zero capacity returns `0.0`, and no revenue/payment/refund/provider/PII field is selected merely to be discarded later.
- [ ] **STOP — ready for user review/staging.**

### Task 9: Minimal admin credential UI and BFF

**Files:**
- Modify: `frontend/src/lib/admin/apiClientTypes.ts`
- Modify: `frontend/src/lib/admin/adminBackend.ts`
- Modify: `frontend/src/components/admin/api-clients/ApiClientEditor.tsx`
- Modify: `frontend/src/components/admin/api-clients/apiClientPresentation.ts`
- Modify: `frontend/src/components/admin/api-clients/apiClientOperations.css`
- Modify: `frontend/src/i18n/locales/apiClientManagement.ts`
- Create: `frontend/src/app/admin/api/api-clients/[clientId]/credentials/route.ts`
- Create: `frontend/src/app/admin/api/api-clients/[clientId]/credentials/revoke/route.ts`
- Create: `frontend/src/tests/ApiClientCredentials.test.tsx`
- Modify: `frontend/src/tests/ApiClientManagement.test.tsx`
- Modify: `frontend/src/tests/adminBackend.test.ts`

**Interfaces:**
- Consumes: Task 4 routes and DTOs; `POST /api/v1/admin/api-clients/{client_id}/credentials` with `IssueCredentialRequest { version: i64 }`, `POST /api/v1/admin/api-clients/{client_id}/credentials/revoke` with strict `RevokeCredentialRequest { version: i64 }` (the backend supplies `CredentialRevocationReason::AdminRequest`), safe `CredentialMetadataResponse`, and one-time `CredentialIssuanceResponse`.
- Produces exact frontend types `ApiClientCredentialMetadata { hasLiveCredential: boolean, issuedAt: string | null, revokedAt: string | null }`, extends `ApiClientDetail` with `credentialMetadata: ApiClientCredentialMetadata`, and produces ephemeral `IssuedCredentialResponse { clientId: string, clientSecret: string, issuedAt: string }`; only the latter contains a secret and it exists in component state for the current render only.
- Produces BFF functions `issueApiClientCredential(clientId: string, version: number, request: Request) -> Promise<Response>` and `revokeApiClientCredential(clientId: string, version: number, request: Request) -> Promise<Response>`; the BFF sends only `{ version }` and cannot select a system revocation reason. Both functions use canonical path allowlisting, approved staff cookie/content-type/origin/CSRF forwarding, and `Cache-Control: no-store, private` responses.
- Produces an explicit `auditActionKey: Record<ApiClientAuditAction, TranslationKey>` mapping for `CREDENTIAL_ISSUED` and `CREDENTIAL_REVOKED` in `frontend/src/components/admin/api-clients/apiClientPresentation.ts` plus `safeAuditActionKey(action: string): TranslationKey` that maps unknown runtime strings to a generic safe management-event label; labels expose only safe human-readable text and never credential UUIDs, digests, hashes, raw actions, or payloads.
- Produces issue modal, copy control, acknowledgement, dismissal clearing, revoke confirmation, replacement-only-after-revocation UI, and an indeterminate transport-failure state that instructs refresh without retrying or fabricating a secret.

- [ ] **Write the failing test:** Add `issue_button_requires_manage_permission`, `issue_button_disables_while_pending`, `issue_modal_shows_secret_once`, `secret_is_not_written_to_storage_url_or_console`, `dismiss_clears_secret`, `revoke_requires_confirmation`, `reissue_is_disabled_until_revoked`, `credential_metadata_has_no_digest`, `bff_rejects_noncanonical_credential_paths`, `bff_revoke_sends_only_version`, `transport_failure_makes_exactly_one_issue_request`, `transport_failure_does_not_auto_retry`, `refresh_shows_active_credential_without_secret`, `recovery_cta_is_revoke_then_replace`, `credential_audit_actions_have_safe_presentation`, and `unknown_audit_action_uses_safe_fallback_without_payload` to `frontend/src/tests/ApiClientCredentials.test.tsx`; keep existing regression assertions in `frontend/src/tests/ApiClientManagement.test.tsx` and `frontend/src/tests/adminBackend.test.ts`.
- [ ] **Run the failing test and state expected failure:** Run `npm test -- --runInBand src/tests/ApiClientCredentials.test.tsx src/tests/ApiClientManagement.test.tsx src/tests/adminBackend.test.ts`; expect missing routes/state/actions and failing secret-handling assertions.
- [ ] **Implement the minimum production change:** Add typed metadata, BFF handlers, one-time in-memory modal, clipboard action, explicit acknowledgement, revocation/replacement controls, bilingual copy, safe `CREDENTIAL_ISSUED`/`CREDENTIAL_REVOKED` presentation, and no-store forwarding. On an issue transport failure, make no second request; show an indeterminate state, allow refresh to reveal only metadata, and expose recovery as revoke then replacement issue. Do not place secrets in localStorage/sessionStorage, URLs, server-component props, analytics, console, or a recovery endpoint.
- [ ] **Run the focused test:** Run the three focused frontend suites; expect all issue/revoke, memory-lifetime, BFF, and safe-response tests to pass.
- [ ] **Run the relevant regression set:** Run `npm test -- --runInBand src/tests/ApiClientManagement.test.tsx src/tests/AdminShell.test.tsx src/tests/adminAuthorization.test.ts src/tests/adminBackend.test.ts`; expect Branch 24 navigation and permission behavior to remain green.
- [ ] **Review the diff:** Search frontend changes for `localStorage`, `sessionStorage`, URL construction using `clientSecret`, `console.`, secret fields in server components, and unsafe audit-action fallbacks; each must be absent. Confirm refresh never reconstructs a secret and the BFF sends exactly one backend request per explicit issue action.
- [ ] **STOP — ready for user review/staging.**

### Task 10: Security, logging, concurrency, and runtime-permission hardening

**Files:**
- Modify: `backend/src/infrastructure/http/request_tracing.rs`
- Modify: `backend/src/infrastructure/http/external.rs`
- Modify: `backend/src/infrastructure/database/api_client.rs`
- Modify: `backend/src/infrastructure/database/external_auth.rs`
- Modify: `backend/src/config.rs`
- Modify: `backend/tests/runtime_database_permissions.rs`
- Create: `backend/tests/external_security_http.rs`
- Create: `backend/tests/external_logging.rs`
- Modify: `backend/tests/external_auth_concurrency.rs`

**Interfaces:**
- Consumes: all prior exact external-auth, lifecycle, scope, DTO, and grant interfaces.
- Produces `ExternalAuthDiagnostic::{Missing, Malformed, UnknownClient, SecretMismatch, Expired, Suspended, Revoked, ScopeDenied}` and `record_external_auth_diagnostic(span: &tracing::Span, diagnostic: ExternalAuthDiagnostic) -> ()`; the diagnostic contains only a safe category and never a credential/token/header/body/query value.
- Produces explicit security-test helpers `assert_redacted_debug<T: Debug>(value: &T, forbidden: &[&str]) -> ()` for non-secret structures and runtime permission assertions against the exact Task 2 grant matrix.

**Database safety (required before any database command):**

DEV is `127.0.0.1:5433` / `x_fly`; do not mutate it. TEST is `127.0.0.1:5434` / `x_fly_concurrency_test`. Prove TEST setup/runtime URL identity and role separation before permission/concurrency tests; keep `DATABASE_URL` invalid/non-DEV. Never run `docker compose down -v`.

- [ ] **Write the failing test:** Add `sentinel_secret_token_and_authorization_never_enter_logs`, `unknown_client_and_wrong_secret_have_generic_public_outcomes`, `internal_uuid_never_enters_external_json`, `runtime_cannot_update_immutable_verifier_fields`, `runtime_cannot_delete_auth_history`, `suspend_commit_invalidates_next_request`, `revoke_commit_invalidates_next_request`, `scope_removal_barrier_has_both_permitted_outcomes`, `pepper_is_required_in_every_runtime`, and `expired_token_rows_are_not_authorized` to `backend/tests/external_security_http.rs`, `backend/tests/external_logging.rs`, `backend/tests/external_auth_concurrency.rs`, and `backend/tests/runtime_database_permissions.rs` as appropriate.
- [ ] **Run the failing test and state expected failure:** Prove TEST identity, run `cargo test --test external_security_http --test external_logging --test external_auth_concurrency --test runtime_database_permissions`; expect missing diagnostics/redaction and privilege assertions.
- [ ] **Implement the minimum production change:** Add redaction-safe tracing tests/guards, lifecycle and scope-removal barrier coverage, exact column-level grant checks, startup pepper enforcement, and safe error/log category mappings. Generate the pepper only in the CI job environment and never print it. Do not add an application rate limiter or a `429` response.
- [ ] **Run the focused test:** Run the four focused suites against TEST; expect all security, race, logging, and privilege tests to pass.
- [ ] **Run the relevant regression set:** Run `cargo test --locked --no-fail-fast` with guarded TEST setup/runtime credentials; expect the complete backend suite to remain green.
- [ ] **Review the diff:** Map every threat to a test: DB leak (`digest_not_plaintext`), credential theft/replay (`expiry/revoke`), guessing/timing (`dummy HMAC`), state races (`serialization tests`), scope escalation (`scope tests`), header logging (`logging tests`), ID enumeration (`generic auth`), duplicate issuance (`partial unique/concurrency`), analytics PII (`DTO/SQL projection`), UUID disclosure (`JSON tests`), pepper compromise (`config/version/redacted formatting`), and token growth (`retention documented, no cleanup worker`). Confirm no secret-bearing value or diagnostic category enters a log/error/serialized response.
- [ ] **STOP — ready for user review/staging.**

### Task 11: Reconciliation, documentation, and closure verification

**Files:**
- Modify: `backend/README.md`
- Modify: `BACKEND.md`
- Modify: `DATABASE.md`
- Modify: `DESIGN.md`
- Modify: `.github/workflows/ci.yml` only for confirmed Branch 25 inventory/test facts
- Test: `backend/tests/external_auth_rules.rs`
- Test: `backend/tests/database_lifecycle.rs`
- Test: `backend/tests/runtime_database_permissions.rs`
- Test: `backend/tests/external_auth_repository.rs`
- Test: `backend/tests/external_auth_concurrency.rs`
- Test: `backend/tests/admin_external_credentials_http.rs`
- Test: `backend/tests/external_token_http.rs`
- Test: `backend/tests/external_scope_error_http.rs`
- Test: `backend/tests/external_flights_http.rs`
- Test: `backend/tests/external_analytics_repository.rs`
- Test: `backend/tests/external_analytics_http.rs`
- Test: `backend/tests/external_security_http.rs`
- Test: `backend/tests/external_logging.rs`
- Test: `frontend/src/tests/ApiClientCredentials.test.tsx`
- Test: `frontend/src/tests/ApiClientManagement.test.tsx`
- Test: `frontend/src/tests/adminBackend.test.ts`

**Interfaces:**
- Consumes: the completed implementation and the approved spec at `docs/superpowers/specs/2026-09-11-branch-25-external-rest-api-design.md`.
- Produces: documentation for pepper configuration, token exchange, routes, DTO/privacy boundary, grants/ownership, edge go-live dependency, and explicit Branch 26 deferrals.
- Produces: a reconciled Branch 25 statement that names only `flights:read` and `analytics:read`, does not promise application rate limiting, and does not describe revenue analytics or downstream operational APIs.

- [ ] **Write the failing test:** Before edits, run a repository documentation scan that asserts the final Branch 25 references contain the approved routes, `expiresIn: 900`, exact client/token request fields, no token-response scope snapshot, no application `429 EXTERNAL_RATE_LIMITED` contract, `aircraftCode` rather than aircraft description, the flight-departure/Bangkok analytics semantics, and the exact `33 → 35`, `34 → 36`, `26 → 27` terminology; record each current mismatch as an expected failure.
- [ ] **Run the failing test and state expected failure:** Run `rg -n "Branch 25|EXTERNAL_RATE_LIMITED|aircraft description|caller-provided scopes|scope|application tables|migration count|Stripe-only|TBD|TODO" DESIGN.md BACKEND.md DATABASE.md backend/README.md .github/workflows/ci.yml docs/superpowers/specs/2026-09-11-branch-25-external-rest-api-design.md docs/superpowers/plans/2026-09-11-branch-25-external-rest-api.md`; expect stale broad-scope/old-count/old-aircraft references in product documentation before reconciliation, while interpreting explicitly forbidden-contract statements in the spec/plan as allowed context.
- [ ] **Implement the minimum production change:** Update only the four specified product documents and confirmed CI assertions; document edge rate limiting as deployment dependency, not an application contract; document the masked ephemeral CI pepper, separate application/total/owner counts, exact audit/grant/interface contracts, and all route/privacy semantics; preserve Branch 26 security headers, abuse controls, alerting, device boundary, and cleanup-worker deferrals.
- [ ] **Run the focused test:** Re-run the documentation scan and review both planning documents fully; expect no stale Branch 25 promise, no forbidden application 429 contract, no token response scope claim, no aircraft-description promise, no Stripe-only analytics default, no debug-visible pepper, no vague grant/interface language, and correct counts.
- [ ] **Run the relevant regression set:** From `backend/`, run `cargo fmt --check`, `cargo build --locked`, `cargo clippy --locked --all-targets -- -D warnings`, the focused external/auth/database tests listed above, `cargo test --locked --no-fail-fast`, the existing RustSec reachability guard exactly as CI does (`cargo tree --locked -e normal,build,dev -i rsa` and the same for `sqlx-mysql`, failing if either tree is non-empty), `cargo install cargo-audit --version 0.22.2 --locked`, and `cargo audit` from `backend/` so it reads the existing `backend/.cargo/audit.toml` with the exact narrow `RUSTSEC-2023-0071` ignore. Preserve the comment/behavior that the advisory is not claimed patched and the guard fails if `rsa` or `sqlx-mysql` becomes active. Before backend commands that construct application configuration, extend the existing CI credential-generation step with `external_pepper="$(openssl rand -hex 32)"`, emit only `::add-mask::$external_pepper`, and append `EXTERNAL_API_CREDENTIAL_PEPPER_V1=$external_pepper` to `$GITHUB_ENV`; do not print or persist it as a repository secret. From `frontend/`, run `npm ci`, `npm test`, `npm run lint`, `./node_modules/.bin/next typegen`, `npm run typecheck`, `npm run build`, and `npm audit --audit-level=high`. From the repository root, run `docker compose --profile test config --quiet` and `git diff --check`. No deployment command is included.
- [ ] **Review the diff:** Confirm only intended Branch 25 files changed; inspect the CI queries and permission tests separately for `27` successful/total migration-ledger entries, `35` public tables excluding `_sqlx_migrations`, `36` total public tables including `_sqlx_migrations`, and `36` public tables owned by `x_fly_migrator`; verify no raw-secret artifacts, generated build directories, local `.env`, stale aircraft description, Stripe-only default, vague interface/grant text, or unrelated Branch 26 work.
- [ ] **STOP — ready for user review/staging.**

## Threat-model coverage mapping

| Threat | Primary task/test | Deferred portion |
|---|---|---|
| Database leak | Task 3 `persists_only_credential_hmac_digest`, Task 10 runtime tests | Full database-compromise containment remains an operational concern |
| Credential theft | Tasks 4–5 one-time secret and 15-minute token tests | Integrator endpoint/storage security |
| Token replay | Tasks 3 and 5 expiry/revocation tests | Replay within the 15-minute TTL |
| Credential guessing | Tasks 1 and 3 dummy-HMAC/constant-time tests | Edge resource limiting |
| Suspended/revoked access | Tasks 3 and 10 client-row serialization tests | An already authenticated in-flight request may finish |
| Scope escalation | Task 6 zero-scope/cross-scope tests and Task 3 scope-removal barriers | Authorized API_ADMIN governance errors |
| Auth-header logging | Tasks 5 and 10 tracing/log-capture tests | Reverse-proxy redaction configuration |
| Timing/oracle behavior | Tasks 1, 3, and 10 digest tests | Perfect network timing equalization |
| Public ID enumeration | Tasks 1 and 5 generic invalid-client outcomes | Public ID remains intentionally non-secret |
| Duplicate issuance | Tasks 2–3 partial unique/index and concurrency tests; Task 9 one-request transport-loss test | Lost one-time response still requires manual revoke/replacement |
| Analytics PII exposure | Task 8 SQL projection/DTO and cohort-semantics tests | Future analytics expansion requires a new review |
| Internal UUID disclosure | Tasks 1, 4, 7, 8, and 10 JSON tests | None planned for this slice |
| Pepper compromise | Tasks 1 and 10 config/version tests | Multi-version pepper rotation deferred |
| Token-table growth | Task 10 expiry/non-authorizability tests; Task 11 documentation | Cleanup worker/retention policy deferred |

## Dependency conclusion

The approved design is implementable with the dependencies already declared in
`backend/Cargo.toml`: `rand`, `hmac`, `sha2`, `hex`, and `subtle`. No new Rust or
frontend dependency is required by this plan. If implementation discovers a
dependency is unavoidable, stop at that task, document the exact API need and
security reason, and wait for user approval; do not install it as part of this
plan.

## Database count confirmation

The plan uses the authoritative terminology everywhere:

```text
Application tables:                    33 → 35
Total public tables including ledger:  34 → 36
Public tables owned by x_fly_migrator: 34 → 36
Migration count:                       26 → 27
```

It keeps the application-table, total-public-table, ownership, and migration
labels distinct; `36` is never called the application-table count.

## Implementation handoff

This plan intentionally contains no automatic Git staging or commit steps. The
user controls staging, commits, pushes, merges, branch changes, and deployment.
Each task stops at a review checkpoint so a reviewer can reject or adjust one
security boundary without accepting the remainder.
