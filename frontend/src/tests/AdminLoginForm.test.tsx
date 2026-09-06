import { fireEvent, screen, waitFor } from "@testing-library/react";

import { AdminLoginForm } from "@/components/admin/auth/AdminLoginForm";
import { AdminLoginPanel } from "@/components/admin/auth/AdminLoginPanel";
import { render } from "@/tests/renderWithLanguage";

describe("AdminLoginForm", () => {
  beforeEach(() => {
    global.fetch = jest.fn();
  });

  it("submits credentials through the staff auth boundary", async () => {
    jest.mocked(fetch).mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({
      email: "flight@x-fly.internal",
      expiresAt: "2026-09-06T12:00:00Z",
      permissions: ["flights:read", "flights:write"],
      roles: ["FLIGHT_MANAGER"],
      }),
    } as Response);
    render(<AdminLoginForm />);

    fireEvent.change(screen.getByLabelText("Staff email"), { target: { value: "flight@x-fly.internal" } });
    fireEvent.change(screen.getByLabelText("Password"), { target: { value: "A secure staff passphrase" } });
    fireEvent.submit(screen.getByRole("button", { name: "Sign in" }).closest("form")!);

    await waitFor(() => expect(fetch).toHaveBeenCalledTimes(1));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("uses the same generic error for rejected credentials", async () => {
    jest.mocked(fetch).mockResolvedValue({ ok: false, status: 401, json: async () => ({ error: { code: "STAFF_LOGIN_FAILED", message: "Email or password is incorrect." } }) } as Response);
    render(<AdminLoginForm />);
    fireEvent.change(screen.getByLabelText("Staff email"), { target: { value: "unknown@x.test" } });
    fireEvent.change(screen.getByLabelText("Password"), { target: { value: "not the password" } });
    fireEvent.submit(screen.getByRole("button", { name: "Sign in" }).closest("form")!);
    expect(await screen.findByRole("alert")).toHaveTextContent("Email or password is incorrect.");
  });

  it("distinguishes safe throttle guidance from credential failure", async () => {
    jest.mocked(fetch).mockResolvedValue({ ok: false, status: 429, json: async () => ({ error: { code: "STAFF_LOGIN_THROTTLED", message: "Unable to sign in. Try again later." } }) } as Response);
    render(<AdminLoginForm />);
    fireEvent.change(screen.getByLabelText("Staff email"), { target: { value: "staff@x.test" } });
    fireEvent.change(screen.getByLabelText("Password"), { target: { value: "not the password" } });
    fireEvent.submit(screen.getByRole("button", { name: "Sign in" }).closest("form")!);
    expect(await screen.findByRole("alert")).toHaveTextContent("Unable to sign in. Try again later.");
  });

  it("renders deterministic Thai labels from the shared language policy", () => {
    render(<AdminLoginPanel />, { locale: "th" });
    expect(screen.getByRole("heading", { name: "เข้าสู่ระบบสำหรับพนักงาน" })).toBeInTheDocument();
    expect(screen.getByLabelText("อีเมลพนักงาน")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "เข้าสู่ระบบ" })).toBeInTheDocument();
  });
});
