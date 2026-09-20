#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
common=(PATH="$PATH" XFLY_TARGET_ENV=TEST XFLY_SAFETY_CONFIRM=TEST_ONLY
  XFLY_HIGH_VU_CONFIRM=TEST_ONLY XFLY_DIAGNOSTIC_ENDPOINT=flight_search
  XFLY_BASE_URL=http://127.0.0.1:18080 XFLY_API_PREFIX=/api/v1
  XFLY_TEST_DB_HOST=127.0.0.1 XFLY_TEST_DB_PORT=5434 XFLY_TEST_DB_NAME=x_fly_concurrency_test
  XFLY_TEST_DATABASE_URL=postgresql://test@127.0.0.1:5434/x_fly_concurrency_test
  XFLY_READ_MANIFEST="$repo_root/performance/data/public-read.example.json")
reject() {
  local expected="$1" output code
  shift
  set +e
  output="$(env -i "${common[@]}" "$@" bash "$repo_root/performance/scripts/run-endpoint-diagnostic.sh" 2>&1)"
  code=$?
  set -e
  [[ "$code" -ne 0 && "$output" == *"$expected"* ]] || { printf '%s\n' "$output"; exit 1; }
}
reject XFLY_HIGH_VU_CONFIRM XFLY_HIGH_VU_CONFIRM=
reject XFLY_DIAGNOSTIC_ENDPOINT XFLY_DIAGNOSTIC_ENDPOINT=
reject XFLY_DIAGNOSTIC_ENDPOINT XFLY_DIAGNOSTIC_ENDPOINT=invalid
reject 'XFLY_TARGET_ENV must be TEST' XFLY_TARGET_ENV=DEV
reject 'XFLY_SAFETY_CONFIRM must equal TEST_ONLY' XFLY_SAFETY_CONFIRM=no
reject '5433 is reserved for DEV' XFLY_TEST_DB_PORT=5433
reject 'x_fly is reserved for DEV' XFLY_TEST_DB_NAME=x_fly
reject 'only scheme, host, and optional port' XFLY_BASE_URL=http://127.0.0.1:18080/api/v1
reject_inspect() {
  local expected="$1" output code
  shift
  set +e
  output="$(env -i "${common[@]}" "$@" k6 inspect --include-system-env-vars "$repo_root/performance/k6/endpoint-diagnostic.js" 2>&1)"
  code=$?
  set -e
  [[ "$code" -ne 0 && "$output" == *"$expected"* ]] || { printf '%s\n' "$output"; exit 1; }
}
reject_inspect XFLY_HIGH_VU_CONFIRM XFLY_HIGH_VU_CONFIRM=
reject_inspect XFLY_DIAGNOSTIC_ENDPOINT XFLY_DIAGNOSTIC_ENDPOINT=
reject_inspect XFLY_DIAGNOSTIC_ENDPOINT XFLY_DIAGNOSTIC_ENDPOINT=invalid
reject_inspect 'XFLY_TARGET_ENV must be TEST' XFLY_TARGET_ENV=DEV
reject_inspect 'XFLY_SAFETY_CONFIRM must equal TEST_ONLY' XFLY_SAFETY_CONFIRM=no
reject_inspect XFLY_READ_MANIFEST XFLY_READ_MANIFEST=
reject_inspect 'only scheme, host, and optional port' XFLY_BASE_URL=http://127.0.0.1:18080/api/v1
for endpoint in flight_search flight_detail seat_availability; do
  env -i "${common[@]}" XFLY_DIAGNOSTIC_ENDPOINT="$endpoint" k6 inspect --include-system-env-vars "$repo_root/performance/k6/endpoint-diagnostic.js" >/dev/null
done
printf '%s\n' 'endpoint diagnostic guards and three k6 inspections passed'
