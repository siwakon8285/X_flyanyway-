import assert from 'node:assert/strict';
import { test } from 'node:test';

const apiUrlModule = new URL('../lib/api-url.js', import.meta.url);

async function loadBuilder() {
  const module = await import(apiUrlModule);
  return module.createUrlBuilder;
}

test('root-only base and API prefix build the expected endpoint URL', async () => {
  const createUrlBuilder = await loadBuilder();
  const builder = createUrlBuilder({
    baseUrl: 'http://127.0.0.1:18080',
    apiPrefix: '/api/v1',
  });

  assert.equal(builder.root('/health'), 'http://127.0.0.1:18080/health');
  assert.equal(builder.api('/airports'), 'http://127.0.0.1:18080/api/v1/airports');
});

test('trailing slashes normalize without duplicating the API prefix', async () => {
  const createUrlBuilder = await loadBuilder();
  const builder = createUrlBuilder({
    baseUrl: 'http://127.0.0.1:18080/',
    apiPrefix: '/api/v1/',
  });

  const url = builder.api('/airports');

  assert.equal(url, 'http://127.0.0.1:18080/api/v1/airports');
  assert.doesNotMatch(url, /\/api\/v1\/api\/v1/);
});

test('API query parameters are added to the shared URL without changing its prefix', async () => {
  const createUrlBuilder = await loadBuilder();
  const builder = createUrlBuilder({
    baseUrl: 'http://127.0.0.1:18080',
    apiPrefix: '/api/v1',
  });

  assert.equal(
    builder.api('/flights/xf-201/seats', {
      departure: '2026-10-18',
      cabin: 'business',
    }),
    'http://127.0.0.1:18080/api/v1/flights/xf-201/seats?departure=2026-10-18&cabin=business',
  );
});

test('missing base or API prefix is rejected explicitly', async () => {
  const createUrlBuilder = await loadBuilder();

  assert.throws(
    () => createUrlBuilder({ baseUrl: '', apiPrefix: '/api/v1' }),
    /XFLY_BASE_URL must be explicitly configured/,
  );
  assert.throws(
    () => createUrlBuilder({ baseUrl: 'http://127.0.0.1:18080', apiPrefix: '' }),
    /XFLY_API_PREFIX must be explicitly configured/,
  );
});

test('base paths and malformed API prefixes are rejected', async () => {
  const createUrlBuilder = await loadBuilder();

  assert.throws(
    () => createUrlBuilder({
      baseUrl: 'http://127.0.0.1:18080/api/v1',
      apiPrefix: '/api/v1',
    }),
    /XFLY_BASE_URL must contain only scheme, host, and optional port/,
  );
  for (const apiPrefix of ['api/v1', '//api/v1', '/api/v1?token=secret', 'http://api/v1']) {
    assert.throws(
      () => createUrlBuilder({
        baseUrl: 'http://127.0.0.1:18080',
        apiPrefix,
      }),
      /XFLY_API_PREFIX must be a relative path without credentials, query, or fragment/,
    );
  }
});
