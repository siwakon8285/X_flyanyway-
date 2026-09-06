import {
  STAFF_PERMISSIONS,
  STAFF_ROLES,
  type StaffPermission,
  type StaffPrincipal,
  type StaffRole,
} from "@/lib/admin/adminTypes";

const staffRoleSet = new Set<string>(STAFF_ROLES);
const staffPermissionSet = new Set<string>(STAFF_PERMISSIONS);

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}

function parseStaffPrincipal(value: unknown): StaffPrincipal {
  if (typeof value !== "object" || value === null) {
    throw new Error("Invalid staff session response");
  }

  const candidate = value as Record<string, unknown>;
  if (
    typeof candidate.email !== "string" ||
    typeof candidate.expiresAt !== "string" ||
    Number.isNaN(Date.parse(candidate.expiresAt)) ||
    !isStringArray(candidate.roles) ||
    !candidate.roles.every((role) => staffRoleSet.has(role)) ||
    !isStringArray(candidate.permissions) ||
    !candidate.permissions.every((permission) => staffPermissionSet.has(permission))
  ) {
    throw new Error("Invalid staff session response");
  }

  return {
    email: candidate.email,
    expiresAt: candidate.expiresAt,
    roles: candidate.roles as StaffRole[],
    permissions: candidate.permissions as StaffPermission[],
  };
}

export { parseStaffPrincipal };
