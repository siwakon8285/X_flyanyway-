import { screen } from "@testing-library/react";
import { redirect } from "next/navigation";

import BookingsPage from "@/app/admin/(protected)/bookings/page";
import BookingDetailPage from "@/app/admin/(protected)/bookings/[bookingReference]/page";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";
import type { StaffPrincipal } from "@/lib/admin/adminTypes";
import { render } from "@/tests/renderWithLanguage";

jest.mock("next/headers", () => ({ cookies: async () => ({ toString: (): string => "x_fly_staff_session=test" }) }));
jest.mock("next/navigation", () => ({ redirect: jest.fn((path) => { throw new Error(`redirect:${path}`); }) }));
jest.mock("@/lib/admin/adminBackend", () => ({ fetchStaffPrincipal: jest.fn() }));

const operator: StaffPrincipal = { email: "operator@x.test", roles: ["BOOKING_OPERATIONS"], permissions: ["bookings:read", "bookings:manage"], expiresAt: "2026-12-01T00:00:00Z" };

describe("Booking Management pages", () => {
  beforeEach(() => { jest.clearAllMocks(); global.fetch = jest.fn().mockReturnValue(new Promise(() => {})); });
  it("renders list and exact detail routes for effective read permission", async () => {
    jest.mocked(fetchStaffPrincipal).mockResolvedValue(operator);
    render(await BookingsPage());
    expect(screen.getByRole("heading", { name: "Booking Operations" })).toBeInTheDocument();
    render(await BookingDetailPage({ params: Promise.resolve({ bookingReference: "xf22aaa222" }) }));
    expect(fetch).toHaveBeenCalledWith("/admin/api/bookings/XF22AAA222", expect.anything());
  });
  it("redirects unauthenticated and unrelated roles", async () => {
    jest.mocked(fetchStaffPrincipal).mockResolvedValue(null);
    await expect(BookingsPage()).rejects.toThrow("redirect:/admin/login");
    jest.mocked(fetchStaffPrincipal).mockResolvedValue({ ...operator, roles: ["EXECUTIVE"], permissions: ["dashboard:read", "analytics:read", "reports:read"] });
    await expect(BookingsPage()).rejects.toThrow("redirect:/admin");
    expect(redirect).toHaveBeenCalled();
  });
});
