#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
manifest="$repo_root/performance/data/public-read.example.json"

k6_args=(
  inspect
  --env XFLY_PROFILE=focused-250
  --env XFLY_BASE_URL=http://127.0.0.1:18080
  --env XFLY_API_PREFIX=/api/v1
  --env XFLY_TARGET_ENV=TEST
  --env XFLY_SAFETY_CONFIRM=TEST_ONLY
  --env XFLY_READ_MANIFEST="$manifest"
  "$repo_root/performance/k6/run.js"
)

set +e
k6_missing_confirmation="$(k6 "${k6_args[@]}" 2>&1)"
k6_missing_status=$?
set -e

if [[ "$k6_missing_status" -eq 0 ]]; then
  printf '%s\n' 'focused-250 k6 inspect unexpectedly succeeded without confirmation' >&2
  exit 1
fi
if [[ "$k6_missing_confirmation" != *'focused-250 profile requires XFLY_HIGH_VU_CONFIRM=TEST_ONLY'* ]]; then
  printf '%s\n' "$k6_missing_confirmation" >&2
  exit 1
fi

k6 --env XFLY_HIGH_VU_CONFIRM=TEST_ONLY "${k6_args[@]}" >/dev/null

set +e
wrapper_missing_confirmation="$(
  env -i \
    PATH="$PATH" \
    PROFILE=focused-250 \
    XFLY_PROFILE=focused-250 \
    bash "$repo_root/performance/scripts/run-public-read.sh" 2>&1
)"
wrapper_missing_status=$?
set -e

if [[ "$wrapper_missing_status" -eq 0 ]]; then
  printf '%s\n' 'focused-250 wrapper unexpectedly continued without confirmation' >&2
  exit 1
fi
if [[ "$wrapper_missing_confirmation" != *'refusing focused-250 profile without XFLY_HIGH_VU_CONFIRM=TEST_ONLY'* ]]; then
  printf '%s\n' "$wrapper_missing_confirmation" >&2
  exit 1
fi
