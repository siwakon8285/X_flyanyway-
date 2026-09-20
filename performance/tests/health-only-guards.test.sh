#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
health_script="$repo_root/performance/k6/health-only.js"

k6_args=(
  inspect
  --env XFLY_BASE_URL=http://127.0.0.1:18080
  --env XFLY_API_PREFIX=/api/v1
  --env XFLY_TARGET_ENV=TEST
  --env XFLY_SAFETY_CONFIRM=TEST_ONLY
  "$health_script"
)

set +e
k6_missing_confirmation="$(k6 "${k6_args[@]}" 2>&1)"
k6_missing_status=$?
set -e

if [[ "$k6_missing_status" -eq 0 ]]; then
  printf '%s\n' 'health-only k6 inspect unexpectedly succeeded without high-VU confirmation' >&2
  exit 1
fi
if [[ "$k6_missing_confirmation" != *'health-only-240 requires XFLY_HIGH_VU_CONFIRM=TEST_ONLY'* ]]; then
  printf '%s\n' "$k6_missing_confirmation" >&2
  exit 1
fi

k6_output="$(k6 --env XFLY_HIGH_VU_CONFIRM=TEST_ONLY "${k6_args[@]}")"
[[ "$k6_output" == *'health_transport'* ]] || {
  printf '%s\n' 'health-only k6 inspect did not load the health transport scenario' >&2
  exit 1
}

set +e
wrapper_missing_confirmation="$(
  env -i \
    PATH="$PATH" \
    XFLY_TARGET_ENV=TEST \
    XFLY_SAFETY_CONFIRM=TEST_ONLY \
    XFLY_BASE_URL=http://127.0.0.1:18080 \
    XFLY_API_PREFIX=/api/v1 \
    bash "$repo_root/performance/scripts/run-health-only.sh" 2>&1
)"
wrapper_missing_status=$?
set -e

if [[ "$wrapper_missing_status" -eq 0 ]]; then
  printf '%s\n' 'health-only wrapper unexpectedly continued without high-VU confirmation' >&2
  exit 1
fi
if [[ "$wrapper_missing_confirmation" != *'health-only-240 requires XFLY_HIGH_VU_CONFIRM=TEST_ONLY'* ]]; then
  printf '%s\n' "$wrapper_missing_confirmation" >&2
  exit 1
fi

printf '%s\n' 'health-only safety guard tests passed'
