# Branch 25 — External REST API Design Specification

**Status:** Approved architecture; implementation not started
**Branch:** `feat/25-external-rest-api`
**Base:** `2f7628099f11c8b4fe8dd11d24cf766df07f6282`

## 1. Goal

Branch 25 adds the first secure machine-facing External REST API for API
clients registered by Branch 24. It connects the existing administrative
registry to one-time credential issuance, short-lived opaque access tokens,
an authenticated external principal, server-side scope checks, and the
smallest useful read-only flight and aggregate-analytics surfaces.

Staff sessions, customer cookies, and external credentials remain separate
security domains. External callers never receive PostgreSQL access and never
use the Next.js staff BFF.

## 2. Existing Branch 24 boundary

Branch 24 owns the administrative identity and authorization configuration:

```text
API_ADMIN effective staff permission
  → api_clients
  → api_scope_catalog
  → api_client_allowed_scopes
  → api_client_management_audit
```

It provides public Client IDs in the `XFC` plus 16-character human-safe
uppercase format, typed `flights:read` and `analytics:read` scopes, ACTIVE /
SUSPENDED /
REVOKED lifecycle states, optimistic versioning, and append-only audit.

Branch 24 does not provide client secrets, credentials, tokens, external
authentication, scope enforcement at data-consumption time, or External REST
API routes. `ACTIVE` means administratively eligible; it does not mean that a
request is authenticated.

The existing staff boundary remains PostgreSQL-backed opaque
`x_fly_staff_session` authentication, Argon2id passwords, current-state RBAC,
and exact Origin/CSRF checks for browser mutations. No external code path may
reuse `AuthenticatedStaff`, a staff session cookie, or staff permission names.

## 3. Approved decisions

- Use Option B: long-lived client credential exchanged for a short-lived
  opaque access token.
- Do not implement JWT, OAuth2 authorization-server compliance, refresh tokens,
  or signed external access tokens.
- Generate a 32-byte client secret and show it exactly once.
- Store only an HMAC-SHA-256 verifier using a deployment-held 32-byte pepper.
- Generate a 256-bit access token and store only its SHA-256 hash.
- Set access-token TTL to 15 minutes (`expiresIn: 900`).
- Check current client status, credential status, token expiry/revocation, and
  relational scopes on every protected request.
- Permit one unrevoked credential per API client; retain revoked history.
- Support revoke-then-replace credential rotation, but not overlapping
  zero-downtime credentials.
- Defer long-lived credential expiry and last-used persistence.
- Use `/api/v1/external/...`; external routes are server-to-server and bypass
  the Next.js BFF.
- Do not grant browser CORS access to the machine-facing external routes.
- Initially expose only public flight reads and aggregate, non-financial
  analytics.

## 4. Authentication architecture

### 4.1 Credential and crypto format

The admin issuance response contains a public Client ID and a separate
64-character lowercase hexadecimal `clientSecret`. The secret represents 32
cryptographically random bytes and is never reconstructed from the database.

The token exchange request is:

```http
POST /api/v1/external/token
Content-Type: application/json

{"clientId":"XFC…","clientSecret":"<64 lowercase hex characters>"}
```

The request body uses strict deserialization (`deny_unknown_fields`) and both
fields are required strings. The adapter validates the canonical public Client
ID representation and the 64-character lowercase-hex secret representation
before entering credential authentication. Malformed JSON, missing fields,
wrong JSON types, unknown fields (including `scope`/`scopes`), non-canonical
field values, or values over the approved structural bounds are request
contract failures and return `400 EXTERNAL_REQUEST_INVALID`; they are not
authentication attempts. Only a request that passes those representation
checks reaches credential authentication. A syntactically valid but unknown
Client ID, wrong secret, suspended/revoked client, revoked credential, or other
credential-state failure returns the same generic
`401 EXTERNAL_CLIENT_AUTHENTICATION_FAILED` response.

The application computes:

```text
HMAC-SHA-256(
  EXTERNAL_API_CREDENTIAL_PEPPER_V1,
  "x-fly-client-credential-v1\0" || public_client_id || "\0" || secret_bytes
)
```

The digest is fixed at 32 bytes. Verification compares the supplied digest
with the stored digest using a constant-time fixed-length comparison. A
representation-valid but unknown Client ID still performs equivalent HMAC work
against a fixed dummy digest where practical, then returns the same public
failure response. Non-canonical Client IDs and malformed secret
representations are rejected as request-contract failures before this
authentication path. The exchange service computes the candidate digest and a
dummy digest before the repository lookup/transaction; the repository compares
the candidate against the stored digest under the locked-client transaction
when a live credential exists, and the unknown-client path discards both
digests after the dummy work without inserting a token.

The domain/application boundary uses distinct secret-bearing value types:

```rust
ExternalApiCredentialPepper([u8; 32])
PlaintextClientSecret([u8; 32])
PlaintextAccessToken([u8; 32])
CredentialDigest([u8; 32])
AccessTokenHash([u8; 32])
```

None of these types implements `Debug`, `Display`, or `Serialize`. They expose
bytes only through narrow internal accessors used by the crypto and repository
boundaries. The HTTP adapter performs the only explicit conversion of a
plaintext secret or token to its one-time response string; no domain value is
generically serializable.

`EXTERNAL_API_CREDENTIAL_PEPPER_V1` is required whenever the normal backend
configuration constructs the external-auth capability; the normal DEV, TEST,
and production application processes all construct it, so none of those
runtime environments may start without the variable. It is exactly 32 decoded
random bytes represented as 64 hexadecimal characters, is not stored in
PostgreSQL, and must be identical across API instances. Missing or malformed
configuration causes startup/configuration failure. Isolated unit tests may
inject an explicit fixed byte array into the crypto constructor, but no
application/test configuration path may silently default or bypass the
validation.

The pepper is parsed into `ExternalApiCredentialPepper`, never retained as a
raw `String` in a generally printable configuration field, and never included
in logs or error formatting. Because the current `AppConfig` derives
`Debug`, Branch 25 replaces that automatic implementation with a manual
redacted implementation (or an equivalent redacted configuration wrapper)
that renders every secret-bearing field, including the pepper, only as
`<redacted>`; this includes the database URL, existing signing/payment
secrets, and the new pepper. Tests format configuration, crypto, and
repository-adjacent structures with sentinel values and assert that the
sentinel pepper, client secret, access token, credential digest, and token hash
cannot appear.

### 4.2 Access token format

The exchange response is exactly the following conceptual shape:

```json
{
  "accessToken": "xfa_v1_<64 lowercase hex characters>",
  "tokenType": "Bearer",
  "expiresIn": 900
}
```

It contains no scope snapshot. Scopes are current relational authorization
state and are loaded during each protected request.

The plaintext access token is 32 cryptographically random bytes formatted for
the bearer response. PostgreSQL stores only `SHA-256(token_bytes)`. There is
no refresh token. A caller obtains another token by exchanging the long-lived
client credential again while the client is ACTIVE.

## 5. Credential lifecycle

### 5.1 Issue

An API_ADMIN-authorized admin submits the current API-client version to issue a
credential. The service generates the secret before persistence, calculates
the digest, and asks the repository to atomically lock and validate the
client, insert the credential, append `CREDENTIAL_ISSUED`, increment the
client version, materialize safe metadata inside the transaction, and commit.

Issuance is allowed for ACTIVE and SUSPENDED clients so an integration can be
prepared before activation. It is forbidden for REVOKED clients and when an
unrevoked credential already exists.

The HTTP adapter explicitly maps the committed application result to a
one-time response containing the plaintext secret. It never re-reads a secret
after commit. If the browser loses the response after commit, the UI shows an
indeterminate/failure state and instructs the operator to refresh. Refresh may
show only safe metadata that a live credential exists; it never reconstructs
the secret. Recovery is revoke the live credential, then issue a replacement;
the browser must not retry issuance automatically.

### 5.2 Revoke and replace

Credential revocation requires API_ADMIN manage permission and the current
client version. The admin HTTP request contains only that version; the route
assigns the system-owned `ADMIN_REQUEST` reason. Suspension, terminal client
revocation, and replacement flows assign `CLIENT_SUSPENDED`, `CLIENT_REVOKED`,
and `REPLACED` internally and never accept those reasons from a browser. The
transaction locks the client, marks the live credential revoked, revokes its
live access tokens, appends `CREDENTIAL_REVOKED`, increments the client
version, and commits. A replacement may be issued only after that credential
is revoked and the client remains ACTIVE or SUSPENDED.

A client may therefore have multiple historical credential rows but never
more than one unrevoked row. Overlapping credentials and scheduled,
zero-downtime rotation are deferred.

Credential issuance and every invalidating lifecycle mutation acquire the
`api_clients` row lock first. Their deterministic race outcomes are:

- Issue credential versus SUSPEND: either operation may linearize first. The
  result is one stored credential, final client state SUSPENDED, and no usable
  access token; a credential issued just before suspension is invalidated by
  the suspension transaction.
- Issue credential versus terminal REVOKE: if REVOKE commits first, issuance
  fails against the terminal state. If issuance commits first, REVOKE marks
  that credential revoked and revokes its live tokens in the same transaction.
  A committed REVOKED client therefore has neither a live credential nor a
  live token.
- Issue credential versus credential REVOKE: if the existing revocation
  commits first, replacement issuance may subsequently succeed; while another
  credential remains live, the client-row lock and partial unique index reject
  a second live credential.

Concurrency tests use barriers/transaction coordination rather than sleeps.

### 5.3 Client state

| Client state | Credential exchange | Existing tokens | Protected routes |
|---|---|---|---|
| ACTIVE | Allowed with live credential | Allowed until expiry/revocation | Allowed by scope |
| SUSPENDED | Denied | Revoked atomically by suspension | Denied |
| REVOKED | Denied permanently | Revoked atomically | Denied permanently |

Reactivation from SUSPENDED does not revive old tokens; a new exchange is
required. REVOKED is terminal and cannot issue a new credential.

Long-lived credential expiry and `last_used_at` are deliberately absent from
Branch 25.

## 6. Access-token lifecycle

Every successful exchange inserts a new access-token row with a 15-minute
absolute expiry. Multiple unexpired tokens may exist for one credential; token
issuance is not idempotent because a repeated exchange is a normal renewal
operation.

Protected requests use:

```http
Authorization: Bearer xfa_v1_<64 lowercase hex characters>
```

Authentication hashes the presented token, performs one indexed join across
the token, credential, client, and current relational scopes, and rejects any
expired/revoked token, revoked credential, SUSPENDED client, or REVOKED client.

Scope loading must not be an inner join that removes an otherwise valid token
when no scope rows are present. The query uses a `LEFT JOIN` plus typed scope
aggregation (or an equivalent two-step read) so a valid ACTIVE client can
produce an `ExternalPrincipal` with an empty scope set. The route guard then
returns `403 EXTERNAL_SCOPE_DENIED`; it is not reported as invalid
credentials. The normal ACTIVE-client invariant still requires at least one
allowed scope, so this is fail-closed handling for an unexpected zero-scope
state.

Expired rows are retained as history in this branch. A cleanup worker and
retention policy are deferred because they are operational hardening, not an
authorization prerequisite for the first academic/demo release.

## 7. Transaction/concurrency semantics

Token issuance has an explicit linearization point. The repository performs:

```text
BEGIN
  SELECT api_clients ... FOR UPDATE
  SELECT the live credential for that client under the same lock
  constant-time compare the supplied credential digest
  require client ACTIVE and credential unrevoked
  INSERT external_access_tokens(token_hash, expires_at, ...)
COMMIT
```

Client suspension, client revocation, and credential revocation/replacement
use the same API-client-row serialization boundary. Consequently, a token
cannot be inserted after a suspend/revoke transaction has committed. If token
issuance acquires the row lock first, its insert may commit before the later
lifecycle mutation, which then revokes it.

A request that authenticated immediately before a later suspension or
revocation commit may finish normally. Database locks are not held for the
entire HTTP handler or endpoint query.

Scope removal follows the same request linearization philosophy. If removal
commits before the authentication/scope-loading query, that request observes
the current scope set and receives `403 EXTERNAL_SCOPE_DENIED`. If the request
has authenticated and passed its scope guard before the removal commit, the
in-flight request may finish. No database lock is held across the endpoint
handler. A barrier-based test proves both orderings.

Credential issuance also locks the client row, checks optimistic `version`,
and relies on a partial unique index as a second line of defense against
double issuance. Mutation responses are materialized before commit to avoid
false failures and duplicate retries.

## 8. ExternalPrincipal

Authentication produces:

```rust
ExternalPrincipal {
    api_client_id: Uuid,                 // in-process only
    client_id: String,                   // public-safe identifier
    scopes: BTreeSet<ApiClientScope>,     // current relational scopes
}
```

The internal UUID is usable by application/repository code for correlation but
is never serialized, placed in URLs, or exposed in external errors. The
principal itself is never serialized directly; endpoint adapters construct
dedicated response DTOs.

## 9. Scope authorization

The external router has three explicit layers:

1. Bearer authentication for every protected route.
2. Typed route-scope enforcement (`flights:read` or `analytics:read`).
3. Dedicated minimal response DTOs.

The token request accepts exactly `clientId` and `clientSecret`. Unknown
fields, including `scope` and `scopes`, are rejected as
`400 EXTERNAL_REQUEST_INVALID`; no caller-supplied scope can influence
authorization. Token rows do not store scopes. Current
`api_client_allowed_scopes` assignments are the only authorization source.
Scope removal therefore applies to the next protected request without waiting
for token expiry, subject to the in-flight request rule in Section 7.

## 10. Route contract

Approved routes:

```text
POST /api/v1/external/token
GET  /api/v1/external/flights
GET  /api/v1/external/flights/{flightPublicId}
GET  /api/v1/external/analytics/summary
```

The token route accepts client credentials and returns an opaque token. The
other routes require the bearer header. None accepts a staff cookie or
customer cookie. None is forwarded through Next.js.

The conceptual router hierarchy is:

```text
/api/v1
├── customer routes (existing public/customer boundary)
├── admin routes (existing staff-session + RBAC boundary)
└── external
    ├── POST /token (client-credential exchange, no bearer principal yet)
    └── protected subrouter
        ├── GET /flights
        ├── GET /flights/{flightPublicId}
        └── GET /analytics/summary
            └── ExternalBearerAuthLayer → ExternalPrincipal extension
                → typed route-scope guard → endpoint DTO handler
```

The external subrouter is mounted independently of the customer and admin
subrouters. Its bearer layer runs before endpoint handlers, and its scope
guards run after principal extraction but before data queries.

The current `build_router` applies one credentialed `CorsLayer` after merging
health, public/customer, and admin routes. Branch 25 changes only that
composition: construct a browser router containing every existing
non-external route (health, public/customer, and admin) and apply the existing
`CorsLayer` to that router; construct the external router separately without
any browser CORS layer; then merge the two routers and apply the existing
request-tracing layer outside both. The parent merge must not receive the
browser CORS layer, and Branch 25 does not introduce a second CORS subsystem.

An external request carrying `Origin: https://example.test` must not receive
`Access-Control-Allow-Origin`, `Access-Control-Allow-Credentials`,
`Access-Control-Allow-Headers`, or `Access-Control-Allow-Methods` from either
the token endpoint or a protected external route. Existing customer/admin CORS
tests continue to prove their approved browser behavior. Safe request tracing
continues to apply, but never records Authorization, request bodies, query
strings, or cookies.

## 11. Flight DTO contract

`GET /external/flights` accepts the existing bounded public-search inputs:

```text
origin:       three uppercase airport characters
destination:  three uppercase airport characters, different from origin
departure:    YYYY-MM-DD
cabin:        customer-bookable cabin accepted by the current public query
```

`GET /external/flights/{flightPublicId}` accepts the public flight identifier,
departure date, and cabin. It returns a dedicated external DTO containing only:

- public flight identifier;
- flight number;
- origin and destination codes;
- public schedule values;
- arrival-day offset and duration;
- public status and `aircraftCode` from the existing flight-service field;
- public Business/First prices in THB where available.

It excludes internal UUIDs, management versions, audit state, staff fields,
seat-hold state, capacity configuration, and inventory ownership. Existing
Business/First booking rules remain unchanged; Branch 25 has no booking route.
The existing public flight repository behavior is reused behind this DTO and
never exposed as a persistence model.

## 12. Analytics privacy/metric semantics

`GET /external/analytics/summary` uses a dedicated aggregate query. It never
loads `DashboardReport` and filters fields afterward.

The response contains only:

```text
period.from
period.to
generatedAt
totalBookings
ticketsIssued
cancelledBookings
bookedSeats
sellableSeats
occupancyPercent
```

Semantics:

- Dates are inclusive calendar dates in `Asia/Bangkok`, matching the current
  dashboard timezone.
- Both date parameters are optional: `to` defaults to today in Bangkok and
  `from` defaults to 29 days before `to`; the resulting inclusive range must
  satisfy `from <= to` and remain at most 366 days.
- Optional `route` is the validated `AAA-BBB` public route filter.
- The cohort basis is authoritative flight departure, never payment-success
  time or booking-created time. For the current schema, the query derives the
  departure instant from `flight_instances.departure_date` plus
  `flight_services.departure_time`, interpreted in the service's approved
  `origin_time_zone` (falling back to the existing airport timezone exactly as
  current flight code does), and converts that instant to `Asia/Bangkok`.
  An instance belongs to the cohort when that converted local calendar date is
  within the inclusive `[from, to]` range. Rows without a complete
  authoritative departure instant are not silently assigned a date.
- Optional `route` filters the flight-instance cohort, while the `cabin`
  selection filters all booking, seat, and inventory aggregates consistently.
  A specified cabin
  selects `BUSINESS` or `FIRST`; when omitted, the selected cabin set is
  exactly current `BUSINESS` plus `FIRST` for every metric. Legacy `ECONOMY`
  and `PREMIUM_ECONOMY` are excluded from current sellable analytics and are
  not silently folded into an unfiltered summary.
- All supported successful booking/payment paths are included. In the current
  schema that means successful `STRIPE` and `MOCK_BITCOIN` attempts; the query
  filters only `status = 'SUCCEEDED'`, has no provider filter, and must not
  inherit the executive dashboard's Stripe-only default. Provider identifiers
  and references are never returned.
- In the current relational model, a booking record is the single successful
  `payment_attempts` row for a `seat_holds` row, joined through that hold to a
  cohort `flight_instance`; the existing one-success-per-hold constraint is
  the de-duplication rule. The authoritative current cancellation state is
  the related `tickets.status`/cancellation record, not payment-provider
  metadata.
- `totalBookings` counts all booking records belonging to the selected
  flight/date/route/cabin cohort, including bookings later cancelled.
- `cancelledBookings` counts the subset whose authoritative current booking
  state is cancelled (`tickets.status = 'CANCELLED'`), expressed as a
  distinct-booking filtered count rather than a payment-provider event count.
- `ticketsIssued` counts ticket records with an issuance record (`tickets.issued_at`)
  belonging to cohort bookings, regardless of their current cancellation
  status; this is a historical issuance metric, not a currently-valid-ticket
  metric.
- `bookedSeats` counts currently booked, non-cancelled occupied seats in the
  filtered cohort.
- `sellableSeats` counts configured sellable capacity for active Business and
  First inventory represented by the same filtered flight-instance cohort.
- `occupancyPercent` is `bookedSeats / sellableSeats * 100` when
  `sellableSeats > 0`, rounded to two decimals, and is `0.0` when
  `sellableSeats == 0`.

Task 8 implements this with separate aggregate CTEs (or equivalent
subqueries) rooted in a `flight_cohort` CTE that inner-joins the origin
airport, requires a non-null `departure_time`, converts the local
`departure_date + departure_time` using the approved origin zone, and applies
the public route filter to `origin_code`/`destination_code`. The optional
cabin filter is applied as the normalized cabin on `seat_holds.cabin` for
bookings and on `flight_seats.cabin` for both seat and inventory aggregates;
when absent, every aggregate applies the `BUSINESS`/`FIRST` set.
One booking CTE joins `payment_attempts` with `status = 'SUCCEEDED'` through
`seat_holds` to the cohort and derives `COUNT(DISTINCT payment_attempts.id)`
for total and cancellation counts plus
`COUNT(DISTINCT tickets.id) FILTER (WHERE tickets.issued_at IS NOT NULL)` for
issued tickets; one seat CTE
joins `payment_attempt_seats` to `flight_seats` and counts only rows with
`released_at IS NULL`, `booking_status = 'BOOKED'`, and a related ticket whose
`ticket.status IS DISTINCT FROM 'CANCELLED'`; and one inventory CTE counts
`flight_seats.sellable = TRUE` for current `BUSINESS`/`FIRST` seats on
scheduled (`flight_services.status = 'SCHEDULED'`) instances. The final query
left-joins those aggregates to avoid row multiplication and returns one safe
summary row. It selects no provider references, payment secrets, contact
fields, PII, or `DashboardReport` fields.

The endpoint never returns passenger/contact PII, booking references, ticket
numbers, payment data, revenue, refunds, pending-refund state, attention
queues, per-booking/per-passenger/per-flight operational rows, staff data,
audit history, or internal UUIDs. Revenue analytics require a separate policy
decision and are outside this contract.

## 13. Error contract

External errors use:

```json
{
  "error": {
    "code": "EXTERNAL_REQUEST_INVALID",
    "message": "The external request is invalid.",
    "requestId": "server-generated UUID"
  }
}
```

Public classes:

| Condition | Status | Code |
|---|---:|---|
| Missing/malformed/invalid/expired bearer credential | 401 | `EXTERNAL_AUTHENTICATION_FAILED` |
| Suspended/revoked client or credential | 401 | `EXTERNAL_AUTHENTICATION_FAILED` |
| Structurally invalid token-exchange JSON/body or field representation | 400 | `EXTERNAL_REQUEST_INVALID` |
| Well-formed token-exchange credential that fails authentication | 401 | `EXTERNAL_CLIENT_AUTHENTICATION_FAILED` |
| Missing required scope | 403 | `EXTERNAL_SCOPE_DENIED` |
| Invalid query/body | 400 | `EXTERNAL_REQUEST_INVALID` |
| Valid request but no public resource | 404 | `EXTERNAL_RESOURCE_NOT_FOUND` |
| Authentication storage unavailable | 503 | `EXTERNAL_AUTH_UNAVAILABLE` |
| Flight/analytics dependency unavailable | 503 | `EXTERNAL_SERVICE_UNAVAILABLE` |
| Unexpected internal failure | 500 | `EXTERNAL_INTERNAL_ERROR` |

Unknown but well-formed client, wrong secret, suspended state, revoked state,
and revoked credential never receive distinct public messages. Missing fields,
malformed JSON, wrong types, unknown fields, non-canonical field
representations, and oversized request values are instead the 400 contract
failure above and never enter credential authentication. Internal diagnostics
may classify only the 401 authentication outcomes. Bearer-protected 401
responses include a safe
`WWW-Authenticate: Bearer` challenge. There is no `429 EXTERNAL_RATE_LIMITED`
application contract in Branch 25; token-endpoint abuse limiting is a
documented Cloudflare/Nginx or equivalent go-live edge dependency.

Token and credential-bearing responses use `Cache-Control: no-store, private`;
the token response additionally uses `Pragma: no-cache`.

## 14. Database model

One append-only migration is planned:

```text
20260911000100_create_external_api_auth.sql
```

### `api_client_credentials`

- `id UUID PRIMARY KEY`, internal only.
- `api_client_id UUID NOT NULL REFERENCES api_clients(id) ON DELETE RESTRICT`.
- `secret_digest BYTEA NOT NULL`, exactly 32 bytes.
- `digest_version SMALLINT NOT NULL CHECK (digest_version = 1)`, currently
  constrained to `1` for the `EXTERNAL_API_CREDENTIAL_PEPPER_V1` verifier.
- `issued_at TIMESTAMPTZ NOT NULL`.
- `issued_by_staff_user_id UUID NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT`.
- `revoked_at TIMESTAMPTZ NULL`.
- `revoked_by_staff_user_id UUID NULL REFERENCES staff_users(id) ON DELETE RESTRICT`.
- `revocation_reason TEXT NULL CHECK (revocation_reason IS NULL OR
  revocation_reason IN ('ADMIN_REQUEST', 'CLIENT_SUSPENDED',
  'CLIENT_REVOKED', 'REPLACED'))`.
- Partial unique index `api_client_credentials_one_live_idx` on
  `(api_client_id) WHERE revoked_at IS NULL`, enforcing one unrevoked
  credential per client.

### `external_access_tokens`

- `id UUID PRIMARY KEY`, internal only.
- `api_client_credential_id UUID NOT NULL REFERENCES api_client_credentials(id) ON DELETE RESTRICT`.
- `token_hash BYTEA NOT NULL`, exactly 32 bytes, with unique constraint/index
  `external_access_tokens_token_hash_key`.
- `issued_at TIMESTAMPTZ NOT NULL`.
- `expires_at TIMESTAMPTZ NOT NULL`.
- `revoked_at TIMESTAMPTZ NULL`.
- Constraint requiring `expires_at > issued_at`.
- Partial index `external_access_tokens_credential_live_idx` on
  `(api_client_credential_id) WHERE revoked_at IS NULL`, supporting
  credential-wide revocation of live tokens.

`secret_digest` and `token_hash` are sensitive verifier material. Neither may
enter admin/external DTOs, SQL error text, logs, URLs, or test snapshots.

### Existing `api_client_management_audit` compatibility

The same migration alters, rather than replaces, the Branch 24 audit table:

- Add nullable `credential_id UUID NULL REFERENCES api_client_credentials(id)
  ON DELETE RESTRICT` with the implicit NULL default, so every historical
  Branch 24 row remains valid.
- Add `api_client_management_audit_credential_idx` on
  `(credential_id, created_at DESC, id DESC)` with `WHERE credential_id IS NOT
  NULL` for credential history lookups.
- `ALTER TABLE api_client_management_audit DROP CONSTRAINT
  api_client_management_audit_action_check`, then add a replacement whose
  allowed values are exactly the six Branch 24 actions
  (`CLIENT_CREATED`, `CLIENT_METADATA_UPDATED`, `CLIENT_SCOPES_UPDATED`,
  `CLIENT_ACTIVATED`, `CLIENT_SUSPENDED`, and `CLIENT_REVOKED`) plus exactly
  `CREDENTIAL_ISSUED` and `CREDENTIAL_REVOKED`. The replacement is additive
  and never narrows the historical action set.
- Add `api_client_management_audit_credential_context_check` requiring
  `credential_id IS NOT NULL` for the two credential actions and
  `credential_id IS NULL` for every existing client-management action.

Credential audit events retain the existing `before_state` and `after_state`
`ApiClientAuditSnapshot` JSON shape so the Branch 24 repository and admin audit
DTO continue to parse every action. The new `credential_id` is correlation
only: it is not selected into the ordinary browser/admin audit DTO. Dedicated
safe credential metadata (status, issued/revoked timestamps, and whether a
live credential exists) is exposed separately, without the credential UUID,
digest, token hash, or plaintext secret. `CREDENTIAL_ISSUED` and
`CREDENTIAL_REVOKED` always carry the correlated internal credential row while
their client snapshot remains the current client snapshot.
The Rust `ApiClientAuditAction` enum and parser are extended with matching
`CredentialIssued` and `CredentialRevoked` variants; all six Branch 24 enum
values, their `as_str` values, and their existing snapshot mapping remain
unchanged. The frontend presentation map gives the two new actions explicit
safe labels. Any runtime action string outside the known union maps to a
generic safe “management event” label and renders no raw action, payload, or
credential metadata.

The migration creates the two auth tables before adding the audit foreign key,
so the new reference is valid in one append-only migration. Test cleanup must
delete audit rows first, then access tokens and credentials, then scopes and
client fixtures; production deletion remains prohibited by `RESTRICT` and
runtime privileges.

## 15. Ownership/grants

Both new application tables are owned by `x_fly_migrator`. `x_fly_runtime`
owns no application object and receives no migration-ledger mutation rights.

Because staff API-client administration and external runtime authentication
execute through the normal Axum connection, runtime grants must support the
actual operations while protecting immutable fields:

The provisioning script first revokes all table privileges on the two new
tables from `x_fly_runtime` and `PUBLIC`, then grants only the column lists
below. The `id` columns retain their `gen_random_uuid()` defaults and are
omitted from every application `INSERT`; the two new tables must also be
removed from any existing table-level SELECT/INSERT/UPDATE grant lists before
the column-level grants are added.

- `api_client_credentials`: `SELECT` only on the columns needed for
  verification, safe metadata, and mutation predicates/`RETURNING`:
  `id`, `api_client_id`, `secret_digest`, `digest_version`, `issued_at`,
  `revoked_at`, and `revocation_reason`; `INSERT` only on the immutable
  issuance columns (`api_client_id`, `secret_digest`, `digest_version`,
  `issued_at`, and `issued_by_staff_user_id`); and column-level `UPDATE` only
  on `revoked_at`, `revoked_by_staff_user_id`, and `revocation_reason`.
  `issued_by_staff_user_id` and `revoked_by_staff_user_id` are write-only
  attribution fields for the runtime role and are never selected or returned
  by application SQL.
- `external_access_tokens`: `SELECT` on `id`, `api_client_credential_id`,
  `token_hash`, `issued_at`, `expires_at`, and `revoked_at`; `INSERT` only on
  `api_client_credential_id`, `token_hash`, `issued_at`, and `expires_at`; and
  column-level `UPDATE` only on `revoked_at`.
- `api_client_management_audit`: existing `SELECT` plus `INSERT` on
  `api_client_id`, `actor_staff_user_id`, `action`, `before_state`,
  `after_state`, nullable `credential_id`, and `created_at`; `id` remains a
  database default. The existing insert helper currently supplies
  `clock_timestamp()` for `created_at`, so that column is explicitly granted.
  Existing six actions remain valid and credential actions must supply a
  credential correlation. Existing Branch 24 `insert_audit` writes insert
  exactly `(api_client_id, actor_staff_user_id, action, before_state,
  after_state, created_at)`; the nullable `credential_id` is omitted and
  defaults to `NULL`. New `CREDENTIAL_ISSUED` and `CREDENTIAL_REVOKED` writes
  insert exactly that same set plus `credential_id`.
- Table-level and column-level `INSERT` privileges are cumulative. Therefore
  provisioning executes this explicit broad-privilege removal first:

  ```sql
  REVOKE ALL PRIVILEGES ON TABLE public.api_client_management_audit
      FROM x_fly_runtime, PUBLIC;
  REVOKE INSERT ON TABLE public.api_client_management_audit
      FROM x_fly_runtime, PUBLIC;
  ```

  It then removes the audit table from the existing table-level `GRANT
  INSERT` list, restores only the approved table-level `SELECT` for
  `x_fly_runtime`, and grants
  `INSERT (api_client_id, actor_staff_user_id, action, before_state,
  after_state, credential_id, created_at)` to `x_fly_runtime`. No audit write
  privilege is granted to `PUBLIC`; generated `id` remains protected. This
  single column grant is the union needed by both the six historical Branch 24
  actions and the two correlated credential actions.
- Because PostgreSQL checks columns referenced by `WHERE`/`RETURNING`, the
  listed `SELECT` columns are intentional prerequisites for the revocation and
  authentication statements. Application SQL must use explicit projections,
  never `SELECT *`, so the write-only attribution fields do not require a
  broader read grant. Table-level `INSERT` is not granted where a column list
  can express the exact issuance shape.
- No runtime DELETE or TRUNCATE of credential/token history.
- No runtime UPDATE of `id`, `api_client_id`, `secret_digest`,
  `digest_version`, `issued_at`, `issued_by_staff_user_id`,
  `api_client_credential_id`, `token_hash`, or `expires_at`.
- No runtime `ALTER`, ownership change, schema CREATE, or migration-ledger
  write.

Tests must connect separately as `x_fly_runtime` and prove each prohibited
operation fails. They must assert that `has_table_privilege` for `INSERT` on
`api_client_management_audit` is false, that `has_column_privilege` is true
only for the seven approved columns and false for generated `id`, and that
the ACL contains no `PUBLIC` write grant. They must also prove the existing
Branch 24 insert shape and both correlated credential-event insert shapes
succeed under `x_fly_runtime`, while audit `UPDATE`, `DELETE`, `TRUNCATE`,
`ALTER`, and ownership changes fail.

## 16. Admin issuance/revocation UX

The existing API Client detail page gains only:

- safe credential status/metadata (`credentialMetadata` on the existing detail
  response, containing only `hasLiveCredential`, `issuedAt`, and `revokedAt`;
  timestamps describe the newest credential row by `issued_at`, whether live
  or most recently revoked);
- Issue Credential action;
- one-time secret modal with copy control;
- explicit “cannot be recovered” acknowledgement;
- Revoke Credential action;
- replacement issuance after revocation.

The secret exists in React memory only. It is cleared on modal dismissal and
never placed in localStorage, sessionStorage, a URL, server-component
persistent props, analytics, console output, or a recover endpoint. The BFF
forwards the response with no-store headers and no secret-bearing logging.

If the database commit succeeds but the browser loses the response, the UI must
send no second issuance request. It shows an indeterminate/failure state,
allows refresh to reveal only safe metadata, and uses the recovery sequence
revoke then issue replacement.

## 17. Secret-handling rules

- Use OS-backed cryptographic randomness for client secrets and access tokens.
- Never persist plaintext client secrets or access tokens.
- Never log plaintext secrets, tokens, Authorization headers, request bodies,
  credentials, payment secrets, or PII.
- Never include secret material in errors, `Debug`, `Display`, `Serialize`,
  URLs, or generic telemetry.
- Plaintext domain values are non-observable types by default. Only the HTTP
  adapter constructs the one-time issuance JSON response.
- HMAC and token digest comparisons use fixed-length constant-time comparison.
- No secret recovery endpoint exists.

## 18. Audit/observability

Reuse `api_client_management_audit` with only the additions required for
`CREDENTIAL_ISSUED` and `CREDENTIAL_REVOKED`, plus safe internal credential
correlation. Audit rows contain actor, action, timestamp, and safe state; they
never contain secrets, token hashes, Authorization, scopes supplied by a
caller, IP, or user agent.

External authentication diagnostics may record only safe categories such as:

- request ID from the existing server-generated tracing span;
- validated public Client ID when safe and syntactically valid;
- authentication outcome category (`Missing`, `Malformed`, `UnknownClient`,
  `SecretMismatch`, `Expired`, `Suspended`, `Revoked`, or `ScopeDenied`);
- required scope and scope decision;
- latency/status already emitted by safe HTTP tracing.

They must not record the bearer value, client secret, Authorization header,
request body, query string, or raw database error. Authentication failures are
generic externally while internal logs remain category-based and redacted.

## 19. Threat model

| Threat | Branch 25 mitigation | Residual/deferred risk |
|---|---|---|
| Database leak | HMAC credential verifier; SHA-256 token hashes; no plaintext | Database compromise still exposes stored business data |
| Credential theft | Secret used only at exchange; revoke/replace | Integrator endpoint/storage compromise |
| Token replay | 256-bit opaque tokens, TLS, 15-minute TTL | Replay remains possible within TTL |
| Credential guessing | 256-bit secret, generic failure, constant-time digest comparison | Edge resource exhaustion needs deployment limiting |
| Suspended/revoked access | Current state checked every request; live tokens revoked | An already authenticated in-flight request may finish |
| Scope escalation | Typed relational scopes; token request has no scope field | Authorized API_ADMIN misconfiguration |
| Auth-header logging | Existing tracing excludes headers; regression tests | Reverse proxy must also redact Authorization |
| Timing/oracle behavior | Dummy HMAC work and fixed-length constant-time compare | Network/DB timing is not perfectly identical |
| Public ID enumeration | 80-bit random public ID; generic failures | Public ID is intentionally non-secret |
| Duplicate issuance | Client row lock, version check, partial unique index | Lost one-time response requires manual recovery |
| Analytics PII exposure | Dedicated aggregate query and minimal DTO | Future analytics expansion needs new review |
| Internal UUID disclosure | Separate DTOs; principal never serialized | Regression covered by contract tests |
| Pepper compromise | Deployment-only pepper and version marker | Pepper rotation/multi-version support deferred |
| Token-table growth | Expiry makes rows unusable; no auth bypass | Retention/cleanup worker deferred |

## 20. Test strategy

The implementation plan uses TDD for each boundary.

Domain tests cover secret/token format, strict lengths, state transitions,
scope checks, and non-observable secret types.

Repository tests cover HMAC-digest persistence, token-hash persistence,
ACTIVE/SUSPENDED/REVOKED behavior, expiry, one-live-credential uniqueness,
transactional lifecycle revocation, and pre-commit response materialization.

HTTP tests cover missing/malformed/invalid credentials, valid exchange and
bearer authentication, suspension/revocation, expiry, insufficient/correct
scope, DTO minimization, stable errors, `WWW-Authenticate`, and no-store.

Endpoint tests cover positive/negative `flights:read`, positive/negative
`analytics:read`, cross-scope denial, bounded filters, and missing resources.

Security tests prove secrets/tokens never appear in logs or responses after
issuance, internal UUIDs are absent, plaintext/digest confusion fails, and
runtime PostgreSQL privileges reject prohibited mutations.

Concurrency tests cover simultaneous issuance, credential issuance versus
suspend/revoke/replacement, token exchange versus suspend/revoke, and scope
removal versus protected-request authentication using barriers and the same
database row-serialization boundary. They also cover an empty authenticated
scope set becoming 403 rather than 401.

All database-backed tests use the guarded TEST-only setup and runtime roles.
They preserve the cross-process-safe fixture allocator and never use DEV.

## 21. Deployment dependencies

- Configure `EXTERNAL_API_CREDENTIAL_PEPPER_V1` as 64 hexadecimal characters
  representing 32 random bytes on every API instance.
- CI extends the existing ephemeral-project-credentials step with
  `external_pepper="$(openssl rand -hex 32)"`, emits only
  `::add-mask::$external_pepper`, and appends
  `EXTERNAL_API_CREDENTIAL_PEPPER_V1=$external_pepper` to `$GITHUB_ENV` before
  backend commands that construct `AppConfig`; the value is never printed,
  committed, or stored as a repository secret. Local TEST uses a private
  ignored environment value, while DEV and production configuration remain
  operator-managed.
- Run the migration as `x_fly_migrator`; normal startup remains readiness-only
  through `x_fly_runtime`.
- Apply reviewed runtime grants after migration.
- Terminate TLS before accepting client credentials or bearer tokens.
- Configure Cloudflare/Nginx/equivalent edge limits for the token endpoint and
  external namespace before public exposure. This is a go-live dependency,
  not an application-level Branch 25 `429` contract.
- Configure the edge/proxy to omit Authorization, credential bodies, and
  sensitive query values from access logs.
- Keep PostgreSQL private; external clients connect only to HTTPS API routes.

## 22. Explicit deferred scope

Branch 25 does not include:

- JWT or OAuth2 authorization-server compliance;
- refresh tokens;
- simultaneous live credentials or overlapping rotation;
- long-lived credential expiry;
- last-used persistence, usage billing, or accounting;
- browser third-party consumption or external booking APIs;
- passengers, tickets, baggage, payments, refunds, or campaign endpoints;
- revenue analytics;
- a broad security-event warehouse or alerting platform;
- an application-level distributed rate limiter;
- broad security-header work;
- company-device enforcement;
- a repository-wide Clean Architecture refactor;
- a token cleanup worker;
- deployment or production exposure.

Credential/token endpoint edge limiting remains a deployment dependency.
Branch 26 owns broader abuse controls, security events, headers, and
company-device hardening.

## 23. Documentation impact

After implementation, reconcile:

- `backend/README.md` for pepper configuration, token exchange, route examples,
  secret handling, and edge limiting;
- `BACKEND.md` for the separate staff/external authentication architecture;
- `DATABASE.md` for sensitive verifier fields, ownership, grants, and TEST
  inspection rules;
- `DESIGN.md` for the actual Branch 25 scope and explicit Branch 26 boundary.

The existing Branch 25 section in `DESIGN.md` is broader than this approved
slice and currently names bookings, passengers, tickets, baggage, rate
limiting, expiry, and last-used behavior. It must be reconciled rather than
treated as an implementation requirement.

## 24. Table/migration count terminology

The authoritative pre-25 baseline is:

```text
33 application tables
 + 1 _sqlx_migrations ledger
 = 34 total public tables
```

Branch 25 adds exactly two application tables:

```text
Application tables:                   33 → 35
Total public tables including ledger: 34 → 36
Public tables owned by x_fly_migrator: 34 → 36
Migration count:                      26 → 27
```

No plan, CI assertion, permission test, or document may collapse the
application-table count and total-public-table count into one label; keep the
three table counts and the migration count above distinct. The ownership count
is a separate verification of the same 36 public tables, not an application
table count.
