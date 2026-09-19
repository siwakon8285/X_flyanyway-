#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
preflight="$repo_root/performance/scripts/preflight.sh"

run_rejected_case() {
  local name="$1"
  local expected="$2"
  shift 2
  local output
  output="$({ env -i PATH="$PATH" HOME="${HOME:-}" "$@" bash "$preflight"; } 2>&1 || true)"
  if [[ "$output" != *"$expected"* ]]; then
    printf 'guard case failed: %s\n%s\n' "$name" "$output" >&2
    return 1
  fi
}

common_env=(
  XFLY_TARGET_ENV=TEST
  XFLY_SAFETY_CONFIRM=TEST_ONLY
  XFLY_BASE_URL=https://load-test.example.invalid
  XFLY_API_PREFIX=/api/v1
  XFLY_TEST_DB_HOST=127.0.0.1
  XFLY_TEST_DB_PORT=5434
  XFLY_TEST_DB_NAME=x_fly_concurrency_test
  XFLY_TEST_DATABASE_URL=postgresql://test@example.invalid/x_fly_concurrency_test
  XFLY_READ_MANIFEST="$repo_root/performance/data/public-read.example.json"
)

run_rejected_case \
  "DEV port" \
  "5433 is reserved for DEV" \
  env "${common_env[@]}" XFLY_TEST_DB_PORT=5433

run_rejected_case \
  "missing API prefix" \
  "XFLY_API_PREFIX must be explicitly configured" \
  env "${common_env[@]}" XFLY_API_PREFIX=

run_rejected_case \
  "embedded API prefix in base URL" \
  "XFLY_BASE_URL must contain only scheme, host, and optional port" \
  env "${common_env[@]}" XFLY_BASE_URL=https://load-test.example.invalid/api/v1

run_rejected_case \
  "malformed API prefix" \
  "XFLY_API_PREFIX must be a relative path without credentials, query, or fragment" \
  env "${common_env[@]}" XFLY_API_PREFIX=api/v1

run_rejected_case \
  "DEV database" \
  "x_fly is reserved for DEV" \
  env "${common_env[@]}" XFLY_TEST_DB_PORT=5434 XFLY_TEST_DB_NAME=x_fly

run_rejected_case \
  "ordinary local API port" \
  "127.0.0.1:8080 is not an approved Phase 1 target" \
  env "${common_env[@]}" XFLY_BASE_URL=http://127.0.0.1:8080

printf '%s\n' 'preflight guard tests passed'
