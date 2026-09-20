#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
[[ "${XFLY_HIGH_VU_CONFIRM:-}" == TEST_ONLY ]] || {
  printf '%s\n' 'endpoint diagnostic requires XFLY_HIGH_VU_CONFIRM=TEST_ONLY' >&2
  exit 1
}
case "${XFLY_DIAGNOSTIC_ENDPOINT:-}" in
  flight_search|flight_detail|seat_availability) ;;
  *) printf '%s\n' 'XFLY_DIAGNOSTIC_ENDPOINT must be flight_search, flight_detail, or seat_availability' >&2; exit 1 ;;
esac
export XFLY_PROFILE=endpoint-diagnostic
bash "$repo_root/performance/scripts/preflight.sh"
if [[ "$XFLY_DIAGNOSTIC_ENDPOINT" == seat_availability ]]; then
  node "$repo_root/performance/scripts/prewarm-seat-inventory.mjs"
fi

# Always unique, always under the existing ignored artifact directory.
mkdir -p "$repo_root/performance/reports/runs"
run_directory="$(mktemp -d "$repo_root/performance/reports/runs/endpoint-${XFLY_DIAGNOSTIC_ENDPOINT}-XXXXXXXX")"
printf 'Diagnostic artifacts: %s\n' "$run_directory"
node "$repo_root/performance/scripts/observe.mjs" "$run_directory" >"$run_directory/observer.log" 2>&1 &
observer_pid=$!
stop_observer() {
  kill "$observer_pid" 2>/dev/null || true
  wait "$observer_pid" 2>/dev/null || true
}
trap stop_observer EXIT

set +e
(cd "$repo_root" && k6 run --include-system-env-vars \
  --log-format raw --log-output none \
  --console-output "$run_directory/k6.log" \
  --summary-export "$run_directory/k6-summary.json" \
  performance/k6/endpoint-diagnostic.js) >"$run_directory/k6-output.log" 2>&1
k6_status=$?
set -e
printf '%s\n' "$k6_status" >"$run_directory/k6-exit-code.txt"
exit "$k6_status"
