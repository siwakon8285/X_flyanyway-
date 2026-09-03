# X-Fly booking API

This service provides the PostgreSQL-authoritative seat hold and Passenger Information flows.

## Architecture

The dependency direction is:

```text
Axum handler
  → application use case
    → domain repository trait
      → SQLx repository
        → PostgreSQL
```

Domain types have no Axum dependencies. The SQLx repository uses deterministic seat-number ordering and `SELECT ... FOR UPDATE OF seat` inside a transaction. Expired holds are interpreted as available at query/lock time, so correctness never depends on a cleanup worker.

## Lifecycle and authorization

- Inventory is `AVAILABLE` when it is sellable, not booked, and has no active hold.
- A hold has a fixed server-controlled 10-minute expiry by default.
- Reading or saving passengers never refreshes or extends that expiry.
- The public hold UUID may be kept by the browser for revalidation and handoff.
- A separate random 256-bit secret is stored only in a scoped `HttpOnly`, `SameSite=Lax` cookie; PostgreSQL stores only its SHA-256 hash.
- Cookie names are scoped by hold UUID, so a browser can retain more than one unrelated hold without exposing secrets to JavaScript.
- `BOOKED` inventory cannot be held. The schema is ready for a later transaction to lock an active hold, create the booking, set its seats to `BOOKED`, clear `hold_id`, set `booked_at`, and set `consumed_at`. That conversion is intentionally outside Branch 12.

## API

```text
GET    /health
GET    /api/v1/flights/{flight_id}/seats?departure=YYYY-MM-DD&cabin=...&holdId=...
POST   /api/v1/seat-holds
GET    /api/v1/seat-holds/{hold_id}
PUT    /api/v1/seat-holds/{hold_id}
DELETE /api/v1/seat-holds/{hold_id}
POST   /api/v1/seat-holds/{hold_id}/validation
GET    /api/v1/seat-holds/{hold_id}/passengers
PUT    /api/v1/seat-holds/{hold_id}/passengers
GET    /api/v1/seat-holds/{hold_id}/extras
PUT    /api/v1/seat-holds/{hold_id}/extras
GET    /api/v1/seat-holds/{hold_id}/review
```

The validation endpoint is the Continue gate: it revalidates authorization and expiry and independently requires held seats to equal adults + children. Lap infants do not consume seats. Conflicting inventory returns HTTP 409 with `SEAT_UNAVAILABLE` and `conflictingSeats`.

The passenger resource uses the same hold-scoped HttpOnly authorization cookie. `GET` returns the active hold, authoritative passenger slots, any saved passenger draft, and `readyToContinue`. `PUT` atomically replaces the full draft after checking the hold, held-seat count, passenger count/order/types, and every passenger field. Adult, child, and infant slots are always ordered in that sequence. Age is attained age on outbound departure: adult 12+, child 2–11, and infant under 2.

Passenger details are stored in `hold_passengers` and cascade with their temporary hold; this does not create the future final Booking aggregate. API validation errors contain stable codes and field coordinates, never submitted PII. Passenger repository failures are logged without raw database error details to avoid logging sensitive bound values.

## Local development

Copy the Compose environment from the repository root and the API environment into `backend/`:

```bash
cp .env.example .env
cp backend/.env.example backend/.env
```

The normal development topology runs PostgreSQL in Docker and the Rust API directly on the Mac. Start only the development database from the repository root:

```bash
docker compose up -d db
```

Then start Axum in another terminal. It reads backend secrets from `backend/.env`:

```bash
cd backend
cargo run
```

Start Next.js in a third terminal:

```bash
cd frontend
npm run dev
```

Stop the backend and frontend with `Ctrl+C`. Stop the development database from the repository root with:

```bash
docker compose stop db
```

`DATABASE_URL` is the host-run API connection and uses `127.0.0.1:5433` by default. The API binds to `127.0.0.1:8080`, and the frontend continues to use `http://localhost:8080/api/v1`. The development database is published on `localhost:${POSTGRES_HOST_PORT}` (default `5433`). Configure pgAdmin on the Mac with:

```text
Host: localhost
Port: 5433 (or POSTGRES_HOST_PORT)
Database: x_fly (or POSTGRES_DB)
Username: x_fly_app (or POSTGRES_USER)
Password: the local POSTGRES_PASSWORD
```

The host-run backend and pgAdmin both connect through port `5433`, so they inspect the same Docker PostgreSQL instance. Useful tables are `flight_instances`, `flight_seats`, `seat_holds`, and `hold_passengers`; the passenger table makes successful saves inspectable locally while remaining protected behind the API in the application. `seat_holds` exposes expiry/ownership hashes without storing the browser secret. Do not publish production PostgreSQL. Production inspection remains pgAdmin over an SSH tunnel to server-side `127.0.0.1:5432`.

The root Compose file intentionally contains PostgreSQL services only. `backend/Dockerfile` is retained for future production/self-hosted deployment builds, but it is not part of the local Compose workflow.

## Travel Extras demo fixtures

Travel Extras are persisted against the active, authorized seat hold in `hold_extras` and are exposed through `GET`/`PUT /api/v1/seat-holds/{hold_id}/extras`. The browser submits only `passengerOrdinal`, `productCode`, and `quantity`; the backend resolves eligibility and price. Saved rows retain a whole-baht unit-price snapshot in `THB`, and every PUT atomically replaces the complete selection set.

Demo extra-baggage prices are `BAG_10KG` = THB 1,500, `BAG_20KG` = THB 2,800, and `BAG_30KG` = THB 3,900. Adults and children may select one baggage tier, one meal preference, and multiple assistance requests. Infants are informational-only in this MVP: they have no independent seat, extra-baggage purchase, meal preference, or assistance selection.

Included cabin fixtures are:

| Cabin | Cabin baggage | Checked baggage | Seat selection | Meal service |
| --- | ---: | ---: | --- | --- |
| Economy | 7 kg | 20 kg | Included | Standard |
| Premium Economy | 7 kg | 25 kg | Included | Enhanced |
| Business | 10 kg | 40 kg | Included | Premium |
| First | 14 kg | 50 kg | Included | Signature |

`seat_holds.extras_saved_at` is strictly a Travel Extras workflow-readiness marker. It means the customer explicitly reviewed and saved the Extras step, including an explicit save with no selections. It does **not** mean payment completed, a booking was confirmed, or a ticket was issued. Later booking, payment, and ticketing branches must use their own state and timestamps and must never reuse `extras_saved_at` for those meanings.

## Booking Review and demo pricing

`GET /api/v1/seat-holds/{hold_id}/review` requires the authorized hold to be active, all seated inventory to remain held, the complete passenger draft to match the hold party, and `extras_saved_at` to exist. It reads passenger-facing summary fields and persisted Extras only; passport numbers, contact details, and emergency contacts are not included in the Review response.

The backend owns the Review amount. Demo flight-service schedules and whole-baht cabin fares are stored in PostgreSQL. Adults and children each pay 100% of the selected cabin fare, while the explicitly modeled lap-infant fare is THB 0. Deterministic fixture charges are `DEMO_PASSENGER_TAX` = THB 700 per seated passenger, `DEMO_AIRPORT_FEE` = THB 500 per seated passenger, and `DEMO_BOOKING_FEE` = THB 300 per hold. These are academic demo fixtures, not real government, airport, or airline charges.

The first ready Review GET idempotently materializes one row in `hold_review_pricing`. This row is an internal cache/materialization of authoritative Review pricing so repeated reads and a future payment use case receive the same amount. It is **not** payment, booking confirmation, ticket issuance, or a general pattern for arbitrary GET side effects. Repeated GETs do not add rows or change prices. A successful Passenger or Extras re-save explicitly invalidates the row; the next ready Review GET materializes a new snapshot from current authoritative state. None of these operations updates `seat_holds.expires_at`.

The returned `DEMO_FIXTURE_NONREFUNDABLE_NO_CHANGES` condition is a project demo configuration only. It must always be presented as a fixture policy and never represented as a real airline fare rule.

## Stripe Test Mode payment and paid inventory lifecycle

Payment is available only for an HttpOnly-cookie-authorized, active hold whose seats, passengers, saved Extras marker, and current `hold_review_pricing` snapshot all remain valid. `GET /api/v1/seat-holds/{hold_id}/payment` returns that server-owned amount and currency; create and simulation requests never accept pricing or card fields. Stripe Elements collects Card details directly; X-Fly receives only a UUID request ID and payment method.

`PaymentApplication` depends on a provider-neutral payment/reconciliation boundary. Stripe’s HTTP DTOs and API details remain in infrastructure; Bitcoin uses an invalid, deterministic demo destination and a fixed display conversion of 1 BTC = THB 2,000,000. It never generates key material or contacts a blockchain.

Payment attempts use `CREATED`, `PROCESSING`, `AWAITING_PAYMENT`, `SUCCEEDED`, `FAILED`, and `CANCELLED`. Failed and cancelled attempts are historical and retryable while the hold remains active. Request IDs are idempotent and protected by a request fingerprint. Partial unique indexes allow only one open attempt and one successful attempt per hold.

A transition to `SUCCEEDED` is also the paid-inventory finalization boundary. One guarded PostgreSQL transaction locks the authorized hold and attempt, revalidates the Review snapshot and held seats, records the paid seat association in `payment_attempt_seats`, changes those seats from `AVAILABLE` with this hold ID to `BOOKED` with `booked_at` set and `hold_id` cleared, sets `seat_holds.consumed_at`, and finally records the payment as `SUCCEEDED`. Any failure rolls the whole transaction back, so successful payment cannot coexist with expirable inventory. Expiry before this transition persists the attempt as `FAILED` with `HOLD_EXPIRED`; it never extends or replaces the hold. Ticket and QR representation remain a later concern and will consume the successful attempt, its finalized-seat association, the consumed hold, and BOOKED inventory state.

## Stripe Test Mode manual QA (next checkpoint)

Use only Stripe Test Mode: `backend/.env` holds `sk_test_` and the Stripe CLI-generated `whsec_`, while `frontend/.env.local` holds the public `pk_test_` value. The frontend key, backend key, and `stripe listen` session must all belong to the same Stripe sandbox/account context. Run `stripe login`, then `stripe listen --forward-to localhost:8080/api/v1/payments/stripe/webhook`, and place its generated webhook secret manually in `backend/.env`. Confirm a successful test card, a declined card, and a 3DS card; refresh while confirmation is pending; and verify both webhook delivery and payment-context reconciliation. In the Stripe Dashboard, confirm the PaymentIntent matches the DB `provider_reference`; on success the seat is `BOOKED` and the hold is consumed. For failed or confirmed-cancelled attempts, verify ordinary hold expiry/release can reclaim inventory. No real card must be charged. Bitcoin remains mock-only.

## Isolated PostgreSQL tests

Repository and race tests refuse a database whose URL path does not end in `_test`. `TEST_DATABASE_URL` is separate from `DATABASE_URL` and uses host port `5434` for host-run tests. Start the isolated, tmpfs-backed test database only when required:

```bash
docker compose --profile test up -d db_test
cd backend
cargo test
```

The default example name is `x_fly_concurrency_test`; callers can supply any dedicated `<project_db>_test` through `TEST_DATABASE_URL`. Never point this variable at development or production data.
