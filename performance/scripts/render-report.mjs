import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';

const runDirectory = process.argv[2];
if (!runDirectory) throw new Error('report renderer requires a run directory');

const SENSITIVE_FIELD_PATTERN = /authorization|password|cookie|secret|token/i;

function sanitizeReportValue(value) {
  const text = String(value ?? '');
  if (SENSITIVE_FIELD_PATTERN.test(text) || /postgres(?:ql)?:\/\//i.test(text)) {
    return '[REDACTED]';
  }
  return text.replace(/([?&](?:password|secret|token|authorization|cookie)=)[^&\s]*/gi, '$1[REDACTED]');
}

async function readJson(path) {
  return JSON.parse(await readFile(path, 'utf8'));
}

function safeNumber(value) {
  if (value === null || value === undefined || value === '') return null;
  const number = Number(value);
  return Number.isFinite(number) ? number : null;
}

function parseCsv(text) {
  const lines = text.trim().split('\n').filter(Boolean);
  if (lines.length < 2) return [];
  const headers = lines[0].split(',');
  return lines.slice(1).map((line) => {
    const values = line.split(',');
    return Object.fromEntries(headers.map((header, index) => [header, values[index] ?? '']));
  });
}

function stats(rows, field) {
  const values = rows.map((row) => safeNumber(row[field])).filter((value) => value !== null);
  if (!values.length) return { min: null, avg: null, max: null };
  return {
    min: Math.min(...values),
    avg: values.reduce((sum, value) => sum + value, 0) / values.length,
    max: Math.max(...values),
  };
}

function sanitizedBaseUrl(value) {
  if (!value) return null;
  try {
    const parsed = new URL(value);
    return `${parsed.protocol}//${parsed.host}${parsed.pathname.replace(/\/$/, '')}`;
  } catch {
    return null;
  }
}

function gitValue(args) {
  try {
    return execFileSync('git', args, { encoding: 'utf8' }).trim();
  } catch {
    return null;
  }
}

function sanitizedK6Metrics(k6) {
  const native = k6.metrics ?? {};
  const rate = name => native[name]?.rate ?? native[name]?.value;
  const overall = k6?.metrics?.overall ?? {
    requests: native.http_reqs?.count, rps: native.http_reqs?.rate,
    p50Ms: native.http_req_duration?.med, p95Ms: native.http_req_duration?.['p(95)'],
    p99Ms: native.http_req_duration?.['p(99)'], httpErrorRate: rate('http_req_failed'),
    timeoutRate: rate('http_timeout_rate') ?? rate('health_timeout_rate'), checksRate: rate('checks'),
    droppedIterations: native.dropped_iterations?.count,
  };
  const endpoints = k6?.metrics?.endpoints ?? {};
  return {
    overall: {
      requests: safeNumber(overall.requests),
      rps: safeNumber(overall.rps),
      p50Ms: safeNumber(overall.p50Ms),
      p95Ms: safeNumber(overall.p95Ms),
      p99Ms: safeNumber(overall.p99Ms),
      httpErrorRate: safeNumber(overall.httpErrorRate),
      timeoutRate: safeNumber(overall.timeoutRate),
      checksRate: safeNumber(overall.checksRate),
      droppedIterations: safeNumber(overall.droppedIterations) ?? 0,
    },
    endpoints: Object.fromEntries(Object.entries(endpoints).filter(([name]) =>
      ['airports', 'flight_search', 'flight_detail', 'seat_availability'].includes(name)).map(([name, values]) => [name, {
      requests: safeNumber(values.requests),
      rps: safeNumber(values.rps),
      p50Ms: safeNumber(values.p50Ms),
      p95Ms: safeNumber(values.p95Ms),
      p99Ms: safeNumber(values.p99Ms),
      httpErrorRate: safeNumber(values.httpErrorRate),
      timeoutRate: safeNumber(values.timeoutRate),
      checksRate: safeNumber(values.checksRate),
    }])),
  };
}

function isFunctionalPass(metrics) {
  const overall = metrics.overall;
  return overall.httpErrorRate !== null
    && overall.timeoutRate !== null
    && overall.checksRate !== null
    && overall.requests > 0
    && overall.httpErrorRate === 0
    && overall.timeoutRate === 0
    && overall.checksRate === 1
    && Object.values(metrics.endpoints).every(value =>
      !(value.httpErrorRate > 0 || value.timeoutRate > 0 || (value.checksRate !== null && value.checksRate < 1)));
}

function format(value, digits = 2) {
  return value === null || value === undefined ? 'n/a' : digits === 4 ? String(value) : Number(value).toFixed(digits);
}

function renderMarkdown(summary) {
  const overall = summary.metrics.overall;
  const endpointRows = Object.entries(summary.metrics.endpoints).map(([name, values]) =>
    `| ${name} | ${format(values.rps)} | ${format(values.p50Ms)} | ${format(values.p95Ms)} | ${format(values.p99Ms)} | ${format(values.httpErrorRate, 4)} | ${format(values.timeoutRate, 4)} | ${format(values.checksRate, 4)} |`,
  );
  const host = summary.observations.host;
  const postgres = summary.observations.postgres;
  return [
    '# X-Fly Branch 28 Phase 1 — Public Read',
    '',
    `- Run ID: ${summary.runId}`,
    `- Git: ${summary.git.commit ?? 'unknown'} (${summary.git.branch ?? 'unknown'})`,
    `- Target: ${summary.target.environment} / ${summary.target.baseUrl ?? 'unknown'}`,
    `- Database: ${summary.target.databaseName} on TEST port ${summary.target.databasePort}`,
    `- Profile: ${summary.scenario.profile}; executor: ${summary.scenario.executor}`,
    `- Completion: ${summary.evaluation.completion}; k6 exit: ${summary.evaluation.exitCode ?? 'unknown'}`,
    `- Functional result: ${summary.evaluation.result}`,
    `- Observed functional errors: ${summary.evaluation.observedErrors ? 'YES' : 'none in available metrics'}`,
    `- Threshold evaluation: ${summary.evaluation.thresholdsPassed === null ? 'UNKNOWN' : summary.evaluation.thresholdsPassed ? 'PASS' : 'FAIL'}`,
    '',
    '| Scope | RPS | p50 ms | p95 ms | p99 ms | HTTP errors | Timeouts | Checks |',
    '|---|---:|---:|---:|---:|---:|---:|---:|',
    `| Overall | ${format(overall.rps)} | ${format(overall.p50Ms)} | ${format(overall.p95Ms)} | ${format(overall.p99Ms)} | ${format(overall.httpErrorRate, 4)} | ${format(overall.timeoutRate, 4)} | ${format(overall.checksRate, 4)} |`,
    ...endpointRows,
    '',
    '## Lightweight observations',
    '',
    `- Host CPU percent: min ${format(host.cpuPercent.min)} / avg ${format(host.cpuPercent.avg)} / max ${format(host.cpuPercent.max)}`,
    `- Host RAM used bytes: min ${format(host.memUsedBytes.min, 0)} / avg ${format(host.memUsedBytes.avg, 0)} / max ${format(host.memUsedBytes.max, 0)}`,
    `- PostgreSQL total connections: min ${format(postgres.totalConnections.min, 0)} / avg ${format(postgres.totalConnections.avg, 0)} / max ${format(postgres.totalConnections.max, 0)}`,
    `- PostgreSQL active connections: min ${format(postgres.activeConnections.min, 0)} / avg ${format(postgres.activeConnections.avg, 0)} / max ${format(postgres.activeConnections.max, 0)}`,
    `- PostgreSQL waiting sessions: max ${format(postgres.waitingSessions.max, 0)}`,
    `- PostgreSQL blocked sessions: max ${format(postgres.blockedSessions.max, 0)}`,
    '',
    'These measurements are TEST-only evidence for the explicitly identified target. They are not production capacity or an official SLO.',
    '',
  ].join('\n');
}

const k6 = await readJson(`${runDirectory}/k6-summary.json`);
const hostRows = parseCsv(await readFile(`${runDirectory}/host.csv`, 'utf8'));
const postgresRows = parseCsv(await readFile(`${runDirectory}/postgres.csv`, 'utf8'));
const metrics = sanitizedK6Metrics(k6);
const profile = k6.scenario?.profile || process.env.XFLY_PROFILE || 'unknown';
const profiles = await readJson(new URL('../k6/profiles.json', import.meta.url));
profiles['endpoint-diagnostic'] = await readJson(new URL('../k6/endpoint-diagnostic-profile.json', import.meta.url));
profiles['health-only-240'] = await readJson(new URL('../k6/health-only-profile.json', import.meta.url));
let exitCode = null;
try {
  const value = (await readFile(`${runDirectory}/k6-exit-code.txt`, 'utf8')).trim();
  if (/^\d+$/.test(value)) exitCode = Number(value);
} catch (error) { if (error.code !== 'ENOENT') throw error; }
const completion = exitCode === 0 ? 'completed' : exitCode === null ? 'unknown' : 'aborted-or-unsuccessful';
const observedErrors = [metrics.overall, ...Object.values(metrics.endpoints)].some(value =>
  value.httpErrorRate > 0 || value.timeoutRate > 0 || (value.checksRate !== null && value.checksRate < 1));
const functionalPass = completion === 'completed' && isFunctionalPass(metrics);
const thresholdsPassed = typeof k6.thresholdsPassed === 'boolean' ? k6.thresholdsPassed : null;
const summary = {
  schemaVersion: 1,
  generatedAt: new Date().toISOString(),
  runId: sanitizeReportValue(process.env.XFLY_RUN_ID || 'unidentified-run'),
  git: {
    commit: sanitizeReportValue(process.env.XFLY_GIT_COMMIT || gitValue(['rev-parse', 'HEAD'])),
    branch: sanitizeReportValue(process.env.XFLY_GIT_BRANCH || gitValue(['branch', '--show-current'])),
  },
  target: {
    environment: 'TEST',
    baseUrl: sanitizedBaseUrl(process.env.XFLY_BASE_URL),
    databaseHost: sanitizeReportValue(process.env.XFLY_TEST_DB_HOST || ''),
    databasePort: safeNumber(process.env.XFLY_TEST_DB_PORT),
    databaseName: 'x_fly_concurrency_test',
  },
  scenario: {
    name: profile === 'endpoint-diagnostic' ? 'endpoint-diagnostic' : 'public-read',
    profile: sanitizeReportValue(profile),
    executor: sanitizeReportValue(k6.scenario?.executor || profiles[profile]?.executor || 'unknown'),
    executorSource: k6.scenario?.executor ? 'recorded' : 'configured-profile',
    thinkTimeSeconds: safeNumber(process.env.XFLY_THINK_TIME_SECONDS || k6.scenario?.thinkTimeSeconds) ?? 0.1,
  },
  metrics,
  observations: {
    host: {
      cpuPercent: stats(hostRows, 'cpu_percent'),
      memUsedBytes: stats(hostRows, 'mem_used_bytes'),
      memTotalBytes: stats(hostRows, 'mem_total_bytes'),
      load1: stats(hostRows, 'load1'),
    },
    postgres: {
      totalConnections: stats(postgresRows, 'total_connections'),
      activeConnections: stats(postgresRows, 'active_connections'),
      waitingSessions: stats(postgresRows, 'waiting_sessions'),
      blockedSessions: stats(postgresRows, 'blocked_sessions'),
      maxConnections: stats(postgresRows, 'max_connections'),
      observerStatuses: [...new Set(postgresRows.map((row) => row.observer_status).filter(Boolean))],
    },
  },
  evaluation: {
    functionalPass, completion, exitCode, observedErrors, thresholdsPassed,
    result: functionalPass ? 'PASS' : completion !== 'completed' ? 'INCOMPLETE / UNKNOWN' : observedErrors ? 'FAIL' : 'UNKNOWN',
    note: 'TEST-only evidence; not a production capacity claim or official SLO.',
  },
};

await writeFile(`${runDirectory}/summary.json`, `${JSON.stringify(summary, null, 2)}\n`);
await writeFile(`${runDirectory}/report.md`, `${renderMarkdown(summary)}\n`);
