"use client";

import { useEffect, useState } from "react";

type StageMeta = {
  id: "discover" | "book" | "fly" | "arrive" | "beyond";
  name: string;
  number: string;
};

const stages: readonly StageMeta[] = [
  { id: "discover", name: "Discover", number: "01" },
  { id: "book", name: "Book", number: "02" },
  { id: "fly", name: "Fly", number: "03" },
  { id: "arrive", name: "Arrive", number: "04" },
  { id: "beyond", name: "Beyond", number: "05" },
] as const;

const HorizontalJourneyControls = () => {
  const [activeStage, setActiveStage] = useState(0);

  useEffect(() => {
    if (typeof window === "undefined" || !("IntersectionObserver" in window)) {
      return;
    }

    const track = document.querySelector<HTMLElement>("[data-journey-track]");
    if (!track) return;

    const stageElements = track.querySelectorAll<HTMLElement>(
      "[data-journey-stage]",
    );
    if (stageElements.length === 0) return;

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const stageId = entry.target.getAttribute("data-journey-stage");
            const index = stages.findIndex((s) => s.id === stageId);
            if (index !== -1) {
              setActiveStage(index);
              stageElements.forEach((el, elIdx) => {
                el.setAttribute("data-active", String(elIdx === index));
              });
            }
          }
        });
      },
      {
        root: track,
        threshold: 0.55,
      },
    );

    stageElements.forEach((el) => observer.observe(el));

    return () => observer.disconnect();
  }, []);

  const handleStageClick = (index: number) => {
    const stageId = stages[index]?.id;
    const stageElement = document.querySelector<HTMLElement>(
      `[data-journey-stage="${stageId}"]`,
    );
    if (!stageElement) return;

    stageElement.scrollIntoView({
      behavior: "smooth",
      block: "nearest",
      inline: "start",
    });
  };

  return (
    <div
      className="mt-6 flex items-center justify-center px-page-gutter md:hidden"
      data-journey-mobile-nav
    >
      <nav
        aria-label="Journey stages"
        className="flex items-center gap-1.5 rounded-full border border-border/50 bg-[#090c12]/80 px-4 py-2 backdrop-blur-md"
      >
        {stages.map((stage, index) => {
          const isActive = activeStage === index;
          return (
            <button
              aria-current={isActive ? "step" : undefined}
              aria-label={`Go to ${stage.name}`}
              className="group relative flex min-h-[44px] min-w-[44px] items-center justify-center p-1 focus-visible:outline-none"
              key={stage.id}
              onClick={() => handleStageClick(index)}
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

export { HorizontalJourneyControls, stages };
