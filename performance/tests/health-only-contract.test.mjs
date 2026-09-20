import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';

const root = new URL('../../', import.meta.url);

async function readText(relativePath) {
  return readFile(new URL(relativePath, root), 'utf8');
}

async function readJson(relativePath) {
  return JSON.parse(await readText(relativePath));
}

test('health-only profile is the bounded 0-to-240 VU transport diagnostic', async () => {
  const profile = await readJson('performance/k6/health-only-profile.json');

  assert.deepEqual(profile, {
    executor: 'ramping-vus',
    startVUs: 0,
    stages: [
      { target: 240, duration: '60s' },
      { target: 240, duration: '60s' },
      { target: 0, duration: '30s' },
    ],
    gracefulRampDown: '30s',
    gracefulStop: '30s',
  });
});

test('health-only k6 scenario makes exactly a root health GET without fixture access', async () => {
  const source = await readText('performance/k6/health-only.js');

  assert.match(source, /createUrlBuilder/);
  assert.match(source, /root\('\/health'\)/);
  assert.match(source, /http\.get\(healthUrl/);
  assert.doesNotMatch(source, /\.api\(/);
  assert.doesNotMatch(source, /\/airports|\/flights|\/seats|http\.(post|put|patch|del)\s*\(/);
  assert.doesNotMatch(source, /XFLY_READ_MANIFEST|open\([^)]*manifest|prewarm|psql/i);
  assert.doesNotMatch(source, /no-connection-reuse|no-vu-connection-reuse|noVUConnectionReuse/);
});

test('health-only scenario retains the TEST and high-VU safety gates', async () => {
  const source = await readText('performance/k6/health-only.js');

  assert.match(source, /XFLY_TARGET_ENV/);
  assert.match(source, /XFLY_SAFETY_CONFIRM/);
  assert.match(source, /XFLY_HIGH_VU_CONFIRM/);
  assert.match(source, /TEST_ONLY/);
});

test('health-only runner does not invoke public-read preflight, prewarm, or database access', async () => {
  const source = await readText('performance/scripts/run-health-only.sh');

  assert.match(source, /health-only\.js/);
  assert.match(source, /XFLY_HIGH_VU_CONFIRM/);
  assert.doesNotMatch(source, /run-public-read\.sh|prewarm-seat-inventory|\/preflight\.sh|XFLY_READ_MANIFEST|XFLY_TEST_DATABASE_URL|\bpsql\b/);
  assert.doesNotMatch(source, /no-connection-reuse|no-vu-connection-reuse/);
});

test('health-only observer records PIDs, descriptor counts, and host CPU/load without PostgreSQL access', async () => {
  const source = await readText('performance/scripts/observe-health-only.mjs');

  assert.match(source, /api_pid/);
  assert.match(source, /k6_pid/);
  assert.match(source, /fd_count/);
  assert.match(source, /loadavg/);
  assert.match(source, /cpuPercent/);
  assert.doesNotMatch(source, /psql|postgres/i);
});

test('health-only documentation explains macOS listener queue inspection', async () => {
  const source = await readText('performance/README.md');

  assert.match(source, /health-only/i);
  assert.match(source, /18080/);
  assert.match(source, /netstat/);
  assert.match(source, /lsof/);
  assert.match(source, /Recv-Q|Send-Q/);
});

test('local public-read manifests remain ignored', async () => {
  const source = await readText('.gitignore');
  assert.match(source, /performance\/data\/\*\.local\.json/);
});
