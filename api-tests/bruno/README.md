# Branch 25 external token regression

This Bruno collection is a small black-box HTTP regression suite for the
Task 5 `POST /api/v1/external/token` contract. It complements the Rust unit,
repository, and HTTP integration tests; it does not replace them.

## Prerequisites

- Bruno CLI 4.x. Check the installed version with `bru --version`.
- A local/test API listening at `http://127.0.0.1:8080`, or a different URL
  supplied through `baseUrl`.
- An already-issued, currently ACTIVE TEST API-client credential for the
  success and wrong-secret requests. This collection does not log in staff or
  issue credentials through the admin API.

Use TEST/local data only. Never use production credentials.

## Supplying runtime values

The collection contains no credential values or checked-in environment file.
Supply the non-secret URL and the active credential at runtime with the
Bruno 4.1 `--env-var` option:

```sh
cd api-tests/bruno
bru run external-auth \
  --env-var baseUrl="${BASE_URL:-http://127.0.0.1:8080}" \
  --env-var clientId="$TEST_CLIENT_ID" \
  --env-var clientSecret="$TEST_CLIENT_SECRET" \
  --reporter-skip-body
```

Keep `TEST_CLIENT_ID` and `TEST_CLIENT_SECRET` in a private shell or secret
manager. Do not put them in this repository, shell scripts, request files, or
logs. `--reporter-skip-body` keeps ordinary CLI reports from printing response
bodies, including a bearer token.

The invalid-request and CORS requests use only non-secret, canonical-shaped
placeholders and can be run without an active credential. The wrong-secret
request still needs `clientId`; its intentionally invalid canonical secret is
fixed in the request and is not a real credential.

## Running the collection

From this directory, run the complete collection with the command above. Run
one request or the external-auth folder when isolating a failure:

```sh
bru run external-auth/01-token-success.bru \
  --env-var baseUrl="${BASE_URL:-http://127.0.0.1:8080}" \
  --env-var clientId="$TEST_CLIENT_ID" \
  --env-var clientSecret="$TEST_CLIENT_SECRET" \
  --reporter-skip-body
bru run external-auth/02-token-unknown-field.bru \
  --env-var baseUrl="${BASE_URL:-http://127.0.0.1:8080}" \
  --reporter-skip-body
```

The collection currently covers Task 5 only: token exchange success,
structural request failures, generic credential failure, and the absence of
browser CORS headers on the external route. A failure means the observed
HTTP contract differs from the locked Task 5 contract or the configured API
environment/credential is unavailable. Protected endpoint and scope tests
will be added only when their Branch 25 tasks exist.

Returned access tokens are asserted in memory only and are never saved by
the collection. No request logs, custom scripts, or reports are added that
print secrets or tokens.
