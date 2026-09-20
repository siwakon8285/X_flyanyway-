import { readFile } from 'node:fs/promises';

import { createUrlBuilder } from '../lib/api-url.js';

function fail(message) {
  throw new Error(message);
}

function required(name) {
  const value = process.env[name];
  if (!value || !value.trim()) fail(`${name} must be explicitly configured`);
  return value.trim();
}

function validateManifest(manifest) {
  if (manifest?.target?.environment !== 'TEST' || manifest?.target?.databaseName !== 'x_fly_concurrency_test') {
    fail('read manifest must target TEST database x_fly_concurrency_test');
  }
  if (!/^\d{4}-\d{2}-\d{2}$/.test(manifest.departureDate)) {
    fail('read manifest departureDate must use YYYY-MM-DD');
  }
  if (!Array.isArray(manifest.cases) || manifest.cases.length < 4) {
    fail('read manifest must contain at least four read cases');
  }
  return manifest.cases;
}

function validateTarget() {
  if (process.env.XFLY_TARGET_ENV !== 'TEST') fail('XFLY_TARGET_ENV must be TEST');
  if (process.env.XFLY_SAFETY_CONFIRM !== 'TEST_ONLY') fail('XFLY_SAFETY_CONFIRM must equal TEST_ONLY');
  if (process.env.XFLY_TEST_DB_HOST !== '127.0.0.1' || process.env.XFLY_TEST_DB_PORT !== '5434') {
    fail('seat prewarm requires TEST target 127.0.0.1:5434');
  }
  if (process.env.XFLY_TEST_DB_NAME !== 'x_fly_concurrency_test') {
    fail('seat prewarm refuses any database other than x_fly_concurrency_test');
  }
}

validateTarget();
const urlBuilder = createUrlBuilder({
  baseUrl: required('XFLY_BASE_URL'),
  apiPrefix: required('XFLY_API_PREFIX'),
});
const manifestPath = required('XFLY_READ_MANIFEST');
const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
const cases = validateManifest(manifest);
const uniqueCases = [...new Map(cases.map((item) => [`${item.flightId}|${item.cabin}`, item])).values()];

for (const item of uniqueCases) {
  const url = urlBuilder.api(`/flights/${encodeURIComponent(item.flightId)}/seats`, {
    departure: manifest.departureDate,
    cabin: item.cabin,
  });
  let response;
  try {
    response = await fetch(url, {
      headers: { Accept: 'application/json' },
      signal: AbortSignal.timeout(10_000),
    });
    await response.arrayBuffer();
  } catch {
    fail('seat inventory prewarm request failed');
  }
  if (response.status !== 200) {
    fail(`seat inventory prewarm returned HTTP ${response.status}`);
  }
}

console.log(`prewarmed ${uniqueCases.length} TEST seat-map cases`);
