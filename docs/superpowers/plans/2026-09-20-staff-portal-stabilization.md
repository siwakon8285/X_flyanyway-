# Staff Portal Stabilization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the diagnosed Flight Manager save errors business-friendly and bilingual, prove modeled-cost-only edits remain safe after inventory materialization, and remove the remaining raw English Ticket Operations filter labels.

**Architecture:** Keep the Rust flight lifecycle and inventory-protection rules unchanged. Interpret stable backend error codes in `FlightEditor`, add dedicated EN/TH locale copy for structural, stale-version, status, and generic failures, and reuse the existing ticket status/cabin translation maps for filter options.

**Tech Stack:** Rust, SQLx/PostgreSQL TEST fixtures, React/Next.js, TypeScript, Jest, React Testing Library, existing X-Fly i18n provider.

**Spec:** The current Staff Portal QA/fix request in the conversation.

## Global Constraints

- Frontend-only unless the existing backend behavior proves a necessary safe correction; no DEV database operations.
- Do not weaken operational flight mutation restrictions, RBAC, booking integrity, payment, ticket, cancellation, or refund rules.
- Cost-only changes remain planning metadata and must not mutate route, schedule, aircraft, capacity, prices, inventory, bookings, or passengers.
- Do not expose raw backend/domain/database error text to staff users.
- Do not stage, commit, push, merge, deploy, switch branches, or edit applied migrations.

## Review Focus

- A cost-only update after a materialized flight instance must succeed without permitting structural edits: pin in the repository integration test.
- `FLIGHT_STRUCTURAL_CONFLICT`, stale-version, permission, and unknown failures must map to plain EN/TH copy: pin in `FlightEditor.test.tsx`.
- Ticket status and cabin filter options must follow the selected locale while submitted values remain canonical: pin in `TicketOperations.test.tsx`.
- The existing effective-permission navigation must remain role-scoped and SYSTEM_ADMIN must not gain implicit flight write access: retain and rerun `AdminShell.test.tsx` and backend RBAC tests.

### Task 1: Add failing regression coverage

**Files:**
- Modify: `backend/tests/flight_management_repository.rs`
- Modify: `frontend/src/tests/FlightEditor.test.tsx`
- Modify: `frontend/src/tests/TicketOperations.test.tsx`

- [ ] **Step 1: Add the repository regression assertion**

After inserting a `flight_instances` row and asserting a structural change is rejected, create a command that changes only `modeled_operating_cost_amount`, call `repository.update` with the current version, and assert the update succeeds and the returned cost is `Some(1_250_000)`.

- [ ] **Step 2: Add the typed Flight Editor error tests**

Make the PUT response return `{ error: { code: "FLIGHT_STRUCTURAL_CONFLICT", message: "..." } }` and assert the English alert contains the business-facing structural message and not the backend message. Repeat with `locale: "th"` and assert the Thai message is rendered. Add a stale-version assertion so the existing conflict copy remains distinct.

- [ ] **Step 3: Add the Ticket Operations filter locale test**

Render `TicketsWorkspace` with one page response in English and Thai. Assert filter options are localized (`Issued`/`Cancelled`, then their Thai equivalents; cabin names likewise) while option values remain `ISSUED`, `CANCELLED`, `BUSINESS`, `FIRST`, `ECONOMY`, and `PREMIUM_ECONOMY`.

- [ ] **Step 4: Run focused tests and record RED**

Run the focused repository test and the two frontend test files. The new Flight Editor copy and raw Ticket filter labels must fail before implementation; if the repository regression already passes, record that the safe cost-only branch is already present and keep the test as a guard rather than changing backend logic.

### Task 2: Implement the smallest safe localization fixes

**Files:**
- Modify: `frontend/src/components/admin/flights/FlightEditor.tsx`
- Modify: `frontend/src/components/admin/tickets/TicketsWorkspace.tsx`
- Modify: `frontend/src/i18n/locales/flightManagement.ts`

- [ ] **Step 1: Map known flight error codes to locale keys**

Replace the current broad `includes("CONFLICT")` mapping with a switch over known codes. Map structural conflict to a new operational-inventory message, stale version to the existing reload message, status conflict to a status message, permission/not-found/validation/unavailable to existing appropriate keys, and unknown codes to a new safe generic save failure. Never render `error.message` from the API.

- [ ] **Step 2: Add matched EN/TH copy**

Add locale keys for structural conflict, status conflict, and generic save failure. Use plain operational language: the structural message should say seat inventory is already in use; the stale message should say the flight changed and needs reloading; the status message should explain that the current flight status does not allow the action. Keep the cost-only path free of structural warnings when the backend returns success.

- [ ] **Step 3: Reuse existing ticket presentation maps**

Import the existing `statusKey` and `cabinKey` helpers into `TicketsWorkspace` and use them for filter option labels, retaining the API enum values as option values.

### Task 3: GREEN, refactor, and full verification

**Files:**
- Modify only the files above plus the plan document created for this task.

- [ ] **Step 1: Run focused tests until green**

Run the repository regression, `FlightEditor.test.tsx`, and `TicketOperations.test.tsx`, then rerun the existing `AdminShell`, `BookingManagement`, `ApiClientManagement`, and `ExecutiveDashboard` suites.

- [ ] **Step 2: Refactor only if duplication is demonstrated**

Keep error-code mapping local to `FlightEditor` unless a second consumer requires it. Keep canonical ticket values and localized labels together in the existing presentation helper; do not create a new localization framework.

- [ ] **Step 3: Run complete verification sequentially**

Run `cargo fmt --all -- --check`, `cargo check --tests --locked`, focused backend flight/RBAC tests, `cargo test --locked --tests`, focused frontend role/dashboard tests, the full frontend suite, `npm run lint`, `npm run typecheck`, `npm run build`, and `git diff --check`. Run frontend build after typecheck so generated `.next` state is not shared concurrently.

- [ ] **Step 4: Review the diff and report status**

Confirm no migration, DEV database, authorization, operational mutation, or payment/ticket behavior changed. Report exact root causes, files, tests, verification outcomes, remaining language gaps, and `git status --short` without staging or committing.
