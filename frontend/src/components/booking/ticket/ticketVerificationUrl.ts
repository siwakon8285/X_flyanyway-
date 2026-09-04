/**
 * Resolves the frontend base origin safely:
 * - Environment override (NEXT_PUBLIC_SITE_URL or NEXT_PUBLIC_APP_URL)
 * - Browser: window.location.origin (supports localhost, custom ports, and deployed domain automatically)
 * - Fallback: "http://localhost:3000"
 */
function resolveFrontendOrigin(): string {
  if (process.env.NEXT_PUBLIC_SITE_URL) {
    return process.env.NEXT_PUBLIC_SITE_URL.replace(/\/+$/, "");
  }
  if (process.env.NEXT_PUBLIC_APP_URL) {
    return process.env.NEXT_PUBLIC_APP_URL.replace(/\/+$/, "");
  }
  if (typeof window !== "undefined" && window.location?.origin) {
    return window.location.origin;
  }
  return "http://localhost:3000";
}

/**
 * Builds the full public verification URL encoded into the boarding pass QR code.
 * Format: <frontend-origin>/ticket/verify/<signed-token>
 * Strictly zero PII: contains only origin, static path, and the cryptographic signed token.
 */
function buildTicketVerificationUrl(
  token: string,
  explicitOrigin?: string,
): string {
  const origin = (explicitOrigin ?? resolveFrontendOrigin()).replace(
    /\/+$/,
    "",
  );
  return `${origin}/ticket/verify/${encodeURIComponent(token)}`;
}

export { buildTicketVerificationUrl, resolveFrontendOrigin };
