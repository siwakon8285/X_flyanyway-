type NavigationItem = {
  href: string;
  id: string;
  labelKey:
    | "navigation.explore"
    | "navigation.destinations"
    | "navigation.cabins"
    | "navigation.experience";
};

const navigationItems = [
  { href: "#global-reach", id: "explore", labelKey: "navigation.explore" },
  { href: "#global-reach", id: "destinations", labelKey: "navigation.destinations" },
  { href: "#cabins", id: "cabins", labelKey: "navigation.cabins" },
  { href: "#journey-experience", id: "experience", labelKey: "navigation.experience" },
] as const satisfies readonly NavigationItem[];

const bookingHref = "/#flight-search";

export { bookingHref, navigationItems };
export type { NavigationItem };
