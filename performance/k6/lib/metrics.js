import { Counter, Rate, Trend } from 'k6/metrics';

export const timeoutRate = new Rate('http_timeout_rate');

const endpointMetricNames = {
  airports: 'airports',
  flight_search: 'flight_search',
  flight_detail: 'flight_detail',
  seat_availability: 'seat_availability',
};

export const endpointMetrics = Object.fromEntries(
  Object.entries(endpointMetricNames).map(([endpoint, metricName]) => [endpoint, {
    requests: new Counter(`public_read_${metricName}_requests`),
    duration: new Trend(`public_read_${metricName}_duration`, true),
    errors: new Rate(`public_read_${metricName}_error_rate`),
    timeouts: new Rate(`public_read_${metricName}_timeout_rate`),
    checks: new Rate(`public_read_${metricName}_check_rate`),
  }]),
);

export function recordResponse(endpoint, response, checksPassed) {
  const metrics = endpointMetrics[endpoint];
  if (!metrics) {
    throw new Error(`unknown public-read endpoint: ${endpoint}`);
  }
  const errorCode = response.error_code ?? 0;
  const timeout = errorCode === 1050 || errorCode === 1211;
  const error = response.status !== 200 || errorCode !== 0;
  const duration = response.timings?.duration ?? 0;

  timeoutRate.add(timeout, { endpoint });
  metrics.requests.add(1);
  metrics.duration.add(duration);
  metrics.errors.add(error);
  metrics.timeouts.add(timeout);
  metrics.checks.add(Boolean(checksPassed));
}
