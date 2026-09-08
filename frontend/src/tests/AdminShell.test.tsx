import { fireEvent, screen } from "@testing-library/react";

import { AdminShell } from "@/components/admin/shell/AdminShell";
import type { StaffPrincipal } from "@/lib/admin/adminTypes";
import { render } from "@/tests/renderWithLanguage";

const principal: StaffPrincipal = {
  email: "system@x-fly.internal",
  expiresAt: "2026-09-06T12:00:00Z",
  permissions: ["staff:read", "staff:manage", "roles:read", "roles:manage"],
  roles: ["SYSTEM_ADMIN"],
};

describe("AdminShell", () => {
  it("renders the staff identity, effective role, and only available navigation", () => {
    render(<AdminShell principal={principal}><p>Protected workspace</p></AdminShell>);

    expect(screen.getByText("Protected workspace")).toBeInTheDocument();
    expect(screen.getByText("system@x-fly.internal")).toBeInTheDocument();
    expect(screen.getByText("System Admin")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Sign out" })).not.toHaveLength(0);
    expect(screen.getByRole("link", { name: "Workspace" })).toHaveAttribute("href", "/admin");
    expect(screen.queryByText("Flights")).not.toBeInTheDocument();
    expect(screen.queryByText("Executive dashboard")).not.toBeInTheDocument();
  });

  it("opens an accessible mobile navigation and restores it with Escape", () => {
    render(<AdminShell principal={principal}><p>Protected workspace</p></AdminShell>);
    const trigger = screen.getByRole("button", { name: "Open staff navigation" });
    fireEvent.click(trigger);
    expect(screen.getByRole("dialog", { name: "Staff navigation" })).toBeInTheDocument();
    fireEvent.keyDown(document, { key: "Escape" });
    expect(screen.queryByRole("dialog", { name: "Staff navigation" })).not.toBeInTheDocument();
  });

  it("renders bilingual shell copy", () => {
    render(<AdminShell principal={principal}><p>พื้นที่ทำงาน</p></AdminShell>, { locale: "th" });
    expect(screen.getByRole("link", { name: "พื้นที่ทำงาน" })).toBeInTheDocument();
    expect(screen.getByText("ผู้ดูแลระบบ")).toBeInTheDocument();
  });

  it("shows Booking Operations navigation only when its effective read grant exists", () => {
    render(<AdminShell principal={{ ...principal, roles: ["BOOKING_OPERATIONS"], permissions: ["bookings:read", "bookings:manage"] }}><p>Bookings</p></AdminShell>);
    expect(screen.getByRole("link", { name: "Bookings" })).toHaveAttribute("href", "/admin/bookings");
    expect(screen.queryByRole("link", { name: "Flights" })).not.toBeInTheDocument();
  });

  it("keeps Flight Manager navigation limited to flights", () => {
    render(<AdminShell principal={{ ...principal, roles: ["FLIGHT_MANAGER"], permissions: ["flights:read", "flights:write"] }}><p>Flights</p></AdminShell>);
    expect(screen.getByRole("link", { name: "Flights" })).toHaveAttribute("href", "/admin/flights");
    expect(screen.queryByRole("link", { name: "Bookings" })).not.toBeInTheDocument();
  });
});
