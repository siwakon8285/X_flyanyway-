// Inspection-only fixture: no HTTP imports or requests.

import { createFailureRecorder } from '../lib/endpoint-diagnostic.js';

const record = createFailureRecorder(
  'flight_search',
  value => console.log(value),
);

{
  for (let iteration = 0; iteration < 10; iteration++) {
    record(
      {
        status: 0,
        error: 'EOF',
        error_code: 1220,
        timings: { duration: 3 },
      },
      false,
      1,
      iteration,
    );
  }
}

export default function unused() {}
