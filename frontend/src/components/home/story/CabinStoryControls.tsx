"use client";

import { useEffect, useState } from "react";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

type CabinMeta = {
  id: "economy" | "premium-economy" | "business" | "first";
  labelKey: TranslationKey;
};

const cabinsMeta: readonly CabinMeta[] = [
  { id: "economy", labelKey: "common.cabins.economy" },
  { id: "premium-economy", labelKey: "common.cabins.premiumEconomy" },
  { id: "business", labelKey: "common.cabins.business" },
  { id: "first", labelKey: "common.cabins.first" },
] as const;

const CabinStoryControls = () => {
  const [activeCabin, setActiveCabin] = useState(0);
  const { t } = useLanguage();

  useEffect(() => {
    if (typeof window === "undefined" || !("IntersectionObserver" in window)) {
      return;
    }

    const stack = document.querySelector<HTMLElement>(
      "[data-cabin-stage-stack]",
    );
    if (!stack) return;

    const cabinElements = stack.querySelectorAll<HTMLElement>(
      "[data-cabin-stage]",
    );
    if (cabinElements.length === 0) return;

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const cabinId = entry.target.getAttribute("data-cabin-id");
            const index = cabinsMeta.findIndex((c) => c.id === cabinId);
            if (index !== -1) {
              setActiveCabin(index);
            }
          }
        });
      },
      {
        root: stack,
        threshold: 0.55,
      },
    );

    cabinElements.forEach((el) => observer.observe(el));

    return () => observer.disconnect();
  }, []);

  const handleCabinClick = (index: number) => {
    const cabinId = cabinsMeta[index]?.id;
    const cabinElement = document.querySelector<HTMLElement>(
      `[data-cabin-id="${cabinId}"]`,
    );
    if (!cabinElement) return;

    cabinElement.scrollIntoView({
      behavior: "smooth",
      block: "nearest",
      inline: "start",
    });
  };

  return (
    <div
      className="mt-6 flex items-center justify-center px-page-gutter md:hidden"
      data-cabin-mobile-nav
    >
      <nav
        aria-label={t("home.cabins.controlLabel")}
        className="flex items-center gap-1.5 rounded-full border border-border/50 bg-[#090c12]/80 px-4 py-2 backdrop-blur-md"
      >
        {cabinsMeta.map((cabin, index) => {
          const isActive = activeCabin === index;
          return (
            <button
              aria-current={isActive ? "step" : undefined}
              aria-label={t("home.cabins.goTo", { label: t(cabin.labelKey) })}
              className="group relative flex min-h-[44px] min-w-[44px] items-center justify-center p-1 focus-visible:outline-none"
              key={cabin.id}
              onClick={() => handleCabinClick(index)}
              type="button"
            >
              <span
                className={`block transition-all duration-300 ${
                  isActive
                    ? "h-1.5 w-6 sm:w-7 rounded-full bg-brand shadow-[0_0_8px_rgba(255,212,0,0.5)]"
                    : "h-1.5 w-1.5 rounded-full bg-muted-foreground/35 hover:bg-muted-foreground/60"
                }`}
              />
            </button>
          );
        })}
      </nav>
    </div>
  );
};

export { CabinStoryControls, cabinsMeta };
