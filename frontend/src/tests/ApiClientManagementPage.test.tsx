import { screen } from "@testing-library/react";

import ApiClientsPage from "@/app/admin/(protected)/api-clients/page";
import NewApiClientPage from "@/app/admin/(protected)/api-clients/new/page";
import ApiClientDetailPage from "@/app/admin/(protected)/api-clients/[clientId]/page";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";
import type { StaffPrincipal } from "@/lib/admin/adminTypes";
import { render } from "@/tests/renderWithLanguage";

jest.mock("next/headers", () => ({ cookies: async () => ({ toString: (): string => "x_fly_staff_session=test" }) }));
jest.mock("next/navigation", () => ({
  redirect: jest.fn((path) => { throw new Error(`redirect:${path}`); }),
  useRouter: () => ({ replace: jest.fn() }),
}));
jest.mock("@/lib/admin/adminBackend", () => ({ fetchStaffPrincipal: jest.fn() }));

const admin: StaffPrincipal = { email:"api@x.test", roles:["API_ADMIN"], permissions:["api_clients:read","api_clients:manage"], expiresAt:"2026-12-01T00:00:00Z" };

describe("API Client Management pages", () => {
  beforeEach(() => { jest.clearAllMocks(); global.fetch = jest.fn().mockReturnValue(new Promise(() => {})); });

  it("renders list, create, and public Client ID detail routes from effective permissions", async () => {
    jest.mocked(fetchStaffPrincipal).mockResolvedValue(admin);
    jest.mocked(global.fetch).mockImplementation((input) => {
      const url = String(input);
      if (url.endsWith("/scopes")) return Promise.resolve({ ok:true, json:async () => [] } as Response);
      if (url.includes("XFC")) return Promise.resolve({ ok:true, json:async () => ({ clientId:"XFCABCDEFGHJKLMNPQR", name:"Detail", description:null, status:"SUSPENDED", allowedScopes:[], version:1, createdAt:"2026-09-09T00:00:00Z", updatedAt:"2026-09-09T00:00:00Z", createdBy:"api@x.test", updatedBy:"api@x.test", audit:[] }) } as Response);
      return Promise.resolve({ ok:true, json:async () => ({ items:[], nextOffset:null }) } as Response);
    });
    render(await ApiClientsPage());
    expect(screen.getByRole("heading", { name:"API Client Management" })).toBeInTheDocument();
    await screen.findByText("No API clients found");
    render(await NewApiClientPage());
    expect(await screen.findByRole("heading", { name:"Register API client" })).toBeInTheDocument();
    render(await ApiClientDetailPage({ params:Promise.resolve({ clientId:"XFCABCDEFGHJKLMNPQR" }) }));
    await screen.findByRole("heading", { name:"Detail" });
    expect(fetch).toHaveBeenCalledWith("/admin/api/api-clients/XFCABCDEFGHJKLMNPQR", expect.anything());
  });

  it("redirects unauthenticated and unrelated staff without role-name bypasses", async () => {
    jest.mocked(fetchStaffPrincipal).mockResolvedValue(null);
    await expect(ApiClientsPage()).rejects.toThrow("redirect:/admin/login");
    jest.mocked(fetchStaffPrincipal).mockResolvedValue({ ...admin, roles:["SYSTEM_ADMIN"], permissions:["staff:read","staff:manage","roles:read","roles:manage"] });
    await expect(ApiClientsPage()).rejects.toThrow("redirect:/admin");
    await expect(NewApiClientPage()).rejects.toThrow("redirect:/admin");
  });
});
