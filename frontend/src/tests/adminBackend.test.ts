/** @jest-environment node */

import { forwardAdminAuthRequest, forwardAdminFlightRequest, parseStaffPrincipal } from "@/lib/admin/adminBackend";

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
});
