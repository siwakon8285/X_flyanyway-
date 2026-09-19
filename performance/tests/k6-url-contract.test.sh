#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

inspection="$(k6 inspect \
  --env XFLY_BASE_URL=http://127.0.0.1:18080 \
  --env XFLY_API_PREFIX=/api/v1 \
  --env XFLY_TARGET_ENV=TEST \
  --env XFLY_SAFETY_CONFIRM=TEST_ONLY \
  --env XFLY_READ_MANIFEST="$repo_root/performance/data/public-read.example.json" \
  "$repo_root/performance/k6/run.js")"

k6_helper_inspection="$(k6 inspect \
  --env XFLY_BASE_URL=http://127.0.0.1:18080 \
  --env XFLY_API_PREFIX=/api/v1 \
  "$repo_root/performance/k6/url-contract-check.js")"

[[ "$inspection" == *'"public_read"'* ]] || {
  printf '%s\n' 'k6 inspect did not load the public-read scenario' >&2
  exit 1
}
[[ "$inspection" != *'/api/v1/api/v1'* ]] || {
  printf '%s\n' 'k6 inspect output contains a duplicated API prefix' >&2
  exit 1
}
[[ "$k6_helper_inspection" == *'"vus": 1'* ]] || {
  printf '%s\n' 'k6 did not execute the shared URL helper verification' >&2
  exit 1
}

printf '%s\n' 'k6 URL contract inspect passed'
