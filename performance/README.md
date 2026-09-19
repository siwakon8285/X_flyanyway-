# Branch 28 Phase 1 performance tooling

Branch 28 investigation is closed. See [evidence and interpretation](reports/branch-28-closeout.md).
The mixed-workload failure root cause remains unproven; no production capacity or
issue-resolution claim is made. Commands below are retained for future explicitly
authorized work, not instructions to continue the investigation now.

Phase 1 measures the existing public REST read surface with k6:

- `GET /api/v1/airports`
- `GET /api/v1/flights`
- `GET /api/v1/flights/{flight_id}`
- `GET /api/v1/flights/{flight_id}/seats`

The only permitted database target is the disposable TEST database
`x_fly_concurrency_test` on `127.0.0.1:5434`. DEV `127.0.0.1:5433` and
`x_fly` are rejected by the preflight. Phase 1 does not migrate, seed, reset,
truncate, or clean up any database. Seat availability is prewarmed with
read-only HTTP requests that may materialize inventory only in TEST.

## Local configuration

Copy `data/public-read.example.json` to an ignored local manifest and update
the departure date or cases only with data verified in TEST. Then configure
these variables in the shell without printing them:

```bash
export XFLY_TARGET_ENV=TEST
export XFLY_SAFETY_CONFIRM=TEST_ONLY
export XFLY_BASE_URL=http://127.0.0.1:18080
export XFLY_API_PREFIX=/api/v1
export XFLY_TEST_DB_HOST=127.0.0.1
export XFLY_TEST_DB_PORT=5434
export XFLY_TEST_DB_NAME=x_fly_concurrency_test
export XFLY_READ_MANIFEST="$PWD/performance/data/public-read.local.json"
: "${XFLY_TEST_DATABASE_URL:?set the TEST observer URL in the environment}"
: "${XFLY_TEST_RUNTIME_DATABASE_URL:?set the TEST runtime URL in the environment}"
```

The API must be started separately with `DATABASE_URL` set to the TEST runtime
URL and with a dedicated bind port:

```bash
cd backend
DATABASE_URL="$XFLY_TEST_RUNTIME_DATABASE_URL" \\
BACKEND_BIND_ADDRESS=127.0.0.1:18080 \\
cargo run --locked --bin x-fly-api
```

The runner never sources or prints `.env`. `/health` is a root endpoint;
public API endpoints are built from `XFLY_BASE_URL` plus `XFLY_API_PREFIX`.

## Health-only transport diagnostic

`health-only-240` is a separate TEST-only diagnostic. It sends only `GET
/health`, uses k6's default HTTP connection reuse, and does not read a manifest,
connect to PostgreSQL, prewarm seats, or invoke the public-read preflight. Its
stages are 0 to 240 VUs over 60 seconds, 240 VUs for 60 seconds, then 0 VUs
over 30 seconds.

Run it only after the API is already bound to the dedicated TEST port:

```bash
XFLY_TARGET_ENV=TEST \
XFLY_SAFETY_CONFIRM=TEST_ONLY \
XFLY_BASE_URL=http://127.0.0.1:18080 \
XFLY_API_PREFIX=/api/v1 \
XFLY_HIGH_VU_CONFIRM=TEST_ONLY \
./performance/scripts/run-health-only.sh
```

`XFLY_API_PID` is optional. If supplied, the observer records it; otherwise it
tries to identify the listener on local TCP port 18080 with `lsof`. The run
directory contains the native k6 summary, `host.csv`, `processes.csv`, PID
files, and an observer log. The run directory is ignored by Git and no
database URL or credential is written by this runner.

On macOS, inspect the listener queue for port 18080 in a separate terminal:

```bash
netstat -anv -p tcp | grep -E '127\.0\.0\.1\.18080|\.18080 '
```

The `Recv-Q` and `Send-Q` columns are the queue fields to record alongside the
run. Identify process IDs and approximate open descriptor counts with:

```bash
API_PID="$(lsof -nP -a -tiTCP:18080 -sTCP:LISTEN | head -n 1)"
K6_PID="$(cat performance/reports/runs/<run-id>/k6.pid)"
lsof -nP -p "$API_PID" | tail -n +2 | wc -l
lsof -nP -p "$K6_PID" | tail -n +2 | wc -l
```

Capture host CPU and load with:

```bash
uptime
top -l 1 -n 0 | grep -E 'Load Avg|CPU usage|PhysMem'
```

## Runs

### Single-endpoint diagnostic

After configuring the normal TEST environment and verified local manifest above,
run one endpoint at a time:

```bash
XFLY_DIAGNOSTIC_ENDPOINT=flight_search \
XFLY_HIGH_VU_CONFIRM=TEST_ONLY \
bash performance/scripts/run-endpoint-diagnostic.sh
```

The other accepted values are `flight_detail` and `seat_availability`. The wrapper
inherits the full TEST identity/manifest preflight. Its health and airport probes
occur before measurement. Only a seat-availability diagnostic invokes the existing
controlled TEST seat prewarm before measurement; that GET can materialize inventory.
No diagnostic has been executed as part of implementation verification.

The separate profile starts at zero and runs: 200/30s ramp, 200/30s hold,
225/15s ramp, 225/30s hold, 240/15s ramp, 240/60s hold, 250/10s ramp,
250/45s hold, 0/20s ramp (255 seconds plus up to 30 seconds graceful stop).
It reuses existing request parameters, checks, endpoint metrics and think time.
Use the wrapper for execution; `k6 inspect` validates configuration without the
wrapper's live database identity check and does not authorize a run.

Each invocation prints a unique ignored directory under `performance/reports/runs`:

- `k6.log`: at most one sanitized JSON failure record per VU (at most 250).
  Fields: endpoint, `__VU`, `__ITER`, status, error, error_code, duration in ms.
  Check failures with HTTP 200 are included. Known transport error labels survive;
  other error text is `[REDACTED]`. Bodies, headers, URLs and arbitrary error text
  are excluded. The first failure uses each VU's allowance for the whole run.
- `k6-summary.json`: native k6 summary, including existing per-endpoint metrics.
- `k6-output.log` and `k6-exit-code.txt`: console progress/summary and process status.
- `host.csv`, `postgres.csv`, `observer.log`: unchanged existing observer outputs.

The wrapper suppresses k6 engine logging to prevent unbounded request warnings and
raw URL/error leakage; script console output is redirected separately to `k6.log`.
Consequently engine diagnostics are unavailable; consult the exit code for run
failure. No PASS/FAIL verdict is inferred and the existing report renderer is not
invoked. A failed check may have no transport error/code. Sampling omits later
failure types from a VU, so counts must come from the metrics, not log line counts.

```bash
PROFILE=smoke ./performance/scripts/run-public-read.sh
PROFILE=baseline ./performance/scripts/run-public-read.sh
PROFILE=focused-250 XFLY_HIGH_VU_CONFIRM=TEST_ONLY ./performance/scripts/run-public-read.sh
PROFILE=progressive XFLY_HIGH_VU_CONFIRM=TEST_ONLY ./performance/scripts/run-public-read.sh
```

The focused-250 profile is a bounded diagnostic ramp from 100 to 250 VUs. It
requires the same explicit high-VU TEST confirmation as the progressive
profile and is intended for an authorized diagnostic run only.

The progressive profile is intentionally ordered as 10, 25, 50, 100, 250,
500, and 1,000 VUs with a hold at every plateau. Phase 1 stops after the
one-VU smoke unless a later run is explicitly authorized.
