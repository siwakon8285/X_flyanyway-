#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

fail() {
  printf 'Health-only Phase 1 preflight refused: %s\n' "$1" >&2
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

[[ "$XFLY_TARGET_ENV" == TEST ]] || fail 'XFLY_TARGET_ENV must be TEST'
[[ "$XFLY_SAFETY_CONFIRM" == TEST_ONLY ]] || fail 'XFLY_SAFETY_CONFIRM must equal TEST_ONLY'
[[ "${XFLY_HIGH_VU_CONFIRM:-}" == TEST_ONLY ]] || fail 'health-only-240 requires XFLY_HIGH_VU_CONFIRM=TEST_ONLY'

if [[ -n "${XFLY_API_PID:-}" && ! "$XFLY_API_PID" =~ ^[0-9]+$ ]]; then
  fail 'XFLY_API_PID must be a numeric process ID when provided'
fi

case "$XFLY_BASE_URL" in
  http://*|https://*) ;;
  *) fail 'XFLY_BASE_URL must be an HTTP(S) URL' ;;
esac
if [[ "$XFLY_BASE_URL" =~ ^https?://(127\.0\.0\.1|localhost):8080(/|$) ]]; then
  fail '127.0.0.1:8080 is not an approved Phase 1 target'
fi

command -v node >/dev/null 2>&1 || fail 'node is required'
command -v k6 >/dev/null 2>&1 || fail 'k6 is required'

if ! node "$repo_root/performance/scripts/resolve-api-url.mjs" root /health >/dev/null; then
  fail 'API URL contract is invalid'
fi

printf '%s\n' 'Health-only Phase 1 preflight passed for TEST target'
