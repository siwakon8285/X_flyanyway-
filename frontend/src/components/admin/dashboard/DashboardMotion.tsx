"use client";

import type { ReactNode } from "react";
import { useRef } from "react";

import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";
import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";

type DashboardMotionProps = {
  children: ReactNode;
  dataKey: string;
};

const DashboardMotion = ({ children, dataKey }: DashboardMotionProps) => {
  const root = useRef<HTMLDivElement>(null);
  const reducedMotion = useReducedMotion();

  useGSAP(
    () => {
      const element = root.current;
      if (!element || reducedMotion) return;
      const select = gsap.utils.selector(element);
      const timeline = gsap.timeline({ defaults: { ease: gsapEasings.enter } });

      timeline
        .fromTo(select("[data-exec-eyebrow]"), { autoAlpha: 0, y: 10 }, { autoAlpha: 1, duration: motionDurations.ui, y: 0 }, 0.1)
        .fromTo(select("[data-exec-title-line]"), { autoAlpha: 0, yPercent: 82 }, { autoAlpha: 1, duration: motionDurations.reveal, stagger: 0.08, yPercent: 0 }, 0.15)
        .fromTo(select("[data-exec-intro]"), { autoAlpha: 0, y: 14 }, { autoAlpha: 1, duration: motionDurations.reveal, y: 0 }, 0.3)
        .fromTo(select("[data-exec-route-path]"), { strokeDasharray: 1, strokeDashoffset: 1 }, { duration: 0.8, strokeDashoffset: 0 }, 0.42)
        .fromTo(select("[data-exec-mark]"), { autoAlpha: 0, x: 12 }, { autoAlpha: 1, duration: motionDurations.reveal, x: 0 }, 0.48)
        .fromTo(select("[data-exec-filters]"), { autoAlpha: 0, y: 12 }, { autoAlpha: 1, duration: motionDurations.reveal, y: 0 }, 0.58);

      return () => timeline.kill();
    },
    { dependencies: [reducedMotion], revertOnUpdate: true, scope: root },
  );

  useGSAP(
    () => {
      const element = root.current;
      if (!element || reducedMotion || !dataKey) return;
      const select = gsap.utils.selector(element);
      const paths = gsap.utils.toArray<SVGPathElement>("[data-chart-line]", element);
      const sections = gsap.utils.toArray<HTMLElement>("[data-exec-reveal]", element);

      gsap.fromTo(select("[data-dashboard-data]"), { autoAlpha: 0.58, y: 8 }, { autoAlpha: 1, duration: 0.5, ease: gsapEasings.standard, y: 0 });
      gsap.fromTo(select("[data-chart-area]"), { autoAlpha: 0.16, scaleX: 0.18, transformOrigin: "left center" }, { autoAlpha: 1, duration: 0.78, ease: gsapEasings.enter, scaleX: 1 });
      paths.forEach((path) => {
        const length = path.getTotalLength?.() ?? 0;
        if (!length) return;
        gsap.fromTo(path, { strokeDasharray: length, strokeDashoffset: length }, { duration: 0.85, ease: gsapEasings.enter, strokeDashoffset: 0 });
      });
      gsap.fromTo(select("[data-demand-pulse]"), { scaleY: 0, transformOrigin: "center bottom" }, { duration: 0.62, ease: gsapEasings.enter, scaleY: 1, stagger: 0.025 });
      gsap.fromTo(select(".exec-data-point, .exec-latest-point"), { autoAlpha: 0, scale: 0.35, transformOrigin: "center" }, { autoAlpha: 1, duration: 0.36, ease: gsapEasings.enter, scale: 1, stagger: 0.018 });
      gsap.fromTo(select("[data-route-line]"), { scaleX: 0, transformOrigin: "left center" }, { duration: 0.65, ease: gsapEasings.enter, scaleX: 1, stagger: 0.065 });
      gsap.fromTo(select("[data-cabin-segment]"), { autoAlpha: 0, rotate: -8, transformOrigin: "center" }, { autoAlpha: 1, duration: 0.55, ease: gsapEasings.enter, rotate: 0, stagger: 0.06 });
      gsap.fromTo(select("[data-occupancy-fill]"), { scaleX: 0, transformOrigin: "left center" }, { duration: 0.8, ease: gsapEasings.cinematic, scaleX: 1 });
      sections.forEach((section) => {
        const details = section.querySelectorAll("[data-exec-follow]");
        gsap.timeline({
          scrollTrigger: { trigger: section, start: "top 88%", once: true },
          defaults: { ease: gsapEasings.enter },
        })
          .fromTo(section, { autoAlpha: 0, y: 28 }, { autoAlpha: 1, duration: motionDurations.reveal, y: 0 })
          .fromTo(details, { autoAlpha: 0, y: 12 }, { autoAlpha: 1, duration: motionDurations.ui, stagger: 0.07, y: 0 }, 0.18);
      });
    },
    { dependencies: [dataKey, reducedMotion], revertOnUpdate: true, scope: root },
  );

  return <div ref={root}>{children}</div>;
};

export { DashboardMotion };
