type NavigationItem = {
  href: string;
  id: string;
  labelKey:
    | "navigation.explore"
    | "navigation.offers"
    | "navigation.cabins"
    | "navigation.experience"
    | "footer.manageBooking";
};

const navigationItems = [
  { href: "/#explore", id: "explore", labelKey: "navigation.explore" },
  { href: "/#offers", id: "offers", labelKey: "navigation.offers" },
  { href: "/#cabins", id: "cabins", labelKey: "navigation.cabins" },
  { href: "/#experience", id: "experience", labelKey: "navigation.experience" },
  { href: "/manage-booking", id: "manage-booking", labelKey: "footer.manageBooking" },
] as const satisfies readonly NavigationItem[];

const bookingHref = "/#flight-search";
const manageBookingHref = "/manage-booking";

export { bookingHref, manageBookingHref, navigationItems };
export type { NavigationItem };
