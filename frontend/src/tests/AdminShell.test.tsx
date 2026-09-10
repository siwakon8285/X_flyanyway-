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
    const lockup = screen.getByRole("img", { name: "X-Fly Anyway" });
    expect(lockup).toHaveTextContent("-FLY ANYWAY");
    expect(lockup.querySelector("img")).toHaveAttribute("width", "56");
    expect(lockup.querySelector("img")).toHaveAttribute("height", "56");
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
    expect(screen.getByRole("button", { name:"Close dialog" })).toBeInTheDocument();
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

  it("shows Ticket Operations navigation from effective permissions", () => {
    render(<AdminShell principal={{ ...principal, roles:["TICKET_PASSENGER_OPERATIONS"], permissions:["tickets:read","tickets:print","passengers:read"] }}><p>Tickets</p></AdminShell>);
    expect(screen.getByRole("link", { name:"Tickets / Passengers" })).toHaveAttribute("href", "/admin/tickets");
    expect(screen.queryByRole("link", { name:"Bookings" })).not.toBeInTheDocument();
  });

  it("shows API client navigation from effective permissions without System Admin bypass", () => {
    render(<AdminShell principal={{ ...principal, roles:["API_ADMIN"], permissions:["api_clients:read","api_clients:manage"] }}><p>API clients</p></AdminShell>);
    expect(screen.getByRole("link", { name:"API Clients" })).toHaveAttribute("href", "/admin/api-clients");
    expect(screen.queryByRole("link", { name:"Staff / Access" })).not.toBeInTheDocument();
  });
});
