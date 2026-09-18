"use client";

import { useEffect, useState, type RefObject } from "react";

import { cn } from "@/lib/utils/cn";

type StoryMobilePaginationItem = {
  id: string;
  label: string;
};

type StoryMobilePaginationProps = {
  ariaLabel: string;
  containerRef: RefObject<HTMLElement | null>;
  items: readonly StoryMobilePaginationItem[];
  className?: string;
};

const StoryMobilePagination = ({
  ariaLabel,
  className,
  containerRef,
  items,
}: StoryMobilePaginationProps) => {
  const [activeIndex, setActiveIndex] = useState(0);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || !("IntersectionObserver" in window)) return;

    const slides = Array.from(
      container.querySelectorAll<HTMLElement>("[data-story-mobile-slide]"),
    );
    if (slides.length === 0) return;

    const observer = new IntersectionObserver(
      (entries) => {
        const mostVisible = entries
          .filter((entry) => entry.isIntersecting)
          .sort((left, right) => right.intersectionRatio - left.intersectionRatio)[0];
        if (!mostVisible) return;

        const nextIndex = slides.indexOf(mostVisible.target as HTMLElement);
        if (nextIndex >= 0) setActiveIndex(nextIndex);
      },
      { root: container, threshold: 0.6 },
    );

    slides.forEach((slide) => observer.observe(slide));
    return () => observer.disconnect();
  }, [containerRef, items.length]);

  const goTo = (index: number) => {
    const container = containerRef.current;
    const slide = container?.querySelectorAll<HTMLElement>(
      "[data-story-mobile-slide]",
    )[index];
    if (!container || !slide) return;

    setActiveIndex(index);
    if (typeof container.scrollTo !== "function") return;

    const prefersReducedMotion = window.matchMedia?.(
      "(prefers-reduced-motion: reduce)",
    ).matches;
    container.scrollTo({
      behavior: prefersReducedMotion ? "auto" : "smooth",
      left: slide.offsetLeft,
    });
  };

  return (
    <nav
      aria-label={ariaLabel}
      className={cn(
        "pointer-events-none absolute inset-x-0 bottom-4 z-30 flex justify-center",
        className,
      )}
      data-story-mobile-pagination
    >
      <div className="pointer-events-auto flex items-center gap-1.5">
        {items.map((item, index) => {
          const isActive = index === activeIndex;
          return (
            <button
              aria-current={isActive ? "step" : undefined}
              aria-label={item.label}
              className="group inline-flex min-h-11 min-w-11 items-center justify-center rounded-full p-1 outline-none focus-visible:ring-2 focus-visible:ring-focus"
              key={item.id}
              onClick={() => goTo(index)}
              type="button"
            >
              <span
                aria-hidden="true"
                className={cn(
                  "block rounded-full transition-all duration-200 motion-reduce:transition-none",
                  isActive
                    ? "h-1.5 w-7 bg-brand shadow-[0_0_8px_rgba(255,212,0,0.45)]"
                    : "h-1.5 w-1.5 bg-muted-foreground/45 group-hover:bg-muted-foreground/75",
                )}
              />
            </button>
          );
        })}
      </div>
    </nav>
  );
};

export type { StoryMobilePaginationItem };
export { StoryMobilePagination };
