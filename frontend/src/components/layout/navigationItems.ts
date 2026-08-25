type NavigationItem = {
  href: string;
  id: string;
  label: string;
};

const navigationItems = [
  { href: "#global-reach", id: "explore", label: "Explore" },
  { href: "#global-reach", id: "destinations", label: "Destinations" },
  { href: "#cabins", id: "cabins", label: "Cabins" },
  { href: "#journey-experience", id: "experience", label: "Experience" },
] as const satisfies readonly NavigationItem[];

const bookingHref = "#journey";

export { bookingHref, navigationItems };
export type { NavigationItem };
