export function selectEndpoint(endpoint, runners) {
  switch (endpoint) {
    case 'flight_search': return runners.runSearch;
    case 'flight_detail': return runners.runDetail;
    case 'seat_availability': return runners.runSeatAvailability;
    default: throw new Error('XFLY_DIAGNOSTIC_ENDPOINT must be flight_search, flight_detail, or seat_availability');
  }
}

// Only fixed transport labels escape this boundary, never arbitrary error text.
function safeError(error) {
  if (typeof error !== 'string' || !error) return null;
  const labels = ['connection reset by peer', 'connection refused', 'unexpected EOF', 'EOF',
    'request timeout', 'i/o timeout', 'context deadline exceeded', 'broken pipe',
    'too many open files', 'cannot assign requested address'];
  return labels.find(label => error.includes(label)) || '[REDACTED]';
}

function number(value) {
  return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

export function createFailureRecorder(endpoint, emit) {
  selectEndpoint(endpoint, {});
  const recordedVUs = new Set();
  return (response, passed, vu, iteration) => {
    if (recordedVUs.has(vu)) return;
    if (passed && response.status === 200 && !response.error_code && !response.error) return;
    recordedVUs.add(vu);
    emit(JSON.stringify({
      endpoint, __VU: number(vu), __ITER: number(iteration),
      status: number(response.status), error: safeError(response.error),
      error_code: number(response.error_code), duration: number(response.timings?.duration),
    }));
  };
}
