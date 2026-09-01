type NavigationItem = {
  href: string;
  id: string;
  labelKey:
    | "navigation.explore"
    | "navigation.offers"
    | "navigation.cabins"
    | "navigation.experience";
};

const navigationItems = [
  { href: "/#explore", id: "explore", labelKey: "navigation.explore" },
  { href: "/#offers", id: "offers", labelKey: "navigation.offers" },
  { href: "/#cabins", id: "cabins", labelKey: "navigation.cabins" },
  { href: "/#experience", id: "experience", labelKey: "navigation.experience" },
] as const satisfies readonly NavigationItem[];

const bookingHref = "/#flight-search";

export { bookingHref, navigationItems };
export type { NavigationItem };
