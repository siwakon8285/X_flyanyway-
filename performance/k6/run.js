import { handleSummary as writeSummary } from './lib/summary.js';
import { loadConfig } from './lib/config.js';
import publicRead from './scenarios/public-read.js';

const profiles = JSON.parse(open('./profiles.json'));
const profileName = __ENV.XFLY_PROFILE || 'smoke';
const profile = profiles[profileName];

if (!profile) {
  throw new Error(`unknown XFLY_PROFILE: ${profileName}`);
}
if ((profileName === 'progressive' || profileName === 'focused-250')
    && __ENV.XFLY_HIGH_VU_CONFIRM !== 'TEST_ONLY') {
  throw new Error(`${profileName} profile requires XFLY_HIGH_VU_CONFIRM=TEST_ONLY`);
}

const functionalThresholds = profileName === 'smoke'
  ? {
      http_req_failed: ['rate<0.0001'],
      http_timeout_rate: ['rate<0.0001'],
      checks: ['rate>0.9999'],
    }
  : {
      http_req_failed: ['rate<0.01'],
      http_timeout_rate: ['rate<0.001'],
      checks: ['rate>0.99'],
    };

export const options = {
  scenarios: {
    public_read: profile,
  },
  thresholds: functionalThresholds,
  summaryTrendStats: ['avg', 'min', 'med', 'p(90)', 'p(95)', 'p(99)', 'max'],
};

export function setup() {
  return loadConfig();
}

export default function run(config) {
  publicRead(config);
}

export function handleSummary(data) {
  return writeSummary(data, { profile: profileName, executor: profile.executor });
}
