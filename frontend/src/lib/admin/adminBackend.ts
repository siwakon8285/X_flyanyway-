import { parseStaffPrincipal } from "@/lib/admin/adminPrincipal";
import type { StaffPrincipal } from "@/lib/admin/adminTypes";

const API_URL =
  process.env.X_FLY_INTERNAL_API_URL ??
  process.env.NEXT_PUBLIC_X_FLY_API_URL ??
  "http://localhost:8080/api/v1";

const allowedPaths = new Set([
  "/admin/auth/login",
  "/admin/auth/logout",
  "/admin/auth/session",
]);
function staffCookie(cookieHeader: string | null): string | null {
  const value = cookieHeader
    ?.split(";")
    .map((part) => part.trim())
    .find((part) => part.startsWith("x_fly_staff_session="));
  return value || null;
}

async function forwardAdminAuthRequest(request: Request, path: string): Promise<Response> {
  if (!allowedPaths.has(path)) {
    throw new Error("Unsupported admin authentication path");
  }

  const headers = new Headers();
  for (const name of ["content-type", "origin", "x-x-fly-csrf"] as const) {
    const value = request.headers.get(name);
    if (value) headers.set(name, value);
  }
  const cookie = staffCookie(request.headers.get("cookie"));
  if (cookie) headers.set("cookie", cookie);

  const response = await fetch(`${API_URL}${path}`, {
    method: request.method,
    headers,
    body: request.method === "GET" || request.method === "HEAD" ? undefined : await request.arrayBuffer(),
    cache: "no-store",
    redirect: "manual",
  });

  const responseHeaders = new Headers();
  const contentType = response.headers.get("content-type");
  const setCookie = response.headers.get("set-cookie");
  if (contentType) responseHeaders.set("content-type", contentType);
  if (setCookie) responseHeaders.set("set-cookie", setCookie);
  responseHeaders.set("cache-control", "no-store, private");

  return new Response(response.body, {
    status: response.status,
    headers: responseHeaders,
  });
}

async function fetchStaffPrincipal(cookieHeader: string): Promise<StaffPrincipal | null> {
  const cookie = staffCookie(cookieHeader);
  if (!cookie) return null;

  const response = await fetch(`${API_URL}/admin/auth/session`, {
    headers: { cookie },
    cache: "no-store",
  });
  if (response.status === 401) return null;
  if (!response.ok) throw new Error("Staff authentication service is unavailable");
  return parseStaffPrincipal(await response.json());
}

async function forwardDashboardRequest(request: Request): Promise<Response> {
  const headers = new Headers();
  const cookie = staffCookie(request.headers.get("cookie"));
  if (cookie) headers.set("cookie", cookie);
  try {
    const response = await fetch(`${API_URL}/admin/dashboard${new URL(request.url).search}`, {
      method: "GET", headers, cache: "no-store", redirect: "manual", signal: AbortSignal.timeout(15_000),
    });
    return new Response(response.body, {
      status: response.status,
      headers: { "content-type": "application/json", "cache-control": "no-store, private" },
    });
  } catch {
    return Response.json({ error: { code: "DASHBOARD_UNAVAILABLE" } }, {
      status: 503, headers: { "cache-control": "no-store, private" },
    });
  }
}

const flightPath = /^\/admin\/flights(?:\/reference-data|\/[0-9a-f-]+(?:\/cancel)?)?$/;
const bookingPath = /^\/admin\/bookings(?:\/search|\/XF[A-Z2-9]{8}(?:\/cancel)?)?$/;

async function forwardAdminFlightRequest(request: Request, path: string): Promise<Response> {
  if (!flightPath.test(path)) throw new Error("Unsupported flight management path");
  const headers = new Headers();
  const cookie = staffCookie(request.headers.get("cookie"));
  if (cookie) headers.set("cookie", cookie);
  for (const name of ["content-type", "origin", "x-x-fly-csrf"] as const) {
    const value = request.headers.get(name);
    if (value) headers.set(name, value);
  }
  try {
    const response = await fetch(`${API_URL}${path}${request.method === "GET" ? new URL(request.url).search : ""}`, {
      method: request.method, headers, cache: "no-store", redirect: "manual", signal: AbortSignal.timeout(15_000),
      body: request.method === "GET" || request.method === "HEAD" ? undefined : await request.arrayBuffer(),
    });
    return new Response(response.body, { status: response.status, headers: { "content-type": response.headers.get("content-type") ?? "application/json", "cache-control": "no-store, private" } });
  } catch {
    return Response.json({ error: { code: "FLIGHT_MANAGEMENT_UNAVAILABLE" } }, { status: 503, headers: { "cache-control": "no-store, private" } });
  }
}

async function forwardAdminBookingRequest(request: Request, path: string): Promise<Response> {
  if (!bookingPath.test(path)) throw new Error("Unsupported booking management path");
  const headers = new Headers();
  const cookie = staffCookie(request.headers.get("cookie"));
  if (cookie) headers.set("cookie", cookie);
  for (const name of ["content-type", "origin", "x-x-fly-csrf"] as const) {
    const value = request.headers.get(name);
    if (value) headers.set(name, value);
  }
  try {
    const response = await fetch(`${API_URL}${path}${request.method === "GET" ? new URL(request.url).search : ""}`, {
      method: request.method, headers, cache: "no-store", redirect: "manual", signal: AbortSignal.timeout(15_000),
      body: request.method === "GET" || request.method === "HEAD" ? undefined : await request.arrayBuffer(),
    });
    return new Response(response.body, { status: response.status, headers: { "content-type": response.headers.get("content-type") ?? "application/json", "cache-control": "no-store, private" } });
  } catch {
    return Response.json({ error: { code: "BOOKING_MANAGEMENT_UNAVAILABLE" } }, { status: 503, headers: { "cache-control": "no-store, private" } });
  }
}

export { fetchStaffPrincipal, forwardAdminAuthRequest, forwardAdminBookingRequest, forwardAdminFlightRequest, forwardDashboardRequest, parseStaffPrincipal };
