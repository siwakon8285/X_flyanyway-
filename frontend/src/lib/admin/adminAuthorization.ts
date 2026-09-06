import type { TranslationKey } from "@/i18n/types";
import type { StaffPermission, StaffPrincipal, StaffRole } from "@/lib/admin/adminTypes";

type AdminNavigationItem = {
  available: boolean;
  href: string;
  id: string;
  labelKey: TranslationKey;
  permission?: StaffPermission;
};

const navigationCatalog: readonly AdminNavigationItem[] = [
  { available: true, href: "/admin", id: "workspace", labelKey: "admin.navigation.workspace" },
  { available: false, href: "/admin/overview", id: "overview", labelKey: "admin.navigation.overview", permission: "dashboard:read" },
  { available: false, href: "/admin/flights", id: "flights", labelKey: "admin.navigation.flights", permission: "flights:read" },
  { available: false, href: "/admin/bookings", id: "bookings", labelKey: "admin.navigation.bookings", permission: "bookings:read" },
  { available: false, href: "/admin/tickets", id: "tickets", labelKey: "admin.navigation.tickets", permission: "tickets:read" },
  { available: false, href: "/admin/reports", id: "reports", labelKey: "admin.navigation.reports", permission: "reports:read" },
  { available: false, href: "/admin/api-clients", id: "api-clients", labelKey: "admin.navigation.apiClients", permission: "api_clients:read" },
  { available: false, href: "/admin/staff", id: "staff", labelKey: "admin.navigation.staff", permission: "staff:read" },
];

const can = (principal: StaffPrincipal, permission: StaffPermission) =>
  principal.permissions.includes(permission);

const hasRole = (principal: StaffPrincipal, role: StaffRole) =>
  principal.roles.includes(role);

const getVisibleAdminNavigation = (principal: StaffPrincipal) =>
  navigationCatalog
    .filter((item) => item.available && (!item.permission || can(principal, item.permission)))
    .map(({ href, id, labelKey }) => ({ href, id, labelKey }));

export { can, getVisibleAdminNavigation, hasRole, navigationCatalog };
export type { AdminNavigationItem };
