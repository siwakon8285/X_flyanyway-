import {
  can,
  getVisibleAdminNavigation,
  hasRole,
} from "@/lib/admin/adminAuthorization";
import type { StaffPrincipal } from "@/lib/admin/adminTypes";

const principal: StaffPrincipal = {
  email: "system@x-fly.internal",
  expiresAt: "2026-09-06T12:00:00Z",
  permissions: ["staff:read", "staff:manage", "roles:read", "roles:manage"],
  roles: ["SYSTEM_ADMIN"],
};

describe("admin authorization presentation helpers", () => {
  it("uses effective permissions without treating System Admin as a superuser", () => {
    expect(hasRole(principal, "SYSTEM_ADMIN")).toBe(true);
    expect(can(principal, "staff:manage")).toBe(true);
    expect(can(principal, "flights:write")).toBe(false);
  });

  it("omits every unavailable future module instead of exposing dead links", () => {
    expect(getVisibleAdminNavigation(principal)).toEqual([
      { href: "/admin", id: "workspace", labelKey: "admin.navigation.workspace" },
    ]);
  });
});
