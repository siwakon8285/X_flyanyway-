/** @jest-environment node */
import { GET } from "@/app/admin/api/dashboard/route";

describe("dashboard proxy", () => {
  beforeEach(() => { global.fetch = jest.fn(); });
  it("forwards filters and only the staff cookie, preserving authorization status without caching", async () => {
    jest.mocked(fetch).mockResolvedValue(new Response('{"error":{"code":"STAFF_PERMISSION_DENIED"}}', { status: 403, headers: { "content-type": "application/json" } }));
    const response = await GET(new Request("http://localhost/admin/api/dashboard?from=2026-09-01&to=2026-09-06&route=BKK-NRT&cabin=business&provider=STRIPE", { headers: { cookie: "x_fly_staff_session=opaque; customer_token=private", authorization: "secret" } }));
    const [url, init] = jest.mocked(fetch).mock.calls[0];
    expect(String(url)).toContain("/admin/dashboard?from=2026-09-01&to=2026-09-06&route=BKK-NRT&cabin=business&provider=STRIPE");
    expect(new Headers(init?.headers).get("cookie")).toBe("x_fly_staff_session=opaque");
    expect(new Headers(init?.headers).has("authorization")).toBe(false);
    expect(init?.cache).toBe("no-store");
    expect(response.status).toBe(403);
    expect(response.headers.get("cache-control")).toBe("no-store, private");
  });
  it("returns a sanitized unavailable response on backend failure", async () => {
    jest.mocked(fetch).mockRejectedValue(new Error("private database address"));
    const response = await GET(new Request("http://localhost/admin/api/dashboard"));
    expect(response.status).toBe(503);
    expect(await response.text()).not.toContain("private database");
  });
});
