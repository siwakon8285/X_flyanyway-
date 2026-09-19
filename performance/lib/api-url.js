const HTTP_PROTOCOLS = new Set(['http:', 'https:']);
const API_PREFIX_PATTERN = /^\/[A-Za-z0-9._~-]+(?:\/[A-Za-z0-9._~-]+)*$/;

function requiredValue(value, name) {
  if (typeof value !== 'string' || value.trim() === '') {
    throw new Error(`${name} must be explicitly configured`);
  }
  return value.trim();
}

function normalizeBaseUrl(value) {
  const raw = requiredValue(value, 'XFLY_BASE_URL');

  if (raw.includes('?') || raw.includes('#') || raw.includes('@')) {
    throw new Error('XFLY_BASE_URL must be an HTTP(S) URL without credentials');
  }

  const normalizedRaw = raw.replace(/\/+$/, '');
  const match = normalizedRaw.match(/^(https?):\/\/([^/?#@]+)(.*)$/i);
  if (!match) {
    throw new Error('XFLY_BASE_URL must be a valid HTTP(S) URL');
  }

  const protocol = `${match[1].toLowerCase()}:`;
  const authority = match[2];
  if (match[3] !== '') {
    throw new Error('XFLY_BASE_URL must contain only scheme, host, and optional port');
  }
  if (!HTTP_PROTOCOLS.has(protocol) || authority.length === 0 || /\s/.test(authority)) {
    throw new Error('XFLY_BASE_URL must be an HTTP(S) URL without credentials');
  }

  const validHost = /^\[[0-9A-Fa-f:.]+\](?::\d{1,5})?$/.test(authority)
    || /^[A-Za-z0-9.-]+(?::\d{1,5})?$/.test(authority);
  if (!validHost) {
    throw new Error('XFLY_BASE_URL must be an HTTP(S) URL without credentials');
  }

  return `${protocol}//${authority.toLowerCase()}`;
}

function normalizeApiPrefix(value) {
  const raw = requiredValue(value, 'XFLY_API_PREFIX');
  if (raw.startsWith('//') || !raw.startsWith('/') || raw.includes('?') || raw.includes('#') || raw.includes('\\')) {
    throw new Error('XFLY_API_PREFIX must be a relative path without credentials, query, or fragment');
  }

  const normalized = raw.replace(/\/+$/, '');
  if (!API_PREFIX_PATTERN.test(normalized)) {
    throw new Error('XFLY_API_PREFIX must be a relative path without credentials, query, or fragment');
  }

  return normalized;
}

function normalizeEndpoint(endpoint) {
  if (typeof endpoint !== 'string' || !endpoint.startsWith('/') || endpoint.startsWith('//')) {
    throw new Error('endpoint path must be a single relative path beginning with /');
  }
  if (endpoint.includes('?') || endpoint.includes('#') || endpoint.includes('\\')) {
    throw new Error('endpoint path must not contain a query string or fragment');
  }
  if (endpoint.includes('//')) {
    throw new Error('endpoint path must not contain duplicate slashes');
  }

  const normalized = endpoint.replace(/\/+$/, '');
  return normalized || '/';
}

function withQuery(url, query) {
  if (query === undefined || query === null) return url;
  if (typeof query !== 'object' || Array.isArray(query)) {
    throw new Error('query parameters must be an object');
  }

  const pairs = [];
  for (const [key, value] of Object.entries(query)) {
    if (value !== undefined && value !== null) {
      pairs.push(`${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`);
    }
  }
  return pairs.length > 0 ? `${url}?${pairs.join('&')}` : url;
}

function joinPath(prefix, endpoint) {
  const normalizedEndpoint = normalizeEndpoint(endpoint);
  return normalizedEndpoint === '/'
    ? prefix
    : `${prefix}${normalizedEndpoint}`;
}

export function createUrlBuilder({ baseUrl, apiPrefix } = {}) {
  const normalizedBaseUrl = normalizeBaseUrl(baseUrl);
  const normalizedApiPrefix = normalizeApiPrefix(apiPrefix);

  function build(path, query, prefix) {
    const targetPath = prefix
      ? joinPath(normalizedApiPrefix, path)
      : normalizeEndpoint(path);
    return withQuery(`${normalizedBaseUrl}${targetPath}`, query);
  }

  return Object.freeze({
    baseUrl: normalizedBaseUrl,
    apiPrefix: normalizedApiPrefix,
    root(path, query) {
      return build(path, query, false);
    },
    api(path, query) {
      return build(path, query, true);
    },
  });
}
