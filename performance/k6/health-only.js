import { check } from 'k6';
import http from 'k6/http';
import { Rate } from 'k6/metrics';

import { createUrlBuilder } from '../lib/api-url.js';

const PROFILE_NAME = 'health-only-240';
const profile = JSON.parse(open('./health-only-profile.json'));

function requiredEnv(name) {
  const value = __ENV[name];
  if (!value || !value.trim()) {
    throw new Error(`${name} must be explicitly configured`);
  }
  return value.trim();
}

if (requiredEnv('XFLY_TARGET_ENV') !== 'TEST') {
  throw new Error('XFLY_TARGET_ENV must be TEST');
}
if (requiredEnv('XFLY_SAFETY_CONFIRM') !== 'TEST_ONLY') {
  throw new Error('XFLY_SAFETY_CONFIRM must equal TEST_ONLY');
}
if (__ENV.XFLY_HIGH_VU_CONFIRM !== 'TEST_ONLY') {
  throw new Error(`${PROFILE_NAME} requires XFLY_HIGH_VU_CONFIRM=TEST_ONLY`);
}

const urlBuilder = createUrlBuilder({
  baseUrl: requiredEnv('XFLY_BASE_URL'),
  apiPrefix: requiredEnv('XFLY_API_PREFIX'),
});
const healthUrl = urlBuilder.root('/health');
const timeoutRate = new Rate('health_timeout_rate');

export const options = {
  scenarios: {
    health_transport: profile,
  },
  summaryTrendStats: ['avg', 'min', 'med', 'p(90)', 'p(95)', 'p(99)', 'max'],
};

export default function healthOnly() {
  const response = http.get(healthUrl, {
    headers: { Accept: 'application/json' },
    tags: { endpoint: 'health' },
  });
  const passed = check(response, {
    'health status is 200': (value) => value.status === 200,
  });
  const errorCode = response.error_code ?? 0;
  timeoutRate.add(errorCode === 1050 || errorCode === 1211);
  return passed;
}
