"use client";

import type { ReactNode } from "react";
import { useRef } from "react";

import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";
import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { motionMediaQueries } from "@/lib/motion/scroll";

type HeroMotionProps = {
  children: ReactNode;
};

const HeroMotion = ({ children }: HeroMotionProps) => {
  const hero = useRef<HTMLElement>(null);
  const reducedMotion = useReducedMotion();

  useGSAP(
    () => {
      const element = hero.current;
      if (!element || reducedMotion) return;

      const select = gsap.utils.selector(element);
      const headlineWords = select("[data-hero-headline] .word");
      const timeline = gsap.timeline({ defaults: { ease: gsapEasings.enter } });

      timeline
        .fromTo(
          select("[data-hero-line]"),
          { scaleX: 0 },
          { duration: motionDurations.ui, scaleX: 1 },
        )
        .fromTo(
          select("[data-hero-media-frame]"),
          { clipPath: "inset(0 0 100% 0)" },
          {
            clipPath: "inset(0 0 0% 0)",
            duration: motionDurations.reveal,
          },
          0.18,
        )
        .fromTo(
          select("[data-hero-media-frame] img"),
          { scale: 1.05 },
          {
            duration: motionDurations.cinematic,
            ease: gsapEasings.cinematic,
            scale: 1,
          },
          0.24,
        )
        .fromTo(
          select("[data-hero-eyebrow]"),
          { autoAlpha: 0, y: 12 },
          { autoAlpha: 1, duration: motionDurations.ui, y: 0 },
          0.72,
        );

      if (headlineWords.length) {
        timeline.fromTo(
          headlineWords,
          { autoAlpha: 0, yPercent: 60 },
          {
            autoAlpha: 1,
            duration: motionDurations.reveal,
            stagger: 0.055,
            yPercent: 0,
          },
          0.86,
        );
      }

      timeline
        .fromTo(
          select("[data-hero-details]"),
          { autoAlpha: 0, y: 18 },
          { autoAlpha: 1, duration: motionDurations.reveal, y: 0 },
          1.5,
        )
        .fromTo(
          select("[data-hero-scroll-label]"),
          { autoAlpha: 0, y: 8 },
          { autoAlpha: 1, duration: motionDurations.ui, y: 0 },
          1.88,
        );

      const mediaQuery = gsap.matchMedia();

      mediaQuery.add(motionMediaQueries.parallax, () => {
        gsap
          .timeline({
            scrollTrigger: {
              end: "bottom top",
              scrub: 0.5,
              start: "top top",
              trigger: element,
            },
          })
          .to(
            select("[data-hero-media-frame]"),
            { scale: 1.025, yPercent: 4 },
            0,
          )
          .to(select("[data-hero-content]"), { autoAlpha: 0.62, y: -28 }, 0)
          .to(select("[data-hero-scroll-cue]"), { autoAlpha: 0 }, 0);
      });

      return () => mediaQuery.revert();
    },
    {
      dependencies: [reducedMotion],
      revertOnUpdate: true,
      scope: hero,
    },
  );

  return (
    <section
      aria-labelledby="hero-heading"
      className="relative isolate min-h-svh overflow-hidden bg-background"
      id="experience"
      ref={hero}
    >
      {children}
    </section>
  );
};

export { HeroMotion };
export type { HeroMotionProps };
