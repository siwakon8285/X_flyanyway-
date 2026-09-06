const STAFF_ROLES = [
  "EXECUTIVE",
  "FLIGHT_MANAGER",
  "BOOKING_OPERATIONS",
  "TICKET_PASSENGER_OPERATIONS",
  "BAGGAGE_STAFF",
  "API_ADMIN",
  "SYSTEM_ADMIN",
] as const;

type StaffRole = (typeof STAFF_ROLES)[number];

const STAFF_PERMISSIONS = [
  "dashboard:read", "analytics:read", "reports:read", "flights:read",
  "flights:write", "bookings:read", "bookings:manage", "tickets:read",
  "tickets:print", "passengers:read", "bookings:read_limited",
  "passengers:read_limited", "baggage_context:read", "api_clients:read",
  "api_clients:manage", "staff:read", "staff:manage", "roles:read",
  "roles:manage",
] as const;

type StaffPermission = (typeof STAFF_PERMISSIONS)[number];

type StaffPrincipal = {
  email: string;
  expiresAt: string;
  permissions: StaffPermission[];
  roles: StaffRole[];
};

export { STAFF_PERMISSIONS, STAFF_ROLES };
export type { StaffPermission, StaffPrincipal, StaffRole };
