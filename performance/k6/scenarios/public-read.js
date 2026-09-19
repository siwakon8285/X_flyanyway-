import { check, sleep } from 'k6';
import http from 'k6/http';

import { createUrlBuilder } from '../../lib/api-url.js';
import { recordResponse } from '../lib/metrics.js';

const REQUEST_PARAMS = {
  timeout: '10s',
  headers: { Accept: 'application/json' },
};

const urlBuilder = createUrlBuilder({
  baseUrl: __ENV.XFLY_BASE_URL,
  apiPrefix: __ENV.XFLY_API_PREFIX,
});

function encode(value) {
  return encodeURIComponent(value);
}

function jsonBody(response) {
  if (response.status !== 200 || !response.body) {
    return null;
  }
  try {
    return response.json();
  } catch {
    return null;
  }
}

function getCase(config, index) {
  return config.manifest.cases[index % config.manifest.cases.length];
}

function getSearchCase(config, index) {
  return config.manifest.searchCases[index % config.manifest.searchCases.length];
}

export function runAirports(config) {
  const response = http.get(urlBuilder.api('/airports'), {
    ...REQUEST_PARAMS,
    tags: { endpoint: 'airports' },
  });
  const body = jsonBody(response);
  const passed = check(response, {
    'airports status is 200': (item) => item.status === 200,
    'airports response has reference rows': () => Array.isArray(body) && body.length >= config.manifest.airportMinimumCount,
  });
  recordResponse('airports', response, passed);
  return { response, passed };
}

export function runSearch(config, index) {
  const item = getSearchCase(config, index);
  const response = http.get(urlBuilder.api('/flights', {
    origin: item.origin,
    destination: item.destination,
    departure: config.manifest.departureDate,
    cabin: item.cabin,
  }), {
    ...REQUEST_PARAMS,
    tags: { endpoint: 'flight_search' },
  });
  const body = jsonBody(response);
  const passed = check(response, {
    'flight search status is 200': (value) => value.status === 200,
    'flight search response is an array': () => Array.isArray(body),
  });
  recordResponse('flight_search', response, passed);
  return { response, passed };
}

export function runDetail(config, index) {
  const item = getCase(config, index);
  const response = http.get(urlBuilder.api(`/flights/${encode(item.flightId)}`, {
    departure: config.manifest.departureDate,
    cabin: item.cabin,
  }), {
    ...REQUEST_PARAMS,
    tags: { endpoint: 'flight_detail' },
  });
  const body = jsonBody(response);
  const passed = check(response, {
    'flight detail status is 200': (value) => value.status === 200,
    'flight detail response is an object': () => body !== null && typeof body === 'object' && !Array.isArray(body),
  });
  recordResponse('flight_detail', response, passed);
  return { response, passed };
}

export function runSeatAvailability(config, index) {
  const item = getCase(config, index);
  const response = http.get(urlBuilder.api(`/flights/${encode(item.flightId)}/seats`, {
    departure: config.manifest.departureDate,
    cabin: item.cabin,
  }), {
    ...REQUEST_PARAMS,
    tags: { endpoint: 'seat_availability' },
  });
  const body = jsonBody(response);
  const passed = check(response, {
    'seat availability status is 200': (value) => value.status === 200,
    'seat availability response is an object': () => body !== null && typeof body === 'object' && !Array.isArray(body),
  });
  recordResponse('seat_availability', response, passed);
  return { response, passed };
}

export default function publicRead(config) {
  const selector = ((__VU * 1009) + __ITER) % 100;
  const index = (__VU + __ITER) % config.manifest.cases.length;

  if (selector < 10) {
    runAirports(config);
  } else if (selector < 50) {
    runSearch(config, index);
  } else if (selector < 75) {
    runDetail(config, index);
  } else {
    runSeatAvailability(config, index);
  }

  if (config.thinkTimeSeconds > 0) {
    sleep(config.thinkTimeSeconds);
  }
}
