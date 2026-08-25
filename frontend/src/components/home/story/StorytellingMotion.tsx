"use client";

import type { ReactNode } from "react";
import { useRef } from "react";

import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { motionMediaQueries } from "@/lib/motion/scroll";

type StorytellingMotionProps = {
  children: ReactNode;
};

const cinematicCanvasMediaQuery =
  `${motionMediaQueries.desktop} and (min-height: 52rem)`;
const serviceMotionMediaQueries = {
  desktop: motionMediaQueries.desktop,
  mobile: "(max-width: 47.999rem)",
  tablet: "(min-width: 48rem) and (max-width: 63.999rem)",
} as const;

const StorytellingMotion = ({ children }: StorytellingMotionProps) => {
  const root = useRef<HTMLDivElement>(null);
  const reducedMotion = useReducedMotion();

  useGSAP(
    () => {
      const element = root.current;
      if (!element || reducedMotion) return;

      const generalMediaQueries = gsap.matchMedia();

      generalMediaQueries.add(motionMediaQueries.parallax, () => {
        const globalStory = element.querySelector<HTMLElement>("[data-global-story]");
        const cabinStory = element.querySelector<HTMLElement>("[data-cabin-story]");
        const cabinFrame = element.querySelector<HTMLElement>("[data-cabin-frame]");
        const cabinStack = element.querySelector<HTMLElement>("[data-cabin-stage-stack]");
        const cabinStages = gsap.utils.toArray<HTMLElement>(
          "[data-cabin-stage]",
          element,
        );
        const journeyStory = element.querySelector<HTMLElement>("[data-journey-story]");
        if (globalStory) {
          const globalCopy = globalStory.querySelector<HTMLElement>("[data-global-copy]");
          const globalMetric = globalStory.querySelector<HTMLElement>(
            "[data-global-metric]",
          );
          const globalVisual = globalStory.querySelector<HTMLElement>(
            "[data-global-visual]",
          );
          const routePaths = gsap.utils.toArray<SVGPathElement>(
            "[data-route-path]",
            globalStory,
          );
          const worldRegions = gsap.utils.toArray<SVGPathElement>(
            "[data-world-region]",
            globalStory,
          );
          const routeNodes = gsap.utils.toArray<SVGGElement>(
            "[data-route-node]",
            globalStory,
          );
          const regionLabels = gsap.utils.toArray<HTMLElement>(
            "[data-global-regions] li",
            globalStory,
          );

          gsap
            .timeline({
              defaults: { ease: "none" },
              scrollTrigger: {
                end: "bottom 34%",
                invalidateOnRefresh: true,
                scrub: 0.65,
                start: "top 76%",
                trigger: globalStory,
              },
            })
            .fromTo(globalCopy, { opacity: 0.42, y: 36 }, { opacity: 1, y: 0 }, 0)
            .fromTo(
              globalVisual,
              { opacity: 0.58, scale: 0.96, y: 32 },
              { opacity: 1, scale: 1, y: 0 },
              0,
            )
            .fromTo(
              globalMetric,
              { opacity: 0.35, scale: 0.92, y: 18 },
              { opacity: 1, scale: 1, y: 0 },
              0.08,
            )
            .fromTo(
              worldRegions,
              { opacity: 0.12, scale: 0.97, transformOrigin: "center" },
              { opacity: 1, scale: 1, stagger: 0.045 },
              0.04,
            )
            .fromTo(
              routePaths,
              { strokeDasharray: 1, strokeDashoffset: 1 },
              { stagger: 0.08, strokeDashoffset: 0 },
              0.08,
            )
            .fromTo(
              routeNodes,
              { opacity: 0, scale: 0, transformOrigin: "center" },
              { opacity: 1, scale: 1, stagger: 0.055 },
              0.26,
            )
            .fromTo(
              regionLabels,
              { opacity: 0.3, y: 10 },
              { opacity: 1, stagger: 0.05, y: 0 },
              0.38,
            );
        }

        if (cabinStory && cabinFrame && cabinStack && cabinStages.length === 4) {
          const cabinAperture = cabinStory.querySelector<HTMLElement>(
            "[data-cabin-aperture]",
          );
          const cabinLight = cabinStory.querySelector<HTMLElement>(
            "[data-cabin-light]",
          );
          const cabinProgress = cabinStory.querySelector<HTMLElement>(
            "[data-cabin-progress]",
          );
          const cabinAtmospheres = gsap.utils.toArray<HTMLElement>(
            "[data-cabin-atmosphere]",
            cabinStory,
          );
          const progressLine = cabinStory.querySelector<HTMLElement>(
            "[data-cabin-progress-line]",
          );

          gsap.set(cabinFrame, { minHeight: "100svh" });
          gsap.set(cabinStack, { height: "clamp(24rem, 55svh, 42rem)" });
          gsap.set(cabinStages, {
            inset: 0,
            minHeight: 0,
            position: "absolute",
          });
          gsap.set(cabinStages.slice(1), { opacity: 0, y: 40 });
          gsap.set(cabinProgress, { display: "flex" });
          gsap.set(progressLine, { scaleX: 0 });
          gsap.set(cabinLight, { opacity: 0.48, xPercent: 5 });
          gsap.set(cabinAtmospheres[0], { opacity: 0.7 });
          gsap.set(cabinAtmospheres.slice(1), { opacity: 0 });

          const cabinTimeline = gsap.timeline({
            defaults: { ease: "none" },
            scrollTrigger: {
              end: () => {
                const distance = window.matchMedia(motionMediaQueries.desktop).matches
                  ? 2.2
                  : 1.6;

                return `+=${Math.round(window.innerHeight * distance)}`;
              },
              invalidateOnRefresh: true,
              pin: cabinFrame,
              scrub: 0.7,
              start: "top top",
              trigger: cabinStory,
            },
          });

          cabinTimeline.to(
            progressLine,
            { duration: 3.8, scaleX: 1 },
            0,
          );

          cabinStages.slice(1).forEach((stage, index) => {
            const previousStage = cabinStages[index];
            const previousAtmosphere = cabinAtmospheres[index];
            const nextAtmosphere = cabinAtmospheres[index + 1];
            const transitionStart = index + 0.72;

            cabinTimeline
              .to(
                previousStage,
                { duration: 0.34, opacity: 0, y: -32 },
                transitionStart,
              )
              .to(
                stage,
                { duration: 0.42, opacity: 1, y: 0 },
                transitionStart + 0.18,
              )
              .to(
                cabinAperture,
                {
                  duration: 0.72,
                  scale: 1 + (index + 1) * 0.035,
                  xPercent: -(index + 1) * 2,
                },
                transitionStart,
              )
              .to(
                cabinLight,
                {
                  duration: 0.72,
                  opacity: 0.82,
                  xPercent: -(index + 1) * 3,
                },
                transitionStart,
              )
              .to(
                previousAtmosphere,
                { duration: 0.46, opacity: 0 },
                transitionStart,
              )
              .to(
                nextAtmosphere,
                { duration: 0.54, opacity: 0.72 },
                transitionStart + 0.12,
              );
          });
        }

        if (journeyStory) {
          const journeyCopy = journeyStory.querySelector<HTMLElement>(
            "[data-journey-copy]",
          );
          const journeyValues = gsap.utils.toArray<HTMLElement>(
            "[data-journey-value]",
            journeyStory,
          );
          const journeyCta = journeyStory.querySelector<HTMLElement>(
            "[data-journey-cta]",
          );

          gsap
            .timeline({
              defaults: { ease: "none" },
              scrollTrigger: {
                end: "center 42%",
                invalidateOnRefresh: true,
                scrub: 0.55,
                start: "top 84%",
                trigger: journeyStory,
              },
            })
            .fromTo(
              journeyCopy,
              { opacity: 0.46, y: 32 },
              { opacity: 1, y: 0 },
              0,
            )
            .fromTo(
              journeyValues,
              { opacity: 0.32, x: 28 },
              { opacity: 1, stagger: 0.12, x: 0 },
              0.12,
            )
            .fromTo(
              journeyCta,
              { opacity: 0.38, y: 24 },
              { opacity: 1, y: 0 },
              0.36,
            );
        }

      });

      return () => generalMediaQueries.revert();
    },
    {
      dependencies: [reducedMotion],
      revertOnUpdate: true,
      scope: root,
    },
  );

  useGSAP(
    () => {
      const element = root.current;
      if (!element || reducedMotion) return;

      const cinematicMediaQueries = gsap.matchMedia();

      cinematicMediaQueries.add(cinematicCanvasMediaQuery, () => {
        const aircraftStory = element.querySelector<HTMLElement>(
          "[data-aircraft-story]",
        );
        const aircraft = aircraftStory?.querySelector<HTMLElement>(
          "[data-story-aircraft]",
        );
        const atmosphere = aircraftStory?.querySelector<HTMLElement>(
          "[data-aircraft-atmosphere]",
        );
        const flightCorridor = aircraftStory?.querySelector<HTMLElement>(
          "[data-aircraft-flight-corridor]",
        );
        const route = aircraftStory?.querySelector<SVGPathElement>(
          "[data-aircraft-route]",
        );
        const milestoneCanvas = aircraftStory?.querySelector<HTMLElement>(
          "[data-aircraft-milestone-canvas]",
        );
        const beats = gsap.utils.toArray<HTMLElement>(
          "[data-aircraft-beat]",
          aircraftStory ?? undefined,
        );

        if (
          aircraftStory &&
          aircraft &&
          atmosphere &&
          flightCorridor &&
          route &&
          milestoneCanvas &&
          beats.length === 4
        ) {
          gsap.set(beats, { opacity: 0 });

          const aircraftTimeline = gsap.timeline({
            defaults: { ease: "none" },
            scrollTrigger: {
              end: "bottom bottom",
              invalidateOnRefresh: true,
              scrub: 0.72,
              start: "top top",
              trigger: flightCorridor,
            },
          });

          aircraftTimeline
            .fromTo(
              atmosphere,
              { scale: 0.96, yPercent: 5 },
              { duration: 4, scale: 1.04, yPercent: -4 },
              0,
            )
            .fromTo(
              route,
              { strokeDasharray: 1, strokeDashoffset: 1 },
              { duration: 4, strokeDashoffset: 0 },
              0,
            )
            .fromTo(
              aircraft,
              {
                rotate: -1.5,
                scale: 0.82,
                x: () => -aircraft.offsetWidth * 1.08,
                yPercent: 7,
              },
              {
                duration: 4,
                rotate: 1,
                scale: 1.04,
                x: () => flightCorridor.clientWidth + aircraft.offsetWidth * 0.08,
                yPercent: -6,
              },
              0,
            );

          const milestoneStarts = [0.58, 1.35, 2.2, 3.05] as const;
          const revealOffsets = [
            { x: -24, y: 0 },
            { x: 0, y: 24 },
            { x: 24, y: 0 },
            { x: 0, y: 24 },
          ] as const;

          beats.forEach((beat, index) => {
            const revealStart = milestoneStarts[index];
            const offset = revealOffsets[index];

            if (index > 0) {
              aircraftTimeline.to(
                beats[index - 1],
                { duration: 0.22, opacity: 0.56 },
                revealStart + 0.16,
              );
            }

            aircraftTimeline.fromTo(
              beat,
              { opacity: 0, x: offset.x, y: offset.y },
              { duration: 0.28, opacity: 1, x: 0, y: 0 },
              revealStart,
            );
          });
        }

      });

      return () => cinematicMediaQueries.revert();
    },
    {
      dependencies: [reducedMotion],
      revertOnUpdate: true,
      scope: root,
    },
  );

  useGSAP(
    () => {
      const element = root.current;
      if (!element || reducedMotion) return;

      const serviceStory = element.querySelector<HTMLElement>(
        "[data-service-story]",
      );
      const serviceEyebrow = serviceStory?.querySelector<HTMLElement>(
        "[data-service-eyebrow]",
      );
      const serviceEyebrowLine = serviceStory?.querySelector<HTMLElement>(
        "[data-service-eyebrow-line]",
      );
      const serviceHeadingLines = gsap.utils.toArray<HTMLElement>(
        "[data-service-heading-line]",
        serviceStory ?? undefined,
      );
      const serviceIntro = serviceStory?.querySelector<HTMLElement>(
        "[data-service-intro]",
      );
      const serviceFrames = gsap.utils.toArray<HTMLElement>(
        "[data-service-image-frame]",
        serviceStory ?? undefined,
      );
      const serviceImages = gsap.utils.toArray<HTMLElement>(
        "[data-service-image]",
        serviceStory ?? undefined,
      );
      const serviceCaptions = gsap.utils.toArray<HTMLElement>(
        "[data-service-caption]",
        serviceStory ?? undefined,
      );
      const serviceCaptionRules = gsap.utils.toArray<HTMLElement>(
        "[data-service-caption-rule]",
        serviceStory ?? undefined,
      );

      if (
        !serviceStory ||
        !serviceEyebrow ||
        !serviceEyebrowLine ||
        serviceHeadingLines.length !== 2 ||
        !serviceIntro ||
        serviceFrames.length !== 2 ||
        serviceImages.length !== 2 ||
        serviceCaptions.length !== 2 ||
        serviceCaptionRules.length !== 2
      ) {
        return;
      }

      const serviceEntranceMediaQueries = gsap.matchMedia();

      serviceEntranceMediaQueries.add(serviceMotionMediaQueries, (context) => {
        const usesEditorialMasks = Boolean(
          context.conditions?.desktop || context.conditions?.tablet,
        );

        gsap
        .timeline({
          defaults: { ease: "power2.out" },
          scrollTrigger: {
            invalidateOnRefresh: true,
            once: true,
            start: "top 82%",
            trigger: serviceStory,
          },
        })
        .fromTo(
          serviceEyebrowLine,
          { opacity: 0.62, scaleX: 0.18 },
          { duration: 0.5, opacity: 1, scaleX: 1 },
          0,
        )
        .fromTo(
          serviceEyebrow,
          { opacity: 0.82, y: 14 },
          { duration: 0.5, opacity: 1, y: 0 },
          0.06,
        )
        .fromTo(
          serviceHeadingLines,
          { opacity: 0.78, y: 30 },
          { duration: 0.7, opacity: 1, stagger: 0.1, y: 0 },
          0.14,
        )
        .fromTo(
          serviceIntro,
          { opacity: 0.74, y: 22 },
          { duration: 0.6, opacity: 1, y: 0 },
          0.46,
        )
        .fromTo(
          serviceFrames[0],
          {
            clipPath: usesEditorialMasks
              ? "inset(12% 0 0 0)"
              : undefined,
            opacity: 0.86,
            y: usesEditorialMasks ? 26 : 18,
          },
          {
            clipPath: usesEditorialMasks ? "inset(0% 0 0 0)" : undefined,
            duration: 1,
            opacity: 1,
            y: 0,
          },
          0.56,
        )
        .fromTo(
          serviceImages[0],
          { scale: usesEditorialMasks ? 1.06 : undefined },
          { duration: 1.2, scale: usesEditorialMasks ? 1 : undefined },
          0.56,
        )
        .fromTo(
          serviceCaptionRules[0],
          { opacity: 0.55, scaleX: 0.18 },
          { duration: 0.58, opacity: 1, scaleX: 1 },
          1.04,
        )
        .fromTo(
          serviceCaptions[0],
          { opacity: 0.72, y: 14 },
          { duration: 0.55, opacity: 1, y: 0 },
          1.12,
        )
        .fromTo(
          serviceFrames[1],
          {
            clipPath: usesEditorialMasks
              ? "inset(0 16% 0 0)"
              : undefined,
            opacity: 0.8,
            x: usesEditorialMasks ? 38 : 20,
          },
          {
            clipPath: usesEditorialMasks ? "inset(0 0% 0 0)" : undefined,
            duration: 0.88,
            opacity: 1,
            x: 0,
          },
          1.08,
        )
        .fromTo(
          serviceImages[1],
          { scale: usesEditorialMasks ? 1.045 : undefined },
          { duration: 1.05, scale: usesEditorialMasks ? 1 : undefined },
          1.08,
        )
        .fromTo(
          serviceCaptionRules[1],
          { opacity: 0.55, scaleX: 0.18 },
          { duration: 0.54, opacity: 1, scaleX: 1 },
          1.54,
        )
        .fromTo(
          serviceCaptions[1],
          { opacity: 0.72, y: 14 },
          { duration: 0.52, opacity: 1, y: 0 },
          1.62,
        );

        return undefined;
      });

      return () => serviceEntranceMediaQueries.revert();
    },
    {
      dependencies: [reducedMotion],
      revertOnUpdate: true,
      scope: root,
    },
  );

  useGSAP(
    () => {
      const element = root.current;
      if (!element || reducedMotion) return;

      const serviceStory = element.querySelector<HTMLElement>(
        "[data-service-story]",
      );
      const serviceParallaxLayers = gsap.utils.toArray<HTMLElement>(
        "[data-service-image-parallax]",
        serviceStory ?? undefined,
      );
      if (!serviceStory || serviceParallaxLayers.length !== 2) return;

      const serviceMediaQueries = gsap.matchMedia();

      serviceMediaQueries.add(
        {
          desktop: serviceMotionMediaQueries.desktop,
          tablet: serviceMotionMediaQueries.tablet,
        },
        (context) => {
          const distance = context.conditions?.desktop ? 3.5 : 2;

          gsap
            .timeline({
              defaults: { ease: "none" },
              scrollTrigger: {
                end: "bottom top",
                invalidateOnRefresh: true,
                scrub: 0.65,
                start: "top bottom",
                trigger: serviceStory,
              },
            })
            .fromTo(
              serviceParallaxLayers[0],
              { yPercent: -distance },
              { yPercent: distance },
              0,
            )
            .fromTo(
              serviceParallaxLayers[1],
              { yPercent: distance * 0.7 },
              { yPercent: distance * -0.7 },
              0,
            );

          return undefined;
        },
      );

      return () => serviceMediaQueries.revert();
    },
    {
      dependencies: [reducedMotion],
      revertOnUpdate: true,
      scope: root,
    },
  );

  return (
    <div className="relative bg-background" id="journey" ref={root}>
      {children}
    </div>
  );
};

export { StorytellingMotion };
export type { StorytellingMotionProps };
