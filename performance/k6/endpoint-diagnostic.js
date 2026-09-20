import { sleep } from 'k6';
import { loadConfig } from './lib/config.js';
import { runSearch, runDetail, runSeatAvailability } from './scenarios/public-read.js';
import { selectEndpoint, createFailureRecorder } from '../lib/endpoint-diagnostic.js';

const endpoint = __ENV.XFLY_DIAGNOSTIC_ENDPOINT;
const runEndpoint = selectEndpoint(endpoint, { runSearch, runDetail, runSeatAvailability });
if (__ENV.XFLY_HIGH_VU_CONFIRM !== 'TEST_ONLY') {
  throw new Error('endpoint diagnostic requires XFLY_HIGH_VU_CONFIRM=TEST_ONLY');
}
// Validate at init too: inspect exercises configuration without issuing requests.
const config = loadConfig();
const recordFailure = createFailureRecorder(endpoint, value => console.log(value));

export const options = {
  scenarios: { endpoint_diagnostic: JSON.parse(open('./endpoint-diagnostic-profile.json')) },
  tags: { diagnostic_endpoint: endpoint },
  summaryTrendStats: ['avg', 'min', 'med', 'p(90)', 'p(95)', 'p(99)', 'max'],
};

export default function diagnostic() {
  const index = (__VU + __ITER) % config.manifest.cases.length;
  const { response, passed } = runEndpoint(config, index);
  recordFailure(response, passed, __VU, __ITER);
  if (config.thinkTimeSeconds > 0) sleep(config.thinkTimeSeconds);
}
