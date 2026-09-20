import assert from 'node:assert/strict';
import { test } from 'node:test';
import { handleSummary, summarize } from '../k6/lib/summary.js';

test('summary persists configured executor and actual threshold results independently of error rates', () => {
  globalThis.__ENV = { XFLY_PROFILE: 'focused-250', XFLY_TARGET_ENV: 'TEST' };
  try {
    const data = { metrics: { http_req_failed: { values: { rate: 13 / 466134 }, thresholds: { 'rate<0.01': { ok: true } } } } };
    const result = summarize(data, { profile: 'focused-250', executor: 'ramping-vus' });
    assert.equal(result.scenario.executor, 'ramping-vus');
    assert.equal(result.thresholdsPassed, true);
    assert.equal(result.metrics.overall.httpErrorRate, 13 / 466134);
    data.metrics.http_req_failed.thresholds['rate<0.01'].ok = false;
    assert.equal(summarize(data).thresholdsPassed, false);
    assert.equal(summarize({ metrics: {} }).thresholdsPassed, null);
  } finally { delete globalThis.__ENV; }
});

test('summary markdown title uses the actual non-smoke profile', () => {
  globalThis.__ENV = { XFLY_PROFILE: 'focused-250', XFLY_TARGET_ENV: 'TEST' };
  try {
    const output = handleSummary({ metrics: {} }, { profile: 'focused-250', executor: 'ramping-vus' });
    assert.match(output.stdout, /^# X-Fly Phase 1 public-read — focused-250\n/);
    assert.doesNotMatch(output.stdout, /public-read smoke/);
  } finally { delete globalThis.__ENV; }
});
