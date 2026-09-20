# Branch 28 investigation closeout

Evidence below was supplied by the operator at closeout. These runs were not
rerun during the reporting cleanup; no further diagnostics are required for
this branch. Approximate values retain their original precision.

| Workload | Peak VUs | Requests | RPS | p50 ms | p95 ms | p99 ms | HTTP failures | Timeouts |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Health only | 240 | 8,669,859 | — | — | 6.59 | 9.03 | 0 | 0 |
| Flight detail only | 250 | 486,141 | ~1,905.93 | 4.976 | 26.51 | 45.45 | 0 | 0 |
| Flight search only | 250 | 464,690 | ~1,821.63 | 8.2 | 39.8 | 60.31 | 0 | 0 |
| Earlier mixed focused-250 | — | 466,134 | — | — | — | — | ~13 | 0 |

Both isolated public endpoint runs recorded zero failed checks, zero interrupted
iterations, and k6 exit 0. Health-only host CPU averaged 98.78%, peaking at
99.98%; reported API/k6 FD maxima were 255/253. Those observer counts use lsof
rows, which include entries other than numeric file descriptors; they are not
proof of a process FD ceiling.

The mixed run recorded approximately five search, six detail, two seat-availability,
and zero airport HTTP failures. It was manually interrupted shortly after failures
appeared. Its old report's “Functional result: PASS” and “constant-vus” labels were
incorrect. PostgreSQL observations were about 19 total connections against a maximum
of 100, with active connections at most five in the observed tail and no sustained
blocking. Sampling cannot rule out brief pressure between observations.

The earlier mixed-workload failures were not reproduced by health-only,
flight-detail-only, or flight-search-only diagnostics at 240–250 VUs. No single
root cause has been proven. This evidence does not establish a generic ~240
connection ceiling, a backend 256-FD limit, PostgreSQL max-connection exhaustion,
or any specific endpoint as the root cause. An isolated seat result is not
available here. The mixed workload issue is not claimed fixed.

These localhost runs share the load generator, API, PostgreSQL, loopback network,
and host scheduling. They are not production-capacity evidence and do not support
a 100,000-user or 100,000-RPS capacity claim. Separate-machine testing remains
future work, not a prerequisite for this closeout. Performance investigation stops
here; the reusable TEST-only tooling and bounded failure capture remain available.
