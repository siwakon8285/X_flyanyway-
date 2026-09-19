const ENDPOINTS = ['airports', 'flight_search', 'flight_detail', 'seat_availability'];

function values(data, metricName) {
  return data?.metrics?.[metricName]?.values ?? {};
}

function numberOrNull(value) {
  return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

function percentile(metricValues, key) {
  return numberOrNull(metricValues[key]);
}

function sanitizeBaseUrl(value) {
  if (!value) return null;
  try {
    const parsed = new URL(value);
    return `${parsed.protocol}//${parsed.host}${parsed.pathname.replace(/\/$/, '')}`;
  } catch {
    return null;
  }
}

function sanitizeProfile(value) {
  const profile = String(value ?? '').trim();
  return /^[A-Za-z0-9._-]+$/.test(profile) ? profile : '[REDACTED]';
}

function summarizeEndpoint(data, endpoint) {
  const duration = values(data, `public_read_${endpoint}_duration`);
  const requests = values(data, `public_read_${endpoint}_requests`);
  const errors = values(data, `public_read_${endpoint}_error_rate`);
  const timeouts = values(data, `public_read_${endpoint}_timeout_rate`);
  const checks = values(data, `public_read_${endpoint}_check_rate`);
  return {
    requests: numberOrNull(requests.count),
    rps: numberOrNull(requests.rate),
    p50Ms: percentile(duration, 'med'),
    p95Ms: percentile(duration, 'p(95)'),
    p99Ms: percentile(duration, 'p(99)'),
    httpErrorRate: numberOrNull(errors.rate),
    timeoutRate: numberOrNull(timeouts.rate),
    checksRate: numberOrNull(checks.rate),
  };
}

export function summarize(data, scenario = {}) {
  const duration = values(data, 'http_req_duration');
  const requests = values(data, 'http_reqs');
  const errors = values(data, 'http_req_failed');
  const timeouts = values(data, 'http_timeout_rate');
  const checks = values(data, 'checks');
  const dropped = values(data, 'dropped_iterations');
  const profile = sanitizeProfile(scenario.profile || __ENV.XFLY_PROFILE || 'smoke');

  return {
    schemaVersion: 1,
    thresholdsPassed: (() => {
      const results = Object.values(data.metrics ?? {}).flatMap(metric => Object.values(metric.thresholds ?? {}).map(value => value.ok));
      return results.length && results.every(value => typeof value === 'boolean') ? results.every(Boolean) : null;
    })(),
    generatedAt: new Date().toISOString(),
    runId: __ENV.XFLY_RUN_ID || null,
    target: {
      environment: __ENV.XFLY_TARGET_ENV || null,
      baseUrl: sanitizeBaseUrl(__ENV.XFLY_BASE_URL),
      databaseName: 'x_fly_concurrency_test',
    },
    scenario: {
      name: 'public-read',
      ...scenario,
      profile,
      thinkTimeSeconds: Number(__ENV.XFLY_THINK_TIME_SECONDS || 0.1),
    },
    metrics: {
      overall: {
        requests: numberOrNull(requests.count),
        rps: numberOrNull(requests.rate),
        p50Ms: percentile(duration, 'med'),
        p95Ms: percentile(duration, 'p(95)'),
        p99Ms: percentile(duration, 'p(99)'),
        httpErrorRate: numberOrNull(errors.rate),
        timeoutRate: numberOrNull(timeouts.rate),
        checksRate: numberOrNull(checks.rate),
        droppedIterations: numberOrNull(dropped.count) ?? 0,
      },
      endpoints: Object.fromEntries(ENDPOINTS.map((endpoint) => [endpoint, summarizeEndpoint(data, endpoint)])),
    },
  };
}

function markdown(summary) {
  const overall = summary.metrics.overall;
  const lines = [
    `# X-Fly Phase 1 public-read — ${summary.scenario.profile}`,
    '',
    `- Run ID: ${summary.runId ?? 'not set'}`,
    `- Target: ${summary.target.environment ?? 'unknown'} / ${summary.target.baseUrl ?? 'unknown'}`,
    `- Profile: ${summary.scenario.profile}`,
    '',
    '| Scope | RPS | p50 ms | p95 ms | p99 ms | HTTP errors | Timeouts | Checks |',
    '|---|---:|---:|---:|---:|---:|---:|---:|',
    `| Overall | ${overall.rps ?? 'n/a'} | ${overall.p50Ms ?? 'n/a'} | ${overall.p95Ms ?? 'n/a'} | ${overall.p99Ms ?? 'n/a'} | ${overall.httpErrorRate ?? 'n/a'} | ${overall.timeoutRate ?? 'n/a'} | ${overall.checksRate ?? 'n/a'} |`,
    '',
    'Per-endpoint metrics are retained in summary.json.',
  ];
  return `${lines.join('\n')}\n`;
}

export function handleSummary(data, scenario) {
  const summary = summarize(data, scenario);
  const jsonPath = __ENV.XFLY_K6_SUMMARY_PATH || 'k6-summary.json';
  const markdownPath = __ENV.XFLY_K6_SUMMARY_MD_PATH || 'k6-summary.md';
  return {
    [jsonPath]: JSON.stringify(summary, null, 2),
    [markdownPath]: markdown(summary),
    stdout: markdown(summary),
  };
}
