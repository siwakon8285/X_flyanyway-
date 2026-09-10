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
  it("offers the executive dashboard only when every analytics permission is present", () => {
    const executive: StaffPrincipal = { ...principal, roles: ["EXECUTIVE"], permissions: ["dashboard:read", "analytics:read", "reports:read"] };
    expect(getVisibleAdminNavigation(executive)).toEqual([{ href: "/admin/dashboard", id: "dashboard", labelKey: "admin.navigation.overview" }]);
    for (const missing of executive.permissions) {
      expect(getVisibleAdminNavigation({ ...executive, permissions: executive.permissions.filter((p) => p !== missing) }).some((item) => item.id === "dashboard")).toBe(false);
    }
  });
  it("uses effective permissions without treating System Admin as a superuser", () => {
    expect(hasRole(principal, "SYSTEM_ADMIN")).toBe(true);
    expect(can(principal, "staff:manage")).toBe(true);
    expect(can(principal, "flights:write")).toBe(false);
  });

  it("shows flight management only from the effective read permission", () => {
    expect(getVisibleAdminNavigation({ ...principal, roles: ["BAGGAGE_STAFF"], permissions: ["flights:read"] }))
      .toEqual([{ href: "/admin", id: "workspace", labelKey: "admin.navigation.workspace" }, { href: "/admin/flights", id: "flights", labelKey: "admin.navigation.flights" }]);
    expect(getVisibleAdminNavigation({ ...principal, roles: ["FLIGHT_MANAGER"], permissions: ["flights:read", "flights:write"] }))
      .toEqual([{ href: "/admin", id: "workspace", labelKey: "admin.navigation.workspace" }, { href: "/admin/flights", id: "flights", labelKey: "admin.navigation.flights" }]);
  });

  it("shows Booking Operations only from the effective booking read permission", () => {
    const bookingOperations: StaffPrincipal = { ...principal, roles: ["BOOKING_OPERATIONS"], permissions: ["bookings:read", "bookings:manage"] };
    expect(getVisibleAdminNavigation(bookingOperations)).toEqual([
      { href: "/admin", id: "workspace", labelKey: "admin.navigation.workspace" },
      { href: "/admin/bookings", id: "bookings", labelKey: "admin.navigation.bookings" },
    ]);
    expect(getVisibleAdminNavigation(principal).some((item) => item.id === "bookings")).toBe(false);
  });

  it("shows Ticket Operations only when both ticket and passenger reads are effective", () => {
    const operator: StaffPrincipal = { ...principal, roles:["TICKET_PASSENGER_OPERATIONS"], permissions:["tickets:read","tickets:print","passengers:read"] };
    expect(getVisibleAdminNavigation(operator).some((item) => item.id === "tickets")).toBe(true);
    expect(getVisibleAdminNavigation({ ...operator, permissions:["tickets:read","tickets:print"] }).some((item) => item.id === "tickets")).toBe(false);
    expect(getVisibleAdminNavigation({ ...operator, permissions:["passengers:read"] }).some((item) => item.id === "tickets")).toBe(false);
  });

  it("shows API Client Management only from the effective read grant", () => {
    const apiAdmin: StaffPrincipal = { ...principal, roles:["API_ADMIN"], permissions:["api_clients:read","api_clients:manage"] };
    expect(getVisibleAdminNavigation(apiAdmin)).toEqual([
      { href:"/admin", id:"workspace", labelKey:"admin.navigation.workspace" },
      { href:"/admin/api-clients", id:"api-clients", labelKey:"admin.navigation.apiClients" },
    ]);
    expect(getVisibleAdminNavigation(principal).some((item) => item.id === "api-clients")).toBe(false);
  });

  it("omits every unavailable future module instead of exposing dead links", () => {
    expect(getVisibleAdminNavigation(principal)).toEqual([
      { href: "/admin", id: "workspace", labelKey: "admin.navigation.workspace" },
    ]);
  });
});
