import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

const root = new URL('../../', import.meta.url);
const renderer = new URL('../scripts/render-report.mjs', import.meta.url);

test('report renderer emits sanitized machine and human artifacts', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'x-fly-phase1-report-'));
  try {
    await writeFile(join(directory, 'k6-exit-code.txt'), '0\n');
    await writeFile(join(directory, 'k6-summary.json'), JSON.stringify({
      scenario: { profile: 'smoke', thinkTimeSeconds: 0.1 },
      metrics: {
        overall: {
          requests: 20,
          rps: 10,
          p50Ms: 12,
          p95Ms: 20,
          p99Ms: 25,
          httpErrorRate: 0,
          timeoutRate: 0,
          checksRate: 1,
          droppedIterations: 0,
        },
        endpoints: {
          airports: {
            requests: 20,
            rps: 10,
            p50Ms: 12,
            p95Ms: 20,
            p99Ms: 25,
            httpErrorRate: 0,
            timeoutRate: 0,
            checksRate: 1,
          },
        },
      },
    }));
    await writeFile(join(directory, 'host.csv'), [
      'timestamp,cpu_percent,load1,mem_used_bytes,mem_total_bytes',
      '2026-09-18T00:00:00Z,10,1,100,200',
      '2026-09-18T00:00:05Z,20,2,120,200',
    ].join('\n'));
    await writeFile(join(directory, 'postgres.csv'), [
      'timestamp,total_connections,active_connections,waiting_sessions,blocked_sessions,max_connections,observer_status',
      '2026-09-18T00:00:00Z,3,1,0,0,100,ok',
      '2026-09-18T00:00:05Z,4,2,0,0,100,ok',
    ].join('\n'));

    execFileSync(process.execPath, [renderer.pathname, directory], {
      cwd: new URL('../../', import.meta.url).pathname,
      env: {
        ...process.env,
        XFLY_RUN_ID: 'run-password-token',
        XFLY_BASE_URL: 'http://127.0.0.1:18080',
        XFLY_API_PREFIX: '/api/v1',
        XFLY_TEST_DB_HOST: '127.0.0.1',
        XFLY_TEST_DB_PORT: '5434',
        XFLY_PROFILE: 'smoke',
      },
      stdio: 'pipe',
    });

    const summary = JSON.parse(await readFile(join(directory, 'summary.json'), 'utf8'));
    const report = await readFile(join(directory, 'report.md'), 'utf8');
    assert.equal(summary.metrics.overall.p95Ms, 20);
    assert.equal(summary.target.databaseName, 'x_fly_concurrency_test');
    assert.equal(summary.evaluation.functionalPass, true);
    assert.equal(summary.runId, '[REDACTED]');
    assert.doesNotMatch(JSON.stringify(summary), /run-password-token/);
    assert.doesNotMatch(report, /run-password-token/);
    assert.match(report, /TEST-only evidence/);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

async function renderCase({ exitCode, profile = 'focused-250', overall = {}, native = false } = {}) {
  const directory = await mkdtemp(join(tmpdir(), 'xfly-report-semantics-'));
  try {
    const metrics = { requests: 466134, httpErrorRate: 0, timeoutRate: 0, checksRate: 1, ...overall };
    await writeFile(join(directory, 'k6-summary.json'), JSON.stringify(native ? {
      metrics: { http_reqs: { count: 100, rate: 10 }, http_req_failed: { value: 0 },
        http_timeout_rate: { value: 0 }, checks: { value: 1 }, http_req_duration: { med: 1, 'p(95)': 2, 'p(99)': 3 } },
    } : { scenario: { profile }, metrics: { overall: metrics, endpoints: {} } }));
    if (exitCode !== undefined) await writeFile(join(directory, 'k6-exit-code.txt'), `${exitCode}\n`);
    await writeFile(join(directory, 'host.csv'), 'timestamp,cpu_percent\n');
    await writeFile(join(directory, 'postgres.csv'), 'timestamp,total_connections\n');
    execFileSync(process.execPath, [renderer.pathname, directory], {
      env: { PATH: process.env.PATH, XFLY_PROFILE: profile, XFLY_BASE_URL: 'http://127.0.0.1:18080?token=secret' }, stdio: 'pipe',
    });
    return { summary: JSON.parse(await readFile(join(directory, 'summary.json'), 'utf8')),
      report: await readFile(join(directory, 'report.md'), 'utf8') };
  } finally { await rm(directory, { recursive: true, force: true }); }
}

test('interrupted and legacy unknown-completion runs never claim functional PASS', async () => {
  for (const exitCode of [130, 143, 99, undefined]) {
    const { summary, report } = await renderCase({ exitCode });
    assert.equal(summary.evaluation.functionalPass, false);
    assert.doesNotMatch(report, /Functional result: PASS/);
    assert.notEqual(summary.evaluation.completion, 'completed');
  }
});

test('small HTTP/check errors fail clean-functional evaluation and retain visible precision', async () => {
  for (const overall of [{ httpErrorRate: 13 / 466134 }, { checksRate: 1 - 1 / 466134 }]) {
    const { summary, report } = await renderCase({ exitCode: 0, overall });
    assert.equal(summary.evaluation.functionalPass, false);
    assert.match(report, /Functional result: FAIL/);
    assert.ok(report.includes(String(overall.httpErrorRate ?? overall.checksRate)));
    assert.doesNotMatch(report, /token=secret/);
  }
});

test('missing metrics cannot become zero or a clean pass', async () => {
  const { summary } = await renderCase({ exitCode: 0, overall: { httpErrorRate: null } });
  assert.equal(summary.metrics.overall.httpErrorRate, null);
  assert.equal(summary.evaluation.functionalPass, false);
});

test('staged profile executors are derived from profile definitions, including native diagnostics', async () => {
  for (const profile of ['focused-250', 'progressive', 'endpoint-diagnostic']) {
    const { summary } = await renderCase({ exitCode: 0, profile, native: profile === 'endpoint-diagnostic' });
    assert.equal(summary.scenario.executor, 'ramping-vus');
    assert.equal(summary.evaluation.functionalPass, true);
  }
});
