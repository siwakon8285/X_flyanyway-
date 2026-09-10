# API Client Management Implementation Plan

> **For agentic workers:** Execute inline with `superpowers:executing-plans`; use test-driven development for every behavior. The user explicitly forbids staging and commits, so all commit steps are omitted.

**Goal:** Build the internal, permission-authorized API-client registry and governance UI without introducing credentials, tokens, external authentication, or an External REST API.

**Architecture:** A dedicated Rust domain/application/repository boundary owns validated client metadata, system-defined scopes, lifecycle transitions, optimistic concurrency, and transactional audit history. Axum exposes explicit private admin DTOs and effective-permission-protected routes; Next.js mirrors them through a strict BFF and presents a bilingual protected admin workspace using public Client IDs only.

**Tech Stack:** Rust 1.98, Axum 0.8, SQLx 0.8, PostgreSQL, Next.js 16.3, React 19.2, TypeScript, Jest/Testing Library.

**Spec:** Approved Branch 24 design in the conversation and `DESIGN.md` Branch 24 boundary.

## Global Constraints

- Stay on `feat/24-api-client-management`; do not switch or create branches.
- Do not stage, commit, merge, push, deploy, or begin Branch 25.
- Reuse only `api_clients:read` and `api_clients:manage` for staff authorization.
- Scope definitions are system-owned; seed only `flights:read` and `analytics:read`.
- `ACTIVE` means administratively eligible for future API use, not currently authenticated.
- Add no credentials, secrets, tokens, external authentication middleware, rotation, expiry, last-used tracking, or External REST API.
- Use only a validated `TEST_DATABASE_URL` ending in `_test`; never run setup/destructive tests against DEV port 5433/database `x_fly`.
- Keep staff HttpOnly sessions, exact-Origin/CSRF protection, and private/no-store behavior unchanged.

---

### Task 1: Domain contract and migration

**Files:**
- Create: `backend/migrations/20260909000100_create_api_client_management.sql`
- Create: `backend/src/domain/api_client.rs`
- Modify: `backend/src/domain/mod.rs`
- Test: `backend/tests/api_client_rules.rs`

**Interfaces:**
- Produce `ApiClientStatus::{Active,Suspended,Revoked}` and `ApiClientScope::{FlightsRead,AnalyticsRead}`.
- Produce validated `CreateApiClientCommand` and `UpdateApiClientCommand` values and transition validation.
- Public Client IDs match `^XFC[A-HJ-NP-Z2-9]{16}$`; internal UUIDs stay private.

- [ ] Write domain tests for metadata normalization/bounds, typed scope rejection, active-without-scope rejection, allowed lifecycle transitions, and revoked terminal behavior.
- [ ] Run `cargo test --test api_client_rules` and confirm failure because the domain module does not exist.
- [ ] Add the migration with `api_clients`, `api_scope_catalog`, `api_client_allowed_scopes`, and `api_client_management_audit`; include uniqueness, FK/check constraints, actor attribution, versioning, timestamps, deterministic indexes, and the two catalog rows.
- [ ] Implement the minimal domain types and validators until `cargo test --test api_client_rules` passes.

### Task 2: Repository and transactional behavior

**Files:**
- Create: `backend/src/application/api_client.rs`
- Create: `backend/src/infrastructure/database/api_client.rs`
- Modify: `backend/src/application/mod.rs`
- Modify: `backend/src/infrastructure/database/mod.rs`
- Test: `backend/tests/api_client_repository.rs`

**Interfaces:**
- Produce `ApiClientManagement` backed by `ApiClientRepository`.
- Produce list filter `{ search, status, scope, limit, offset }`, page `{ items, next_offset }`, detail `{ client, audit }`, create/update/lifecycle methods, and canonical scope catalog.
- Every mutation takes `actor_staff_user_id`; update/lifecycle commands take `version`.

- [ ] Write integration tests for creation, opaque ID uniqueness, immutable public ID, parameterized search/filtering, limit+1 pagination, metadata/scope edits, version conflicts, lifecycle transitions, revoked preservation, and audit rows.
- [ ] Verify `TEST_DATABASE_URL` with the existing safety helper, then run `cargo test --test api_client_repository` and confirm expected compile/test failure.
- [ ] Implement the repository with bound SQL parameters, escaped `ILIKE` input, random ID collision retry, row locks, same-transaction scope replacement/audit insertion, and read-only detail history.
- [ ] Run the repository test until it passes without warnings.

### Task 3: Backend admin HTTP boundary

**Files:**
- Modify: `backend/src/infrastructure/http/admin/mod.rs`
- Modify: `backend/src/state.rs`
- Modify: `backend/src/main.rs`
- Test: `backend/tests/admin_api_clients_http.rs`

**Interfaces:**
- `GET/POST /api/v1/admin/api-clients`
- `GET /api/v1/admin/api-clients/scopes`
- `GET/PUT /api/v1/admin/api-clients/{client_id}`
- `POST /api/v1/admin/api-clients/{client_id}/{activate|suspend|revoke}`
- Reads require `api_clients:read`; writes require `api_clients:manage`.

- [ ] Write HTTP tests for authentication, effective permission grants, API_ADMIN access, unrelated-role and SYSTEM_ADMIN denial, DTO field minimization, malformed input, unknown scopes, bounded pagination, safe errors, no-store, and Origin/CSRF mutation rejection.
- [ ] Run `cargo test --test admin_api_clients_http` and confirm failure because routes are absent.
- [ ] Wire application state/startup and implement strict request DTO parsing, public-ID validation, status/scope parsing, explicit error mapping, and private/no-store responses.
- [ ] Run the HTTP test until it passes and confirm no role-name authorization checks exist.

### Task 4: Frontend typed boundary and protected routes

**Files:**
- Create: `frontend/src/lib/admin/apiClientTypes.ts`
- Modify: `frontend/src/lib/admin/adminBackend.ts`
- Modify: `frontend/src/lib/admin/adminAuthorization.ts`
- Create: `frontend/src/app/admin/api/api-clients/route.ts`
- Create: `frontend/src/app/admin/api/api-clients/scopes/route.ts`
- Create: `frontend/src/app/admin/api/api-clients/[clientId]/route.ts`
- Create lifecycle route handlers under `frontend/src/app/admin/api/api-clients/[clientId]/.../route.ts`
- Create protected pages under `frontend/src/app/admin/(protected)/api-clients/.../page.tsx`
- Modify: `frontend/src/components/admin/shell/AdminShell.tsx`
- Test: `frontend/src/tests/adminAuthorization.test.ts`
- Test: `frontend/src/tests/adminBackend.test.ts`
- Create: `frontend/src/tests/ApiClientManagementPage.test.tsx`

**Interfaces:**
- Produce strict public types for summaries, details, audit entries, scope catalog, and requests.
- BFF accepts only the canonical public Client ID path and forwards only staff cookie/content-type/origin/CSRF headers.
- Page access depends solely on effective permissions.

- [ ] Extend navigation/BFF/page tests first and confirm failures for the unavailable module/routes.
- [ ] Enable API Clients navigation for `api_clients:read`, add the shell title, strict BFF path allowlist, protected pages, and route handlers.
- [ ] Run the focused frontend tests until they pass.

### Task 5: Bilingual integration-governance UI

**Files:**
- Create: `frontend/src/components/admin/api-clients/ApiClientsWorkspace.tsx`
- Create: `frontend/src/components/admin/api-clients/ApiClientEditor.tsx`
- Create: `frontend/src/components/admin/api-clients/apiClientPresentation.ts`
- Create: `frontend/src/components/admin/api-clients/apiClientOperations.css`
- Create: `frontend/src/i18n/locales/apiClientManagement.ts`
- Modify: `frontend/src/i18n/locales/en.ts`
- Modify: `frontend/src/i18n/locales/th.ts`
- Modify: `frontend/src/i18n/formatters.ts`
- Create: `frontend/src/tests/ApiClientManagement.test.tsx`
- Modify: relevant localization/shell tests

**Interfaces:**
- List supports server-backed search, status and scope filters plus `nextOffset` loading.
- Editor supports create, safe metadata/scope edit, immutable Client ID, explicit status controls, and accessible lifecycle confirmations.
- Presentation maps every status, scope, action, validation/error state, and audit label to typed EN/TH copy.

- [ ] Write component tests for list/filter/pagination, create/edit payloads, immutable identifier display, status/scope labels, confirmation dialogs, revoked read-only behavior, safe errors, EN/TH, and table-only overflow structure.
- [ ] Run the focused test and confirm expected failures because components/locales are absent.
- [ ] Implement the terminal UI using existing Brand/AdminShell/Dialog/form/date conventions with warm ivory, charcoal, `#FFD400`, restrained radii, visible focus, textual statuses, and 48px-class controls.
- [ ] Run focused UI, i18n, shell, and route tests until they pass without React warnings or unexpected console output.

### Task 6: Documentation and complete verification

**Files:**
- Modify: `DESIGN.md`

- [ ] Update Branch 24 documentation with identity separation, API_ADMIN ownership, exact permissions, lifecycle, system-defined scopes, audit behavior, ACTIVE semantics, and the explicit Branch 25 deferral boundary.
- [ ] Confirm no credential/token/authentication schema, route, DTO, UI, or logging behavior was introduced.
- [ ] Run backend gates: `cargo fmt --check`, `cargo build`, `cargo clippy --all-targets`, `cargo test --no-fail-fast`.
- [ ] Run frontend gates: `npm test -- --runInBand`, `npm run lint`, `npm run typecheck`, `npm run build`, `npm audit --audit-level=high`.
- [ ] Run repository gates: `docker compose config --quiet`, `git diff --check`.
- [ ] Record exact test counts and `git status --short`; report every modified/untracked file and recommended manual QA scenarios.
