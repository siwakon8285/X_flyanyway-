import { createUrlBuilder } from '../lib/api-url.js';

const builder = createUrlBuilder({
  baseUrl: __ENV.XFLY_BASE_URL,
  apiPrefix: __ENV.XFLY_API_PREFIX,
});

const expected = {
  health: 'http://127.0.0.1:18080/health',
  airports: 'http://127.0.0.1:18080/api/v1/airports',
  seats: 'http://127.0.0.1:18080/api/v1/flights/xf-201/seats?departure=2026-10-18&cabin=business',
};

const actual = {
  health: builder.root('/health'),
  airports: builder.api('/airports'),
  seats: builder.api('/flights/xf-201/seats', {
    departure: '2026-10-18',
    cabin: 'business',
  }),
};

for (const [name, expectedUrl] of Object.entries(expected)) {
  if (actual[name] !== expectedUrl) {
    throw new Error(`${name} URL contract mismatch`);
  }
}

if (Object.values(actual).some((url) => url.includes('/api/v1/api/v1'))) {
  throw new Error('URL contract contains a duplicated API prefix');
}

export const options = { vus: 1, duration: '1s' };

export default function () {}
