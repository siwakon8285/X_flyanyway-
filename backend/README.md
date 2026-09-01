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
```

The validation endpoint is the Continue gate: it revalidates authorization and expiry and independently requires held seats to equal adults + children. Lap infants do not consume seats. Conflicting inventory returns HTTP 409 with `SEAT_UNAVAILABLE` and `conflictingSeats`.

The passenger resource uses the same hold-scoped HttpOnly authorization cookie. `GET` returns the active hold, authoritative passenger slots, any saved passenger draft, and `readyToContinue`. `PUT` atomically replaces the full draft after checking the hold, held-seat count, passenger count/order/types, and every passenger field. Adult, child, and infant slots are always ordered in that sequence. Age is attained age on outbound departure: adult 12+, child 2–11, and infant under 2.

Passenger details are stored in `hold_passengers` and cascade with their temporary hold; this does not create the future final Booking aggregate. API validation errors contain stable codes and field coordinates, never submitted PII. Passenger repository failures are logged without raw database error details to avoid logging sensitive bound values.

## Local development

Copy the example environment once from the repository root:

```bash
cp .env.example .env
```

The normal development topology runs PostgreSQL in Docker and the Rust API directly on the Mac. Start only the development database from the repository root:

```bash
docker compose up -d db
```

Then start Axum in another terminal:

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

## Isolated PostgreSQL tests

Repository and race tests refuse a database whose URL path does not end in `_test`. `TEST_DATABASE_URL` is separate from `DATABASE_URL` and uses host port `5434` for host-run tests. Start the isolated, tmpfs-backed test database only when required:

```bash
docker compose --profile test up -d db_test
cd backend
cargo test
```

The default example name is `x_fly_concurrency_test`; callers can supply any dedicated `<project_db>_test` through `TEST_DATABASE_URL`. Never point this variable at development or production data.
