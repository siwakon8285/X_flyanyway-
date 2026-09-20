#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

fail() {
  printf 'Phase 1 preflight refused: %s\n' "$1" >&2
  exit 1
}

required() {
  local name="$1"
  [[ -n "${!name:-}" ]] || fail "$name must be explicitly configured"
}

required XFLY_TARGET_ENV
required XFLY_SAFETY_CONFIRM
required XFLY_BASE_URL
required XFLY_API_PREFIX
required XFLY_TEST_DB_HOST
required XFLY_TEST_DB_PORT
required XFLY_TEST_DB_NAME
required XFLY_TEST_DATABASE_URL
required XFLY_READ_MANIFEST

[[ "$XFLY_TARGET_ENV" == TEST ]] || fail 'XFLY_TARGET_ENV must be TEST'
[[ "$XFLY_SAFETY_CONFIRM" == TEST_ONLY ]] || fail 'XFLY_SAFETY_CONFIRM must equal TEST_ONLY'
[[ "$XFLY_TEST_DB_NAME" != x_fly ]] || fail 'x_fly is reserved for DEV'
[[ "$XFLY_TEST_DB_PORT" != 5433 ]] || fail '5433 is reserved for DEV'
[[ "$XFLY_TEST_DB_NAME" == x_fly_concurrency_test ]] || fail 'database must be x_fly_concurrency_test'

if [[ "$XFLY_TEST_DB_HOST:$XFLY_TEST_DB_PORT" != 127.0.0.1:5434 && \
      "$XFLY_TEST_DB_HOST:$XFLY_TEST_DB_PORT" != 172.20.0.3:5432 ]]; then
  fail 'database target must be TEST 127.0.0.1:5434 or Docker TEST 172.20.0.3:5432'
fi

case "$XFLY_BASE_URL" in
  http://*|https://*) ;;
  *) fail 'XFLY_BASE_URL must be an HTTP(S) URL' ;;
esac
[[ "$XFLY_BASE_URL" != *'@'* ]] || fail 'XFLY_BASE_URL must not contain credentials'
[[ "$XFLY_BASE_URL" != *'?'* && "$XFLY_BASE_URL" != *'#'* ]] || fail 'XFLY_BASE_URL must not contain query or fragment'
if [[ "$XFLY_BASE_URL" =~ ^https?://(127\.0\.0\.1|localhost):8080(/|$) ]]; then
  fail '127.0.0.1:8080 is not an approved Phase 1 target'
fi

case "$XFLY_TEST_DATABASE_URL" in
  *:5433*|*/x_fly|*/x_fly\?*|*/x_fly#*) fail 'TEST database URL points at forbidden DEV target' ;;
esac
if [[ -n "${DATABASE_URL:-}" ]]; then
  case "$DATABASE_URL" in
    *:5433*|*/x_fly|*/x_fly\?*|*/x_fly#*) fail 'DATABASE_URL points at forbidden DEV target' ;;
  esac
fi

command -v node >/dev/null 2>&1 || fail 'node is required'
command -v k6 >/dev/null 2>&1 || fail 'k6 is required'
command -v psql >/dev/null 2>&1 || fail 'psql is required'
command -v curl >/dev/null 2>&1 || fail 'curl is required'

if ! health_url="$(node "$repo_root/performance/scripts/resolve-api-url.mjs" root /health)"; then
  fail 'API URL contract is invalid'
fi
if ! airport_url="$(node "$repo_root/performance/scripts/resolve-api-url.mjs" api /airports)"; then
  fail 'API URL contract is invalid'
fi

EXPECTED_DB_HOST="$XFLY_TEST_DB_HOST" EXPECTED_DB_PORT="$XFLY_TEST_DB_PORT" EXPECTED_DB_NAME="$XFLY_TEST_DB_NAME" node --input-type=module <<'NODE'
const expectedHost = process.env.EXPECTED_DB_HOST;
const expectedPort = process.env.EXPECTED_DB_PORT;
const expectedName = process.env.EXPECTED_DB_NAME;
for (const name of ['XFLY_TEST_DATABASE_URL', 'DATABASE_URL']) {
  const value = process.env[name];
  if (!value) continue;
  let parsed;
  try {
    parsed = new URL(value);
  } catch {
    throw new Error(`${name} is not a valid PostgreSQL URL`);
  }
  if (!['postgres:', 'postgresql:'].includes(parsed.protocol)) {
    throw new Error(`${name} must use the PostgreSQL URL scheme`);
  }
  const database = decodeURIComponent(parsed.pathname.replace(/^\//, ''));
  const port = parsed.port || '5432';
  if (parsed.hostname !== expectedHost || port !== expectedPort || database !== expectedName) {
    throw new Error(`${name} does not identify the configured TEST target`);
  }
}
NODE

[[ -f "$XFLY_READ_MANIFEST" ]] || fail 'XFLY_READ_MANIFEST does not exist'

MANIFEST_PATH="$XFLY_READ_MANIFEST" node --input-type=module <<'NODE'
import { readFile } from 'node:fs/promises';

const manifest = JSON.parse(await readFile(process.env.MANIFEST_PATH, 'utf8'));
if (manifest?.target?.environment !== 'TEST' || manifest?.target?.databaseName !== 'x_fly_concurrency_test') {
  throw new Error('read manifest target is not the approved TEST database');
}
if (!/^\d{4}-\d{2}-\d{2}$/.test(manifest.departureDate)) {
  throw new Error('read manifest departureDate is invalid');
}
if (!Array.isArray(manifest.cases) || manifest.cases.length < 4) {
  throw new Error('read manifest needs at least four read cases');
}
for (const item of manifest.cases) {
  if (!/^[A-Z]{3}$/.test(item.origin) || !/^[A-Z]{3}$/.test(item.destination) || item.origin === item.destination) {
    throw new Error('read manifest contains an invalid airport pair');
  }
  if (!['business', 'first'].includes(item.cabin) || !/^[A-Za-z0-9-]+$/.test(item.flightId)) {
    throw new Error('read manifest contains an invalid cabin or flight identifier');
  }
}
NODE

db_name="$(PGCONNECT_TIMEOUT=3 psql --no-psqlrc --quiet --tuples-only --no-align \
  --dbname="$XFLY_TEST_DATABASE_URL" \
  --command='SELECT current_database();' | tr -d '[:space:]')" || fail 'TEST PostgreSQL identity query failed'
[[ "$db_name" == x_fly_concurrency_test ]] || fail 'connected PostgreSQL database is not x_fly_concurrency_test'

assert_http_200() {
  local label="$1"
  local url="$2"
  local status
  status="$(curl --silent --show-error --output /dev/null --write-out '%{http_code}' --max-time 10 "$url" 2>/dev/null || true)"
  [[ "$status" == 200 ]] || fail "$label did not return HTTP 200"
}

assert_http_200 'health endpoint' "$health_url"
assert_http_200 'airport reference endpoint' "$airport_url"

printf '%s\n' 'Phase 1 preflight passed for TEST target'
