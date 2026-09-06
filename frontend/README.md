# X-Fly Anyway Frontend

Next.js App Router foundation for the X-Fly Anyway design system.

## Local development

```bash
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) to use the local booking flow.

## Internal staff shell

`/admin/login` and `/admin` are separate from the accountless customer application. Next.js forwards staff authentication only through same-origin `/admin/api/auth/*` handlers, filtering cookies down to `x_fly_staff_session`. The protected admin layout validates the session on the server before rendering, while client-side permissions only control navigation and presentation. The backend remains authoritative.

The Branch 19 workspace supports the shared English/Thai language setting and intentionally omits unavailable Flights, Bookings, Tickets, Reports, API Clients, Staff Management, and Executive Dashboard links. It contains no analytics or fabricated metrics. Set `X_FLY_INTERNAL_API_URL` for the server-only backend base URL when it differs from `NEXT_PUBLIC_X_FLY_API_URL`.

## Passenger Information handoff

Seat selection opens `/booking/passengers?holdId={public-hold-id}` with only non-sensitive search/recovery context. The Passenger page loads authoritative passenger counts, types, seats, and expiry from the API using the scoped HttpOnly hold cookie. A successful explicit save stays on the implemented Passenger route and shows a ready state.

Branch 13a can use the tested future handoff contract `/booking/extras?holdId={public-hold-id}` after the saved passenger resource reports `readyToContinue`. This branch does not navigate to that route or provide an Extras page.

## Quality checks

```bash
npm run lint
npm run typecheck
npm test
npm run build
npm audit --audit-level=high
```

Passenger validation is dependency-free TypeScript and mirrors the backend rules. Customer-facing Passenger strings are supplied by the existing typed English/Thai dictionaries. Passenger PII is not written to URLs, local storage, or session storage.
