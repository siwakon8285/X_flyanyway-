type NavigationItem = {
  href: string;
  id: string;
  label: string;
};

const navigationItems = [
  { href: "#journey", id: "explore", label: "Explore" },
  { href: "#journey", id: "destinations", label: "Destinations" },
  { href: "#journey", id: "cabins", label: "Cabins" },
  { href: "#experience", id: "experience", label: "Experience" },
] as const satisfies readonly NavigationItem[];

const bookingHref = "#journey";

export { bookingHref, navigationItems };
export type { NavigationItem };
