import { BrandWordmark } from "@/components/brand/BrandWordmark";
import { Container } from "@/components/layout/Container";
import { bookingHref } from "@/components/layout/navigationItems";

const footerGroups = [
  {
    id: "explore",
    label: "Explore",
    links: [
      { href: "#global-reach", id: "explore", label: "Explore" },
      { href: "#global-reach", id: "destinations", label: "Destinations" },
      { href: "#cabins", id: "cabins", label: "Cabins" },
    ],
  },
  {
    id: "travel",
    label: "Travel",
    links: [
      { href: bookingHref, id: "book-flight", label: "Book a Flight" },
      { href: "#top", id: "manage-booking", label: "Manage Booking" },
    ],
  },
  {
    id: "company",
    label: "Company",
    links: [
      { href: "#journey-experience", id: "about", label: "About X-Fly" },
    ],
  },
  {
    id: "support",
    label: "Support",
    links: [{ href: "#top", id: "help", label: "Help Centre" }],
  },
  {
    id: "legal",
    label: "Legal",
    links: [
      { href: "#top", id: "privacy", label: "Privacy" },
      { href: "#top", id: "terms", label: "Terms" },
    ],
  },
] as const;

const Footer = () => (
  <footer className="border-t border-border/80 bg-surface/25 py-section-sm">
    <Container>
      <div className="grid gap-12 border-b border-border/80 pb-12 lg:grid-cols-[minmax(0,1.5fr)_minmax(0,3fr)]">
        <div className="max-w-sm">
          <BrandWordmark className="text-sm" />
          <p className="mt-6 text-body-sm text-muted-foreground">
            A premium global aviation experience designed for what comes next.
          </p>
        </div>
        <div className="grid grid-cols-2 gap-x-6 gap-y-10 sm:grid-cols-3 lg:grid-cols-5">
          {footerGroups.map((group) => (
            <section key={group.id}>
              <h2 className="text-label text-foreground">{group.label}</h2>
              <ul className="mt-4 space-y-3">
                {group.links.map((link) => (
                  <li key={link.id}>
                    <a
                      className="rounded-sm text-body-sm text-muted-foreground outline-none transition-colors hover:text-brand focus-visible:ring-2 focus-visible:ring-focus"
                      href={link.href}
                    >
                      {link.label}
                    </a>
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      </div>
      <div className="flex flex-col gap-3 pt-6 text-caption text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
        <p>© 2026 X-Fly Anyway.</p>
        <p>Designed to go anywhere.</p>
      </div>
    </Container>
  </footer>
);

export { Footer };
