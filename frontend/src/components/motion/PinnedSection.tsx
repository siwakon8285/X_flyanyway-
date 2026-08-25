"use client";

import type { ComponentPropsWithoutRef, ReactNode } from "react";
import { useRef } from "react";

import { ScrollTrigger, gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { motionMediaQueries } from "@/lib/motion/scroll";
import { cn } from "@/lib/utils/cn";

type PinnedSectionProps = Omit<ComponentPropsWithoutRef<"section">, "children"> & {
  children: ReactNode;
};

const PinnedSection = ({ children, className, ...props }: PinnedSectionProps) => {
  const section = useRef<HTMLElement>(null);
  const content = useRef<HTMLDivElement>(null);
  const reducedMotion = useReducedMotion();

  useGSAP(
    () => {
      if (reducedMotion || !section.current || !content.current) return;

      const mediaQueries = gsap.matchMedia();
      mediaQueries.add(motionMediaQueries.desktop, () => {
        ScrollTrigger.create({
          end: () => `+=${Math.min(window.innerHeight * 0.45, 420)}`,
          invalidateOnRefresh: true,
          pin: content.current,
          start: "top 18%",
          trigger: section.current,
        });
      });

      return () => mediaQueries.revert();
    },
    {
      dependencies: [reducedMotion],
      revertOnUpdate: true,
      scope: section,
    },
  );

  return (
    <section className={cn("relative", className)} ref={section} {...props}>
      <div ref={content}>{children}</div>
    </section>
  );
};

export { PinnedSection };
export type { PinnedSectionProps };
