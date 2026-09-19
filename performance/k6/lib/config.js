import { createUrlBuilder } from '../../lib/api-url.js';

const ALLOWED_CABINS = new Set(['business', 'first']);
const DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/;
const AIRPORT_PATTERN = /^[A-Z]{3}$/;
const FLIGHT_ID_PATTERN = /^[A-Za-z0-9-]+$/;

function requiredEnv(name) {
  const value = __ENV[name];
  if (!value || !value.trim()) {
    throw new Error(`${name} must be explicitly configured`);
  }
  return value.trim();
}

function validateCase(item, label) {
  if (!item || typeof item !== 'object') {
    throw new Error(`${label} must be an object`);
  }
  for (const field of ['origin', 'destination', 'cabin']) {
    if (typeof item[field] !== 'string' || !item[field].trim()) {
      throw new Error(`${label}.${field} is required`);
    }
  }
  if (!AIRPORT_PATTERN.test(item.origin) || !AIRPORT_PATTERN.test(item.destination) || item.origin === item.destination) {
    throw new Error(`${label} contains an invalid airport pair`);
  }
  if (!ALLOWED_CABINS.has(item.cabin)) {
    throw new Error(`${label}.cabin must be business or first`);
  }
}

function validateManifest(manifest) {
  if (manifest?.target?.environment !== 'TEST' || manifest?.target?.databaseName !== 'x_fly_concurrency_test') {
    throw new Error('read manifest must target TEST database x_fly_concurrency_test');
  }
  if (typeof manifest.departureDate !== 'string' || !DATE_PATTERN.test(manifest.departureDate)) {
    throw new Error('read manifest departureDate must use YYYY-MM-DD');
  }
  if (!Number.isInteger(manifest.airportMinimumCount) || manifest.airportMinimumCount < 1) {
    throw new Error('read manifest airportMinimumCount must be positive');
  }
  if (!Array.isArray(manifest.cases) || manifest.cases.length < 4) {
    throw new Error('read manifest must contain at least four read cases');
  }
  const keys = new Set();
  for (const [index, item] of manifest.cases.entries()) {
    validateCase(item, `cases[${index}]`);
    if (typeof item.flightId !== 'string' || !FLIGHT_ID_PATTERN.test(item.flightId)) {
      throw new Error(`cases[${index}].flightId is invalid`);
    }
    const key = `${item.flightId}|${item.cabin}`;
    if (keys.has(key)) {
      throw new Error(`duplicate read case ${key}`);
    }
    keys.add(key);
  }
  const searchCases = Array.isArray(manifest.searchCases) && manifest.searchCases.length > 0
    ? manifest.searchCases
    : manifest.cases.map(({ origin, destination, cabin }) => ({ origin, destination, cabin }));
  for (const [index, item] of searchCases.entries()) {
    validateCase(item, `searchCases[${index}]`);
  }
  return { ...manifest, searchCases };
}

const configuredManifest = __ENV.XFLY_READ_MANIFEST
  ? (() => {
      try {
        return JSON.parse(open(__ENV.XFLY_READ_MANIFEST));
      } catch {
        throw new Error('XFLY_READ_MANIFEST must point to valid JSON');
      }
    })()
  : null;

export function loadConfig() {
  if (__ENV.XFLY_TARGET_ENV !== 'TEST') {
    throw new Error('XFLY_TARGET_ENV must be TEST');
  }
  if (__ENV.XFLY_SAFETY_CONFIRM !== 'TEST_ONLY') {
    throw new Error('XFLY_SAFETY_CONFIRM must equal TEST_ONLY');
  }

  const profile = __ENV.XFLY_PROFILE || 'smoke';
  if ((profile === 'progressive' || profile === 'focused-250')
      && __ENV.XFLY_HIGH_VU_CONFIRM !== 'TEST_ONLY') {
    throw new Error(`${profile} profile requires XFLY_HIGH_VU_CONFIRM=TEST_ONLY`);
  }

  requiredEnv('XFLY_READ_MANIFEST');
  if (!configuredManifest) {
    throw new Error('XFLY_READ_MANIFEST must be explicitly configured');
  }

  const urlBuilder = createUrlBuilder({
    baseUrl: requiredEnv('XFLY_BASE_URL'),
    apiPrefix: requiredEnv('XFLY_API_PREFIX'),
  });

  return {
    baseUrl: urlBuilder.baseUrl,
    apiPrefix: urlBuilder.apiPrefix,
    profile,
    thinkTimeSeconds: parseThinkTime(__ENV.XFLY_THINK_TIME_SECONDS),
    manifest: validateManifest(configuredManifest),
  };
}

function parseThinkTime(value) {
  const seconds = value === undefined || value === '' ? 0.1 : Number(value);
  if (!Number.isFinite(seconds) || seconds < 0 || seconds > 5) {
    throw new Error('XFLY_THINK_TIME_SECONDS must be between 0 and 5');
  }
  return seconds;
}
