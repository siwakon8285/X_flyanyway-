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

The permission-aware workspace now includes the Executive Dashboard, Flight Management, Booking Operations, Ticket/Passenger Operations, and API Client Management modules. Navigation reflects effective permissions, but each backend route independently enforces authorization. Staff account provisioning remains an operator CLI responsibility rather than a public or browser registration flow. Set `X_FLY_INTERNAL_API_URL` for the server-only backend base URL when it differs from `NEXT_PUBLIC_X_FLY_API_URL`.

## Passenger Information and Review handoff

Seat selection opens `/booking/passengers?holdId={public-hold-id}` with only non-sensitive search/recovery context. The Passenger page loads authoritative passenger counts, types, seats, and expiry from the API using the scoped HttpOnly hold cookie. A successful explicit save continues directly to `/booking/review` with allowlisted recovery context.

Travel Extras are not an active customer step. The legacy `/booking/extras` route exists only for compatibility and redirects to Review; historical Extras data may still be rendered where required without reintroducing optional-extra selection.

## Quality checks

```bash
npm run lint
npm run typecheck
npm test
npm run build
npm audit --audit-level=high
```

The frontend uses npm only. `npm test` invokes the checked-in `jest --runInBand` script. The checked-in GitHub Actions workflow runs the same test, lint, typecheck, build, and high-severity audit gates; remote GitHub-hosted execution is not yet verified on this branch.

Passenger validation is dependency-free TypeScript and mirrors the backend rules. Customer-facing Passenger strings are supplied by the existing typed English/Thai dictionaries. Passenger PII is not written to URLs, local storage, or session storage.
