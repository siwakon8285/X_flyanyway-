#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
profile="${PROFILE:-${XFLY_PROFILE:-smoke}}"
export XFLY_PROFILE="$profile"

if [[ "$profile" == progressive || "$profile" == focused-250 ]] \
  && [[ "${XFLY_HIGH_VU_CONFIRM:-}" != TEST_ONLY ]]; then
  printf '%s\n' "refusing $profile profile without XFLY_HIGH_VU_CONFIRM=TEST_ONLY" >&2
  exit 1
fi

run_id="${XFLY_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-${profile}}"
export XFLY_RUN_ID="$run_id"
run_directory="${XFLY_RESULT_DIR:-$repo_root/performance/reports/runs}/$run_id"
mkdir -p "$run_directory"

"$repo_root/performance/scripts/preflight.sh"
node "$repo_root/performance/scripts/prewarm-seat-inventory.mjs"

export XFLY_GIT_COMMIT="$(git -C "$repo_root" rev-parse HEAD)"
export XFLY_GIT_BRANCH="$(git -C "$repo_root" branch --show-current)"
export XFLY_K6_SUMMARY_PATH="$run_directory/k6-summary.json"
export XFLY_K6_SUMMARY_MD_PATH="$run_directory/k6-summary.md"

node "$repo_root/performance/scripts/observe.mjs" "$run_directory" >"$run_directory/observer.log" 2>&1 &
observer_pid=$!

stop_observer() {
  kill "$observer_pid" 2>/dev/null || true
  wait "$observer_pid" 2>/dev/null || true
}
trap stop_observer EXIT

interrupted_status=0
trap 'interrupted_status=130' INT
trap 'interrupted_status=143' TERM
# Clear stale completion evidence before starting a run in a reused directory.
: >"$run_directory/k6-exit-code.txt"
set +e
(cd "$repo_root" && k6 run performance/k6/run.js)
k6_status=$?
if [[ "$interrupted_status" -ne 0 ]]; then k6_status="$interrupted_status"; fi
printf '%s\n' "$k6_status" >"$run_directory/k6-exit-code.txt"
set -e

stop_observer
trap - EXIT

node "$repo_root/performance/scripts/render-report.mjs" "$run_directory"
exit "$k6_status"
