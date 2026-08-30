"use client";

import type { ReactNode } from "react";
import { useRef } from "react";

import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { motionMediaQueries } from "@/lib/motion/scroll";
import {
  journeyDepthRoles,
  journeyMotionTiming,
  journeyPromotionBeats,
  journeyStageSlots,
} from "@/components/home/story/journeyStoryModel";

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
        const cabinImages = gsap.utils.toArray<HTMLElement>(
          "[data-cabin-image]",
          element,
        );
        const journeyStory = element.querySelector<HTMLElement>("[data-journey-story]");
        if (globalStory) {
          const globalLine = globalStory.querySelector<HTMLElement>("[data-global-line]");
          const globalEyebrowText = globalStory.querySelector<HTMLElement>(
            "[data-global-eyebrow-text]",
          );
          const globalHeadingLines = gsap.utils.toArray<HTMLElement>(
            "[data-global-heading-line]",
            globalStory,
          );
          const globalBody = globalStory.querySelector<HTMLElement>(
            "[data-global-body]",
          );
          const globalMetric = globalStory.querySelector<HTMLElement>(
            "[data-global-metric]",
          );
          const globalVisual = globalStory.querySelector<HTMLElement>(
            "[data-global-visual]",
          );
          const globalImage = globalStory.querySelector<HTMLElement>(
            "[data-global-image]",
          );
          const routePath = globalStory.querySelector<SVGPathElement>(
            "[data-route-path]",
          );
          const hubsPill = globalStory.querySelector<HTMLElement>(
            "[data-global-hubs-pill]",
          );

          if (globalLine) gsap.set(globalLine, { scaleX: 0, transformOrigin: "left center" });
          if (globalEyebrowText) gsap.set(globalEyebrowText, { opacity: 0, y: 6 });
          if (globalHeadingLines.length > 0) {
            gsap.set(globalHeadingLines, { opacity: 0, y: 24 });
          }
          if (globalBody) gsap.set(globalBody, { opacity: 0, y: 14 });
          if (hubsPill) gsap.set(hubsPill, { opacity: 0, y: 12 });
          if (globalMetric) gsap.set(globalMetric, { opacity: 0, y: 18 });

          const globalTl = gsap.timeline({
            defaults: { ease: "power2.out" },
            scrollTrigger: {
              end: "bottom 30%",
              invalidateOnRefresh: true,
              scrub: 0.7,
              start: "top 80%",
              trigger: globalStory,
            },
          });

          if (globalLine) {
            globalTl.to(globalLine, { duration: 0.25, scaleX: 1 }, 0);
          }
          if (globalEyebrowText) {
            globalTl.to(globalEyebrowText, { duration: 0.22, opacity: 1, y: 0 }, 0.05);
          }

          if (globalHeadingLines.length > 0) {
            globalTl.to(
              globalHeadingLines,
              { duration: 0.45, opacity: 1, stagger: 0.1, y: 0 },
              0.08,
            );
          }

          if (globalBody) {
            globalTl.to(globalBody, { duration: 0.4, opacity: 1, y: 0 }, 0.2);
          }

          if (hubsPill) {
            globalTl.to(hubsPill, { duration: 0.35, opacity: 1, y: 0 }, 0.28);
          }

          // Visual Container & Photographic Image Settle
          globalTl.fromTo(
            globalVisual,
            { opacity: 0.65, y: 26 },
            { duration: 0.8, opacity: 1, y: 0 },
            0,
          );

          if (globalImage) {
            globalTl.fromTo(
              globalImage,
              { scale: 1.06, y: 14 },
              { duration: 1.0, ease: "none", scale: 1.0, y: -8 },
              0,
            );
          }

          if (globalMetric) {
            globalTl.to(globalMetric, { duration: 0.45, opacity: 1, y: 0 }, 0.32);
          }

          if (routePath) {
            globalTl.fromTo(
              routePath,
              { opacity: 0 },
              { duration: 0.4, opacity: 0.85 },
              0.36,
            );
          }
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
          gsap.set(cabinStages[0], { opacity: 1, y: 0 });
          gsap.set(cabinStages.slice(1), { opacity: 0, y: 32 });

          // Initialize desktop media overlapping layers
          if (cabinImages.length === 4) {
            gsap.set(cabinImages[0], { opacity: 1, scale: 1 });
            gsap.set(cabinImages.slice(1), { opacity: 0, scale: 1.03 });
          }

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
            const previousImage = cabinImages[index];
            const nextImage = cabinImages[index + 1];
            const transitionStart = index + 0.72;

            cabinTimeline
              .to(
                previousStage,
                { duration: 0.32, opacity: 0, y: -24 },
                transitionStart,
              );

            if (previousImage) {
              cabinTimeline.to(
                previousImage,
                { duration: 0.38, opacity: 0, scale: 1.015 },
                transitionStart,
              );
            }

            cabinTimeline
              .to(
                stage,
                { duration: 0.40, opacity: 1, y: 0 },
                transitionStart + 0.14,
              );

            if (nextImage) {
              cabinTimeline.fromTo(
                nextImage,
                { opacity: 0, scale: 1.035 },
                { duration: 0.44, opacity: 1, scale: 1, ease: "power2.out" },
                transitionStart + 0.08,
              );
            }

            cabinTimeline
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

  useGSAP(
    () => {
      const element = root.current;
      if (!element || reducedMotion) return;

      const layeredStory = element.querySelector<HTMLElement>("[data-layered-story]");
      if (!layeredStory) return;

      const viewport = layeredStory.querySelector<HTMLElement>("[data-layered-viewport]");
      const copies = gsap.utils.toArray<HTMLElement>("[data-layered-copy]", layeredStory);
      const leftStage = layeredStory.querySelector<HTMLElement>(
        '[data-journey-stage="left"]',
      );
      const rightStage = layeredStory.querySelector<HTMLElement>(
        '[data-journey-stage="right"]',
      );
      const stageCards = {
        left: gsap.utils.toArray<HTMLElement>("[data-journey-card]", leftStage ?? undefined),
        right: gsap.utils.toArray<HTMLElement>("[data-journey-card]", rightStage ?? undefined),
      };

      if (
        !viewport ||
        copies.length !== 3 ||
        stageCards.left.length !== 4 ||
        stageCards.right.length !== 4
      ) {
        return;
      }

      const mediaQueries = gsap.matchMedia();

      mediaQueries.add(motionMediaQueries.desktop, () => {
        const slotVars = (
          side: keyof typeof stageCards,
          role: (typeof journeyDepthRoles)[number],
        ) => {
          const slot = journeyStageSlots[side][role];
          const insetProperty = side === "left" ? "left" : "right";

          return {
            [insetProperty]: slot.inset,
            filter: slot.filter,
            height: slot.height,
            opacity: slot.opacity,
            top: slot.top,
            zIndex: slot.zIndex,
          };
        };

        (Object.keys(stageCards) as Array<keyof typeof stageCards>).forEach(
          (side) => {
            const cards = stageCards[side];

            gsap.set(cards, {
              clearProps: "transform",
              willChange: "top, height, left, right, opacity, filter",
            });
            gsap.set(cards[0], slotVars(side, "front"));
            gsap.set(cards[1], slotVars(side, "queued"));
            gsap.set(cards[2], slotVars(side, "deep"));
            gsap.set(cards[3], slotVars(side, "off-deck"));
          },
        );

        gsap.set(copies, { opacity: 0, y: 14 });
        gsap.set(copies[0], { opacity: 1, y: 0 });

        const tl = gsap.timeline({
          defaults: { ease: journeyMotionTiming.interpolationEase },
          scrollTrigger: {
            end: () =>
              `+=${Math.round(
                window.innerHeight * journeyMotionTiming.scrollDistanceVh,
              )}`,
            invalidateOnRefresh: true,
            pin: viewport,
            scrub: journeyMotionTiming.scrubSmoothing,
            start: "top top",
            trigger: layeredStory,
          },
        });
        let activeCopyIndex = 0;

        journeyPromotionBeats.forEach((beat, beatIndex) => {
          const promotion = beat.promotesStackIndex;
          const start = journeyMotionTiming.pairStarts[beatIndex];

          (["left", "right"] as const).forEach((side) => {
            const cards = stageCards[side];
            const outgoing = cards[(promotion - 1 + cards.length) % cards.length];
            const incoming = cards[promotion];
            const queued = cards[(promotion + 1) % cards.length];
            const deep = cards[(promotion + 2) % cards.length];

            tl.to(
                outgoing,
                {
                  duration: journeyMotionTiming.promotionDuration,
                  ease: journeyMotionTiming.interpolationEase,
                  ...slotVars(side, "off-deck"),
                },
                start,
              )
              .to(
                incoming,
                {
                  duration: journeyMotionTiming.promotionDuration,
                  ease: journeyMotionTiming.interpolationEase,
                  ...slotVars(side, "front"),
                },
                start,
              )
              .to(
                queued,
                {
                  duration: journeyMotionTiming.promotionDuration,
                  ease: journeyMotionTiming.interpolationEase,
                  ...slotVars(side, "queued"),
                },
                start,
              )
              .to(
                deep,
                {
                  duration: journeyMotionTiming.promotionDuration,
                  ease: journeyMotionTiming.interpolationEase,
                  ...slotVars(side, "deep"),
                },
                start,
              );
          });

          const nextCopyIndex = activeCopyIndex + 1;
          const copyStart = start + journeyMotionTiming.copyChangeOffset;

          tl.to(
            copies[activeCopyIndex],
            { duration: 0.28, opacity: 0, y: -12 },
            copyStart,
          ).to(
            copies[nextCopyIndex],
            { duration: 0.32, ease: "power2.out", opacity: 1, y: 0 },
            copyStart + 0.14,
          );
          activeCopyIndex = nextCopyIndex;
        });

        tl.to({}, { duration: 0.65 });

        return undefined;
      });

      return () => mediaQueries.revert();
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

      const moonStory = element.querySelector<HTMLElement>(
        "[data-moon-story]",
      );
      if (!moonStory) return;

      const mediaQueries = gsap.matchMedia();

      mediaQueries.add(motionMediaQueries.parallax, () => {
        const eyebrowLine = moonStory.querySelector<HTMLElement>(
          "[data-moon-eyebrow-line]",
        );
        const label = moonStory.querySelector<HTMLElement>(
          "[data-moon-label]",
        );
        const headline = moonStory.querySelector<HTMLElement>(
          "[data-moon-headline]",
        );
        const body = moonStory.querySelector<HTMLElement>(
          "[data-moon-body]",
        );
        const badge = moonStory.querySelector<HTMLElement>(
          "[data-moon-badge]",
        );
        const moonSphere = moonStory.querySelector<HTMLElement>(
          "[data-moon-sphere]",
        );
        const moonTrajectory = moonStory.querySelector<SVGPathElement>(
          "[data-moon-trajectory]",
        );
        const moonGlow = moonStory.querySelector<HTMLElement>(
          "[data-moon-glow]",
        );
        const starfield = moonStory.querySelector<HTMLElement>(
          "[data-moon-starfield]",
        );

        if (eyebrowLine) gsap.set(eyebrowLine, { scaleX: 0, transformOrigin: "left center" });
        if (label) gsap.set(label, { opacity: 0, y: 8 });
        if (headline) gsap.set(headline, { opacity: 0, y: 24 });
        if (body) gsap.set(body, { opacity: 0, y: 14 });
        if (badge) gsap.set(badge, { opacity: 0, y: 12, scale: 0.95 });
        if (moonSphere) gsap.set(moonSphere, { scale: 0.93 });
        if (moonTrajectory) gsap.set(moonTrajectory, { opacity: 0 });

        const moonTl = gsap.timeline({
          defaults: { ease: "power2.out" },
          scrollTrigger: {
            end: "bottom 40%",
            invalidateOnRefresh: true,
            scrub: 0.6,
            start: "top 80%",
            trigger: moonStory,
          },
        });

        if (eyebrowLine) moonTl.to(eyebrowLine, { duration: 0.25, scaleX: 1 }, 0);
        if (label) moonTl.to(label, { duration: 0.22, opacity: 1, y: 0 }, 0.05);
        if (headline) moonTl.to(headline, { duration: 0.45, opacity: 1, y: 0 }, 0.1);
        if (body) moonTl.to(body, { duration: 0.35, opacity: 1, y: 0 }, 0.2);
        if (badge) moonTl.to(badge, { duration: 0.35, opacity: 1, y: 0, scale: 1 }, 0.28);

        if (moonSphere) {
          moonTl.to(
            moonSphere,
            { duration: 0.8, scale: 1 },
            0.08,
          );
        }
        if (moonGlow) {
          moonTl.fromTo(
            moonGlow,
            { opacity: 0.3, scale: 0.88 },
            { duration: 0.8, opacity: 1, scale: 1 },
            0.12,
          );
        }
        if (moonTrajectory) {
          moonTl.to(
            moonTrajectory,
            { duration: 0.6, opacity: 1 },
            0.18,
          );
        }
        if (starfield) {
          moonTl.fromTo(
            starfield,
            { y: 15 },
            { duration: 1.0, y: -15 },
            0,
          );
        }

        return undefined;
      });

      return () => mediaQueries.revert();
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
