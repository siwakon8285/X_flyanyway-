/** @jest-environment node */

import { loginStaff, logoutStaff, StaffAuthClientError } from "@/lib/admin/adminAuthClient";

const principal = {
  email: "system@x-fly.internal",
  expiresAt: "2026-09-06T12:00:00Z",
  permissions: ["staff:read", "staff:manage", "roles:read", "roles:manage"],
  roles: ["SYSTEM_ADMIN"],
};

describe("staff auth client", () => {
  beforeEach(() => { global.fetch = jest.fn(); });

  it("logs in through the same-origin admin boundary with CSRF protection", async () => {
    jest.mocked(fetch).mockResolvedValue(new Response(JSON.stringify(principal), {
      status: 200, headers: { "content-type": "application/json" },
    }));
    await expect(loginStaff({ email: principal.email, password: "secret" })).resolves.toEqual(principal);
    expect(fetch).toHaveBeenCalledWith("/admin/api/auth/login", expect.objectContaining({
      cache: "no-store", credentials: "include", method: "POST",
      headers: expect.objectContaining({ "X-X-Fly-CSRF": "1" }),
    }));
  });

  it("surfaces only the stable generic login failure contract", async () => {
    jest.mocked(fetch).mockResolvedValue(new Response(JSON.stringify({
      error: { code: "STAFF_LOGIN_FAILED", message: "Email or password is incorrect." },
    }), { status: 401, headers: { "content-type": "application/json" } }));
    await expect(loginStaff({ email: "unknown@x.test", password: "wrong" })).rejects.toEqual(
      expect.objectContaining<Partial<StaffAuthClientError>>({ code: "STAFF_LOGIN_FAILED", status: 401 }),
    );
  });

  it("logs out idempotently with the same CSRF boundary", async () => {
    jest.mocked(fetch).mockResolvedValue(new Response(null, { status: 204 }));
    await expect(logoutStaff()).resolves.toBeUndefined();
    expect(fetch).toHaveBeenCalledWith("/admin/api/auth/logout", expect.objectContaining({
      credentials: "include", method: "POST", headers: { "X-X-Fly-CSRF": "1" },
    }));
  });
});
