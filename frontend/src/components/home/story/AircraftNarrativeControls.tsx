"use client";

import { useEffect, useState } from "react";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

type BeatMeta = {
  id: "global" | "personal" | "premium" | "seamless";
  labelKey: TranslationKey;
};

const beats: readonly BeatMeta[] = [
  { id: "global", labelKey: "home.aircraft.global.label" },
  { id: "personal", labelKey: "home.aircraft.personal.label" },
  { id: "premium", labelKey: "home.aircraft.premium.label" },
  { id: "seamless", labelKey: "home.aircraft.seamless.label" },
] as const;

const AircraftNarrativeControls = () => {
  const [activeBeat, setActiveBeat] = useState(0);
  const { t } = useLanguage();

  useEffect(() => {
    if (typeof window === "undefined" || !("IntersectionObserver" in window)) {
      return;
    }

    const canvas = document.querySelector<HTMLElement>(
      "[data-aircraft-milestone-canvas]",
    );
    if (!canvas) return;

    const beatElements = canvas.querySelectorAll<HTMLElement>(
      "[data-aircraft-beat]",
    );
    if (beatElements.length === 0) return;

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const beatId = entry.target.getAttribute("data-aircraft-zone");
            const index = beats.findIndex((b) => b.id === beatId);
            if (index !== -1) {
              setActiveBeat(index);
            }
          }
        });
      },
      {
        root: canvas,
        threshold: 0.55,
      },
    );

    beatElements.forEach((el) => observer.observe(el));

    return () => observer.disconnect();
  }, []);

  const handleBeatClick = (index: number) => {
    const beatId = beats[index]?.id;
    const beatElement = document.querySelector<HTMLElement>(
      `[data-aircraft-zone="${beatId}"]`,
    );
    if (!beatElement) return;

    beatElement.scrollIntoView({
      behavior: "smooth",
      block: "nearest",
      inline: "start",
    });
  };

  return (
    <div
      className="mt-6 flex items-center justify-center px-page-gutter motion-safe:[@media(min-height:52rem)]:lg:hidden"
      data-aircraft-mobile-nav
    >
      <nav
        aria-label={t("home.aircraft.controlLabel")}
        className="flex items-center gap-1.5 rounded-full border border-border/50 bg-[#090c12]/80 px-4 py-2 backdrop-blur-md"
      >
        {beats.map((beat, index) => {
          const isActive = activeBeat === index;
          return (
            <button
              aria-current={isActive ? "step" : undefined}
              aria-label={t("home.aircraft.goTo", { label: t(beat.labelKey) })}
              className="group relative flex min-h-[44px] min-w-[44px] items-center justify-center p-1 focus-visible:outline-none"
              key={beat.id}
              onClick={() => handleBeatClick(index)}
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

export { AircraftNarrativeControls, beats };
