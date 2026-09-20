import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { test } from 'node:test';
import vm from 'node:vm';
import { createUrlBuilder } from '../lib/api-url.js';

test('diagnostic schedule reaches 250 progressively and existing profiles are unchanged', async () => {
  const profile = JSON.parse(await readFile(new URL('../k6/endpoint-diagnostic-profile.json', import.meta.url)));
  assert.equal(profile.startVUs, 0);
  assert.equal(profile.executor, 'ramping-vus');
  assert.deepEqual(profile.stages, [
    { target: 200, duration: '30s' }, { target: 200, duration: '30s' },
    { target: 225, duration: '15s' }, { target: 225, duration: '30s' },
    { target: 240, duration: '15s' }, { target: 240, duration: '60s' },
    { target: 250, duration: '10s' }, { target: 250, duration: '45s' },
    { target: 0, duration: '20s' },
  ]);
  const original = await readFile(new URL('../k6/profiles.json', import.meta.url));
  assert.equal(createHash('sha256').update(original).digest('hex'), '289ac5eb31ed6f1b5035338dc944c70947a79fa56c9355dc8dcac58b8d1f97fc');
});

test('endpoint selection rejects missing/invalid names and selects only its runner', async () => {
  const { selectEndpoint } = await import('../lib/endpoint-diagnostic.js');
  const runners = { runSearch() {}, runDetail() {}, runSeatAvailability() {} };
  for (const [name, runner] of [['flight_search', runners.runSearch], ['flight_detail', runners.runDetail], ['seat_availability', runners.runSeatAvailability]]) {
    assert.equal(selectEndpoint(name, runners), runner);
  }
  for (const value of [undefined, '', 'airports', 'constructor', 'invalid']) {
    assert.throws(() => selectEndpoint(value, runners), /XFLY_DIAGNOSTIC_ENDPOINT/);
  }
});

test('failure details are limited to one per VU, including failed checks, and redact arbitrary text', async () => {
  const { createFailureRecorder } = await import('../lib/endpoint-diagnostic.js');
  const records = [];
  const record = createFailureRecorder('flight_detail', value => records.push(JSON.parse(value)));
  const response = { status: 200, error_code: 0, timings: { duration: 12 }, body: 'SECRET', headers: { token: 'SECRET' } };
  record(response, true, 1, 0);
  assert.equal(records.length, 0);
  record({ ...response, status: 0, error_code: 1220, error: 'read tcp 127.0.0.1:1: connection reset by peer' }, false, 1, 1);
  for (let i = 2; i < 1000; i++) record(response, false, 1, i);
  assert.deepEqual(records, [{ endpoint: 'flight_detail', __VU: 1, __ITER: 1, status: 0, error: 'connection reset by peer', error_code: 1220, duration: 12 }]);
  record({ ...response, error: 'postgres://user:SECRET@db/private?token=SECRET' }, false, 2, 0);
  assert.equal(records.length, 2);
  assert.equal(records[1].error, '[REDACTED]');
  assert.doesNotMatch(JSON.stringify(records), /SECRET|postgres:|headers|body|127\.0/);
});

test('real public-read dispatch retains 10/40/25/25 weights and diagnostic runners request only their endpoint', async () => {
  const source = await readFile(new URL('../k6/scenarios/public-read.js', import.meta.url), 'utf8');
  // Substitute only k6 boundaries; execute the actual request builders and dispatch.
  const requests = [];
  const context = vm.createContext({
    __ENV: { XFLY_BASE_URL: 'http://127.0.0.1:18080', XFLY_API_PREFIX: '/api/v1' },
    __VU: 1, __ITER: 0, createUrlBuilder,
    check: (response, checks) => Object.values(checks).every(check => check(response)),
    sleep() {}, recordResponse() {},
    http: { get(url, params) {
      requests.push({ url, params });
      return { status: 200, body: '[]', json: () => [], timings: { duration: 1 } };
    } },
    config: { thinkTimeSeconds: 0.1, manifest: {
      departureDate: '2027-01-01', airportMinimumCount: 1,
      cases: [{ flightId: 'test-flight', origin: 'BKK', destination: 'HKT', cabin: 'business' }],
      searchCases: [{ origin: 'BKK', destination: 'HKT', cabin: 'business' }],
    } },
  });
  vm.runInContext(source.replace(/^import .*;\n/gm, '').replace(/export default /g, '').replace(/export /g, ''), context);
  for (let iteration = 0; iteration < 100; iteration++) {
    context.__ITER = iteration;
    vm.runInContext('publicRead(config)', context);
  }
  const counts = {};
  for (const request of requests) counts[request.params.tags.endpoint] = (counts[request.params.tags.endpoint] || 0) + 1;
  assert.deepEqual(counts, { airports: 10, flight_search: 40, flight_detail: 25, seat_availability: 25 });
  const { selectEndpoint } = await import('../lib/endpoint-diagnostic.js');
  for (const [endpoint, expectedPath] of [
    ['flight_search', '/api/v1/flights'], ['flight_detail', '/api/v1/flights/test-flight'],
    ['seat_availability', '/api/v1/flights/test-flight/seats'],
  ]) {
    requests.length = 0;
    const runner = selectEndpoint(endpoint, context);
    const result = runner(context.config, 0);
    assert.equal(requests.length, 1);
    assert.equal(new URL(requests[0].url).pathname, expectedPath);
    assert.equal(requests[0].params.timeout, '10s');
    assert.equal(result.response.status, 200);
    assert.equal(typeof result.passed, 'boolean');
  }
});

test('k6 inspect emits one sanitized console record for repeated synthetic failures', () => {
  const result = spawnSync('k6', [
    'inspect',
    '--log-output', 'stderr',
    '--log-format', 'raw',
    new URL('./k6-failure-capture-check.js', import.meta.url).pathname,
  ], { encoding: 'utf8' });

  assert.equal(result.status, 0, result.stderr);

  const lines = result.stderr
    .trim()
    .split('\n')
    .filter((line) => line.startsWith('{"endpoint":'));

  assert.equal(lines.length, 1);

  assert.deepEqual(JSON.parse(lines[0]), {
    endpoint: 'flight_search',
    __VU: 1,
    __ITER: 0,
    status: 0,
    error: 'EOF',
    error_code: 1220,
    duration: 3,
  });
});
