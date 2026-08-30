"use client";

import { BrandWordmark } from "@/components/brand/BrandWordmark";
import { Container } from "@/components/layout/Container";
import { bookingHref } from "@/components/layout/navigationItems";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

const footerGroups = [
  {
    id: "explore",
    labelKey: "footer.explore",
    links: [
      { href: "#global-reach", id: "explore", labelKey: "footer.explore" },
      { href: "#global-reach", id: "destinations", labelKey: "footer.destinations" },
      { href: "#cabins", id: "cabins", labelKey: "footer.cabins" },
    ],
  },
  {
    id: "travel",
    labelKey: "footer.travel",
    links: [
      { href: bookingHref, id: "book-flight", labelKey: "navigation.bookFlight" },
      { href: "#top", id: "manage-booking", labelKey: "footer.manageBooking" },
    ],
  },
  {
    id: "company",
    labelKey: "footer.company",
    links: [
      { href: "#journey-experience", id: "about", labelKey: "footer.about" },
    ],
  },
  {
    id: "support",
    labelKey: "footer.support",
    links: [{ href: "#top", id: "help", labelKey: "footer.help" }],
  },
  {
    id: "legal",
    labelKey: "footer.legal",
    links: [
      { href: "#top", id: "privacy", labelKey: "footer.privacy" },
      { href: "#top", id: "terms", labelKey: "footer.terms" },
    ],
  },
] as const satisfies readonly {
  id: string;
  labelKey: TranslationKey;
  links: readonly { href: string; id: string; labelKey: TranslationKey }[];
}[];

const Footer = () => {
  const { t } = useLanguage();

  return (
  <footer className="border-t border-border/80 bg-surface/25 py-section-sm">
    <Container>
      <div className="grid gap-12 border-b border-border/80 pb-12 lg:grid-cols-[minmax(0,1.5fr)_minmax(0,3fr)]">
        <div className="max-w-sm">
          <BrandWordmark className="text-sm" />
          <p className="mt-6 text-body-sm text-muted-foreground">
            {t("footer.description")}
          </p>
        </div>
        <div className="grid grid-cols-2 gap-x-6 gap-y-10 sm:grid-cols-3 lg:grid-cols-5">
          {footerGroups.map((group) => (
            <section key={group.id}>
              <h2 className="text-label text-foreground">{t(group.labelKey)}</h2>
              <ul className="mt-4 space-y-3">
                {group.links.map((link) => (
                  <li key={link.id}>
                    <a
                      className="rounded-sm text-body-sm text-muted-foreground outline-none transition-colors hover:text-brand focus-visible:ring-2 focus-visible:ring-focus"
                      href={link.href}
                    >
                      {t(link.labelKey)}
                    </a>
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>
      </div>
      <div className="flex flex-col gap-3 pt-6 text-caption text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
        <p>{t("footer.copyright")}</p>
        <p>{t("footer.designed")}</p>
      </div>
    </Container>
  </footer>
  );
};

export { Footer };
