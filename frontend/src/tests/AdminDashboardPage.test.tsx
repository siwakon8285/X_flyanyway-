import { screen } from "@testing-library/react";
import AdminPage from "@/app/admin/(protected)/page";
import DashboardPage from "@/app/admin/(protected)/dashboard/page";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";
import { redirect } from "next/navigation";
import { render } from "@/tests/renderWithLanguage";
import type { StaffPrincipal } from "@/lib/admin/adminTypes";

jest.mock("next/headers", () => ({ cookies: async () => ({ toString: (): string => "x_fly_staff_session=test" }) }));
jest.mock("next/navigation", () => ({ redirect: jest.fn((path) => { throw new Error(`redirect:${path}`); }) }));
jest.mock("@/lib/admin/adminBackend", () => ({ fetchStaffPrincipal: jest.fn() }));
const principal: StaffPrincipal = { email: "owner@x.test", roles: ["EXECUTIVE"], permissions: ["dashboard:read", "analytics:read", "reports:read"], expiresAt: "2026-09-07T00:00:00Z" };

describe("admin executive entry points", () => {
  beforeEach(() => { jest.clearAllMocks(); global.fetch = jest.fn().mockReturnValue(new Promise(() => {})); });
  it("routes authorized executives from /admin to dashboard", async () => {
    jest.mocked(fetchStaffPrincipal).mockResolvedValue(principal);
    await expect(AdminPage()).rejects.toThrow("redirect:/admin/dashboard");
    expect(redirect).toHaveBeenCalledWith("/admin/dashboard");
  });
  it("renders the executive dashboard for an authorized principal", async () => {
    jest.mocked(fetchStaffPrincipal).mockResolvedValue(principal);
    render(await DashboardPage());
    expect(screen.getByRole("heading", { name: "Commercial performance" })).toBeInTheDocument();
  });
  it.each(principal.permissions)("keeps staff lacking %s on the neutral workspace", async (missing) => {
    jest.mocked(fetchStaffPrincipal).mockResolvedValue({ ...principal, permissions: principal.permissions.filter((p) => p !== missing) });
    render(await AdminPage());
    expect(screen.queryByRole("heading", { name: "Commercial performance" })).not.toBeInTheDocument();
    expect(fetch).not.toHaveBeenCalled();
    await expect(DashboardPage()).rejects.toThrow("redirect:/admin");
  });
  it("redirects unauthenticated requests to login", async () => {
    jest.mocked(fetchStaffPrincipal).mockResolvedValue(null);
    await expect(DashboardPage()).rejects.toThrow("redirect:/admin/login");
  });
});
