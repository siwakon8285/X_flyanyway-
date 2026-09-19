# Phase 1 result artifacts

Each run is written to `performance/reports/runs/<run-id>/`, which is ignored
by Git. The committed artifacts are the sanitized `report.md` and
`summary.json` formats produced by the renderer; raw event streams are not
required for Phase 1.

Reports identify the TEST target, profile, commit, request metrics, checks,
timeouts, dropped iterations, host observations, and PostgreSQL connection
observations. They must not contain database URLs, passwords, authorization
headers, cookies, response bodies, or query-string-bearing URLs.

See [Branch 28 closeout](branch-28-closeout.md) for the completed investigation,
operator-supplied evidence, and limits on interpretation.

Report evaluation now separates completion, observed functional errors, and actual
k6 threshold results. A clean functional PASS requires a recorded exit 0, positive
request count, and complete zero-error/zero-timeout/all-checks-passed metrics.
Nonzero rates remain visible at their stored precision. A threshold can pass even
when small functional error rates are nonzero; it is not a clean functional PASS.

`run-public-read.sh` records `k6-exit-code.txt`, as the endpoint diagnostic wrapper
already does. Nonzero exits are conservatively labeled aborted-or-unsuccessful
(including threshold exit failures); missing/empty completion evidence is unknown.
Legacy reports without exit evidence cannot be retroactively labeled completed.
Exit 0 alone does not independently verify every scheduled iteration was executed;
operator interruption evidence takes precedence. SIGKILL can prevent rendering.

New public-read summaries persist their configured executor and actual threshold
results. Legacy/native summaries resolve an explicitly identified profile from its
JSON definition; `executorSource` distinguishes recorded metadata from that fallback.
Unknown profiles remain unknown. Native diagnostic summaries are supported when
`XFLY_PROFILE=endpoint-diagnostic` is supplied to the renderer; absent threshold
results remain UNKNOWN. The renderer does not execute workloads or alter evidence.
