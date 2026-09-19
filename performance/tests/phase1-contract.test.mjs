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

test('progressive profile contains the approved ordered VU plateaus', async () => {
  const profiles = await readJson('performance/k6/profiles.json');
  assert.deepEqual(
    profiles.progressive.stages.map((stage) => stage.target),
    [10, 10, 25, 25, 50, 50, 100, 100, 250, 250, 500, 500, 1000, 1000, 0],
  );
});

test('progressive profile uses explicit non-zero ramp and hold durations', async () => {
  const profiles = await readJson('performance/k6/profiles.json');
  assert.ok(profiles.progressive.stages.every((stage) => stage.duration));
  assert.equal(Math.max(...profiles.progressive.stages.map((stage) => stage.target)), 1000);
});

test('focused-250 profile matches the bounded saturation diagnostic stages', async () => {
  const profiles = await readJson('performance/k6/profiles.json');
  assert.deepEqual(profiles['focused-250'], {
    executor: 'ramping-vus',
    stages: [
      { target: 100, duration: '60s' },
      { target: 150, duration: '20s' },
      { target: 150, duration: '60s' },
      { target: 200, duration: '20s' },
      { target: 200, duration: '60s' },
      { target: 225, duration: '15s' },
      { target: 225, duration: '60s' },
      { target: 250, duration: '15s' },
      { target: 250, duration: '90s' },
      { target: 0, duration: '30s' },
    ],
    gracefulRampDown: '30s',
    gracefulStop: '30s',
  });
});

test('smoke, baseline, and progressive profile definitions remain unchanged', async () => {
  const profiles = await readJson('performance/k6/profiles.json');
  assert.deepEqual(profiles.smoke, {
    executor: 'constant-vus',
    vus: 1,
    duration: '30s',
  });
  assert.deepEqual(profiles.baseline, {
    executor: 'constant-vus',
    vus: 10,
    duration: '2m',
  });
  assert.deepEqual(profiles.progressive, {
    executor: 'ramping-vus',
    stages: [
      { target: 10, duration: '30s' },
      { target: 10, duration: '60s' },
      { target: 25, duration: '30s' },
      { target: 25, duration: '60s' },
      { target: 50, duration: '30s' },
      { target: 50, duration: '60s' },
      { target: 100, duration: '30s' },
      { target: 100, duration: '90s' },
      { target: 250, duration: '45s' },
      { target: 250, duration: '90s' },
      { target: 500, duration: '60s' },
      { target: 500, duration: '120s' },
      { target: 1000, duration: '90s' },
      { target: 1000, duration: '180s' },
      { target: 0, duration: '60s' },
    ],
    gracefulRampDown: '30s',
    gracefulStop: '30s',
  });
});

test('public-read manifest is TEST-only and contains independent read cases', async () => {
  const manifest = await readJson('performance/data/public-read.example.json');
  assert.equal(manifest.target.environment, 'TEST');
  assert.equal(manifest.target.databaseName, 'x_fly_concurrency_test');
  assert.ok(manifest.departureDate.match(/^\d{4}-\d{2}-\d{2}$/));
  assert.ok(manifest.cases.length >= 4);
  assert.ok(
    new Set(manifest.cases.map((item) => `${item.flightId}|${item.cabin}`)).size >= 4,
  );
});

test('public-read scenario contains only the approved GET endpoints', async () => {
  const source = await readText('performance/k6/scenarios/public-read.js');
  assert.match(source, /\/airports/);
  assert.match(source, /\/flights/);
  assert.match(source, /\/seats/);
  assert.doesNotMatch(source, /http\.(post|put|del|patch)\s*\(/);
  assert.doesNotMatch(source, /seat-holds|manage-booking|external|admin/i);
});

test('sanitized report renderer rejects secret-bearing output fields', async () => {
  const source = await readText('performance/scripts/render-report.mjs');
  assert.match(source, /authorization/i);
  assert.match(source, /password/i);
  assert.match(source, /cookie/i);
  assert.match(source, /redact|sanitize/i);
});
