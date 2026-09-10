/** @jest-environment node */

import { forwardAdminApiClientRequest, forwardAdminAuthRequest, forwardAdminBookingRequest, forwardAdminFlightRequest, parseStaffPrincipal } from "@/lib/admin/adminBackend";

describe("admin backend boundary", () => {
  beforeEach(() => { global.fetch = jest.fn(); });

  it("forwards only the staff cookie and browser security headers", async () => {
    jest.mocked(fetch).mockResolvedValue(new Response(JSON.stringify({ ok: true }), {
      status: 200,
      headers: { "content-type": "application/json", "set-cookie": "x_fly_staff_session=opaque; Path=/admin; HttpOnly" },
    }));
    const request = new Request("http://localhost:3000/admin/api/auth/login", {
      method: "POST",
      headers: {
        cookie: "x_fly_staff_session=opaque; app-locale=en",
        origin: "http://localhost:3000",
        "x-x-fly-csrf": "1",
        "x-not-forwarded": "secret",
        "content-type": "application/json",
      },
      body: JSON.stringify({ email: "staff@x.test", password: "secret" }),
    });
    const response = await forwardAdminAuthRequest(request, "/admin/auth/login");
    const [, init] = jest.mocked(fetch).mock.calls[0];
    const headers = new Headers(init?.headers);
    expect(headers.get("origin")).toBe("http://localhost:3000");
    expect(headers.get("x-x-fly-csrf")).toBe("1");
    expect(headers.get("cookie")).toBe("x_fly_staff_session=opaque");
    expect(headers.get("x-not-forwarded")).toBeNull();
    expect(response.headers.get("set-cookie")).toContain("Path=/admin");
    expect(response.headers.get("cache-control")).toBe("no-store, private");
  });

  it("rejects malformed or unknown authorization data from the backend", () => {
    expect(() => parseStaffPrincipal({ email: "staff@x.test", roles: ["UNIVERSAL_ADMIN"], permissions: [] })).toThrow();
    expect(() => parseStaffPrincipal({ email: "staff@x.test", roles: ["SYSTEM_ADMIN"], permissions: ["admin:all"] })).toThrow();
  });

  it("forwards flight mutations with only the staff cookie and browser security boundary", async () => {
    jest.mocked(fetch).mockResolvedValue(new Response(JSON.stringify({ id: "flight" }), { status: 200, headers: { "content-type": "application/json" } }));
    const request = new Request("http://localhost:3000/admin/api/flights/11111111-1111-4111-8111-111111111111/cancel", { method: "POST", headers: {
      cookie: "x_fly_staff_session=opaque; customer_hold=private", origin: "http://localhost:3000", "x-x-fly-csrf": "1", "content-type": "application/json", "x-leak": "no",
    }, body: JSON.stringify({ version: 1 }) });
    await forwardAdminFlightRequest(request, "/admin/flights/11111111-1111-4111-8111-111111111111/cancel");
    const [, init] = jest.mocked(fetch).mock.calls[0];
    const headers = new Headers(init?.headers);
    expect(headers.get("cookie")).toBe("x_fly_staff_session=opaque");
    expect(headers.get("origin")).toBe("http://localhost:3000");
    expect(headers.get("x-x-fly-csrf")).toBe("1");
    expect(headers.get("x-leak")).toBeNull();
  });

  it("forwards passenger search in a protected POST body without customer credentials", async () => {
    jest.mocked(fetch).mockResolvedValue(new Response(JSON.stringify({ items: [], nextOffset: null }), { status: 200, headers: { "content-type": "application/json" } }));
    const request = new Request("http://localhost:3000/admin/api/bookings/search", { method: "POST", headers: {
      cookie: "x_fly_staff_session=opaque; x_fly_manage_booking=customer-secret", origin: "http://localhost:3000", "x-x-fly-csrf": "1", "content-type": "application/json",
    }, body: JSON.stringify({ passengerName: "Synthetic Passenger", limit: 50, offset: 0 }) });
    await forwardAdminBookingRequest(request, "/admin/bookings/search");
    const [url, init] = jest.mocked(fetch).mock.calls[0];
    const headers = new Headers(init?.headers);
    expect(url).toBe("http://localhost:8080/api/v1/admin/bookings/search");
    expect(headers.get("cookie")).toBe("x_fly_staff_session=opaque");
    expect(String(url)).not.toContain("Synthetic");
    expect(new TextDecoder().decode(init?.body as ArrayBuffer)).toContain("Synthetic Passenger");
  });

  it("forwards only canonical public API-client paths and approved staff headers", async () => {
    jest.mocked(fetch).mockResolvedValue(new Response(JSON.stringify({ clientId:"XFCABCDEFGHJKLMNPQR" }), { status:200, headers:{ "content-type":"application/json" } }));
    const request = new Request("http://localhost:3000/admin/api/api-clients/XFCABCDEFGHJKLMNPQR/revoke", {
      method:"POST",
      headers:{ cookie:"x_fly_staff_session=opaque; customer=private", origin:"http://localhost:3000", "x-x-fly-csrf":"1", "content-type":"application/json", "x-private":"never" },
      body:JSON.stringify({ version:4 }),
    });
    await forwardAdminApiClientRequest(request, "/admin/api-clients/XFCABCDEFGHJKLMNPQR/revoke");
    const [url, init] = jest.mocked(fetch).mock.calls[0];
    const headers = new Headers(init?.headers);
    expect(url).toBe("http://localhost:8080/api/v1/admin/api-clients/XFCABCDEFGHJKLMNPQR/revoke");
    expect(headers.get("cookie")).toBe("x_fly_staff_session=opaque");
    expect(headers.get("x-private")).toBeNull();
    await expect(forwardAdminApiClientRequest(request, "/admin/api-clients/not-a-client/revoke")).rejects.toThrow("Unsupported API client management path");
  });

  it("forwards literal API-client scope codes without transforming their identity or order", async () => {
    jest.mocked(fetch).mockResolvedValue(new Response("{}", { status:201, headers:{ "content-type":"application/json" } }));
    const payload = JSON.stringify({
      name:"Scope fidelity",
      description:null,
      status:"ACTIVE",
      allowedScopes:["flights:read", "analytics:read"],
    });
    const request = new Request("http://localhost:3000/admin/api/api-clients", {
      method:"POST",
      headers:{ "content-type":"application/json" },
      body:payload,
    });
    await forwardAdminApiClientRequest(request, "/admin/api-clients");
    const [, init] = jest.mocked(fetch).mock.calls[0];
    const forwarded = new TextDecoder().decode(init?.body as ArrayBuffer);
    expect(forwarded).toBe(payload);
    expect(JSON.parse(forwarded).allowedScopes).toEqual(["flights:read", "analytics:read"]);
  });
});
