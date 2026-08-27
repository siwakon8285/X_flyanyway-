"use client";

import { useEffect, useState } from "react";

type BeatMeta = {
  id: "global" | "personal" | "premium" | "seamless";
  label: string;
};

const beats: readonly BeatMeta[] = [
  { id: "global", label: "Global" },
  { id: "personal", label: "Personal" },
  { id: "premium", label: "Premium" },
  { id: "seamless", label: "Seamless" },
] as const;

const AircraftNarrativeControls = () => {
  const [activeBeat, setActiveBeat] = useState(0);

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
        aria-label="Aircraft narrative milestones"
        className="flex items-center gap-1.5 rounded-full border border-border/50 bg-[#090c12]/80 px-4 py-2 backdrop-blur-md"
      >
        {beats.map((beat, index) => {
          const isActive = activeBeat === index;
          return (
            <button
              aria-current={isActive ? "step" : undefined}
              aria-label={`Go to ${beat.label}`}
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
