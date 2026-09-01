# X-Fly Anyway Frontend

Next.js App Router foundation for the X-Fly Anyway design system.

## Local development

```bash
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) to use the local booking flow.

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
