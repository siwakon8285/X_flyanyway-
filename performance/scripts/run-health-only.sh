#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
profile='health-only-240'
export XFLY_PROFILE="$profile"

"$repo_root/performance/scripts/health-only-preflight.sh"

run_id="${XFLY_HEALTH_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-${profile}}"
if [[ ! "$run_id" =~ ^[A-Za-z0-9._-]+$ ]]; then
  printf '%s\n' 'Health-only run ID may contain only letters, numbers, dot, underscore, and hyphen' >&2
  exit 1
fi

run_directory="${XFLY_RESULT_DIR:-$repo_root/performance/reports/runs}/$run_id"
mkdir -p "$run_directory"
printf '%s\n' "${XFLY_API_PID:-}" >"$run_directory/api.pid"
: >"$run_directory/k6.pid"
printf 'profile=%s\n' "$profile" >"$run_directory/metadata.txt"
printf 'target_environment=%s\n' "$XFLY_TARGET_ENV" >>"$run_directory/metadata.txt"
printf 'git_commit=%s\n' "$(git -C "$repo_root" rev-parse HEAD)" >>"$run_directory/metadata.txt"
printf 'git_branch=%s\n' "$(git -C "$repo_root" branch --show-current)" >>"$run_directory/metadata.txt"

node "$repo_root/performance/scripts/observe-health-only.mjs" "$run_directory" >"$run_directory/observer.log" 2>&1 &
observer_pid=$!

stop_observer() {
  kill "$observer_pid" 2>/dev/null || true
  wait "$observer_pid" 2>/dev/null || true
}
trap stop_observer EXIT

set +e
(
  cd "$repo_root"
  exec k6 run \
    --env XFLY_TARGET_ENV="$XFLY_TARGET_ENV" \
    --env XFLY_SAFETY_CONFIRM="$XFLY_SAFETY_CONFIRM" \
    --env XFLY_BASE_URL="$XFLY_BASE_URL" \
    --env XFLY_API_PREFIX="$XFLY_API_PREFIX" \
    --env XFLY_HIGH_VU_CONFIRM="$XFLY_HIGH_VU_CONFIRM" \
    --summary-export "$run_directory/k6-summary.json" \
    performance/k6/health-only.js
) &
k6_pid=$!
printf '%s\n' "$k6_pid" >"$run_directory/k6.pid"
wait "$k6_pid"
k6_status=$?
set -e

stop_observer
trap - EXIT
exit "$k6_status"
