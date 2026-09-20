import { createUrlBuilder } from '../lib/api-url.js';

const scope = process.argv[2];
const endpoint = process.argv[3];

if (!['root', 'api'].includes(scope) || !endpoint) {
  throw new Error('usage: resolve-api-url.mjs <root|api> </endpoint>');
}

try {
  const builder = createUrlBuilder({
    baseUrl: process.env.XFLY_BASE_URL,
    apiPrefix: process.env.XFLY_API_PREFIX,
  });
  const url = scope === 'root'
    ? builder.root(endpoint)
    : builder.api(endpoint);
  process.stdout.write(`${url}\n`);
} catch (error) {
  process.stderr.write(`${error.message}\n`);
  process.exitCode = 1;
}
