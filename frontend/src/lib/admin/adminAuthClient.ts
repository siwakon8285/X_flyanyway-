import { parseStaffPrincipal } from "@/lib/admin/adminPrincipal";
import type { StaffPrincipal } from "@/lib/admin/adminTypes";

type LoginInput = { email: string; password: string };

class StaffAuthClientError extends Error {
  constructor(
    readonly status: number,
    readonly code: string,
    message: string,
  ) {
    super(message);
    this.name = "StaffAuthClientError";
  }
}

async function errorFrom(response: Response): Promise<StaffAuthClientError> {
  let code = "STAFF_AUTH_FAILED";
  let message = "Unable to authenticate staff account.";
  try {
    const body = (await response.json()) as { error?: { code?: unknown; message?: unknown } };
    if (typeof body.error?.code === "string") code = body.error.code;
    if (typeof body.error?.message === "string") message = body.error.message;
  } catch {
    // Keep the stable fallback contract for non-JSON upstream failures.
  }
  return new StaffAuthClientError(response.status, code, message);
}

async function loginStaff(input: LoginInput): Promise<StaffPrincipal> {
  const response = await fetch("/admin/api/auth/login", {
    method: "POST",
    headers: { "Content-Type": "application/json", "X-X-Fly-CSRF": "1" },
    credentials: "include",
    cache: "no-store",
    body: JSON.stringify(input),
  });
  if (!response.ok) throw await errorFrom(response);
  return parseStaffPrincipal(await response.json());
}

async function logoutStaff(): Promise<void> {
  const response = await fetch("/admin/api/auth/logout", {
    method: "POST",
    headers: { "X-X-Fly-CSRF": "1" },
    credentials: "include",
    cache: "no-store",
  });
  if (!response.ok && response.status !== 401) throw await errorFrom(response);
}

export { loginStaff, logoutStaff, StaffAuthClientError };
