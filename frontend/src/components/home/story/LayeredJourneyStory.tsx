"use client";

import Image from "next/image";
import type { CSSProperties } from "react";

import { Container } from "@/components/layout/Container";
import {
  journeyChapters,
  journeyDepthRoles,
  journeyMotionTiming,
  journeyPromotionBeats,
  journeyStageCards,
  journeyStageSlots,
} from "@/components/home/story/journeyStoryModel";
import { useLanguage } from "@/i18n/LanguageProvider";

type JourneyStageProps = {
  side: keyof typeof journeyStageCards;
};

const slotStyle = (
  side: JourneyStageProps["side"],
  role: (typeof journeyDepthRoles)[number],
): CSSProperties => {
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

const JourneyStage = ({ side }: JourneyStageProps) => {
  const { t } = useLanguage();

  return (
  <div
    className={
      side === "left"
        ? "relative h-[clamp(36rem,76svh,44rem)] w-full max-w-[25rem] justify-self-start overflow-hidden"
        : "relative h-[clamp(36rem,76svh,44rem)] w-full max-w-[31rem] justify-self-end overflow-hidden"
    }
    data-journey-stage={side}
    data-slot-model="immutable"
    data-stage-emphasis={side === "right" ? "primary" : "secondary"}
    data-stage-bounds="contained"
  >
    <div aria-hidden="true" className="pointer-events-none absolute inset-0">
      {journeyDepthRoles.map((role) => (
        <span
          className="absolute w-px"
          data-anchor-bounds="contained"
          data-composition-zone={role === "queued" ? "center-biased" : undefined}
          data-depth-rank={
            role === "queued"
              ? side === "left"
                ? "queued-near"
                : "queued-far"
              : role === "deep"
                ? side === "left"
                  ? "deep-near"
                  : "deep-far"
                : undefined
          }
          data-journey-anchor={role}
          data-slot-geometry="absolute"
          data-slot-id={`${side}-${role}`}
          data-slot-owner={`${side}-stage`}
          data-slot-direction={
            role === "queued" || role === "deep" ? "inward" : undefined
          }
          data-visual-layer={
            role === "front"
              ? side === "right"
                ? "foremost-front"
                : "supporting-front"
              : undefined
          }
          key={`${side}-${role}-anchor`}
          style={slotStyle(side, role)}
        />
      ))}
    </div>

    {journeyStageCards[side].map((image, index) => {
      const role = journeyDepthRoles[index];
      const frameClass =
        side === "left"
          ? "aspect-[3/4]"
          : "aspect-[4/5]";

      return (
        <figure
          className={`absolute w-auto overflow-hidden rounded-xl bg-[#0d0f14] shadow-2xl ${frameClass}`}
          data-depth-role={role}
          data-journey-card
          data-journey-card-id={`${side}-${index}`}
          data-layout-owner="stage-slot"
          data-persistent-card="true"
          data-stack-index={index}
          data-transform-accumulation="none"
          key={`${side}-${image.src}`}
          style={slotStyle(side, role)}
        >
          <Image
            alt={t(image.altKey)}
            className="object-cover"
            fill
            loading="lazy"
            sizes={
              side === "left"
                ? "(min-width: 1280px) 19rem, 24vw"
                : "(min-width: 1280px) 21rem, 26vw"
            }
            src={image.src}
          />
          <span
            aria-hidden="true"
            className="absolute inset-0 bg-gradient-to-t from-black/30 via-transparent to-black/10"
          />
        </figure>
      );
    })}
  </div>
  );
};

const LayeredJourneyStory = () => {
  const { t } = useLanguage();

  return (
  <section
    aria-labelledby="layered-journey-heading"
    className="relative isolate overflow-hidden bg-[#07080b] py-section-md md:py-0"
    data-layered-story
    id="journey-path"
  >
    <div
      className="relative flex w-full flex-col justify-center md:min-h-svh"
      data-layered-viewport
    >
      <Container
        className="mb-12 text-center lg:absolute lg:inset-x-0 lg:top-12 lg:z-40 lg:mb-0"
        data-journey-heading
      >
        <div className="flex items-center justify-center gap-3">
          <span aria-hidden="true" className="h-px w-8 bg-brand" />
          <p className="text-label uppercase tracking-widest text-brand">
            {t("home.journey.label")}
          </p>
          <span aria-hidden="true" className="h-px w-8 bg-brand" />
        </div>
        <h2
          className="mt-3 text-h3 uppercase tracking-tight text-foreground text-balance"
          id="layered-journey-heading"
        >
          {t("home.journey.heading")}
        </h2>
      </Container>

      <div
        className="relative mx-auto hidden min-h-svh w-full max-w-[90rem] grid-cols-[minmax(0,0.82fr)_minmax(12rem,0.42fr)_minmax(0,1fr)] items-center gap-12 px-page-gutter pb-16 pt-28 lg:grid lg:gap-16"
        data-desktop-pin="bounded"
        data-copy-change-offset={journeyMotionTiming.copyChangeOffset}
        data-interpolation-ease={journeyMotionTiming.interpolationEase}
        data-journey-desktop
        data-pair-starts={journeyMotionTiming.pairStarts.join(",")}
        data-promotion-duration={journeyMotionTiming.promotionDuration}
        data-scroll-distance-vh={journeyMotionTiming.scrollDistanceVh}
        data-scrub-smoothing={journeyMotionTiming.scrubSmoothing}
      >
        <JourneyStage side="left" />

        <div className="relative z-40 min-h-52 text-center" data-journey-copy-region>
          {journeyChapters.map((chapter, index) => (
            <div
              className={`absolute inset-0 flex flex-col items-center justify-center ${
                index === 0 ? "opacity-100" : "opacity-0"
              }`}
              data-layered-copy={chapter.id}
              key={chapter.id}
            >
              <p className="mb-3 text-caption text-brand">{t(chapter.labelKey)}</p>
              <h3 className="text-h3 font-semibold text-foreground text-balance">
                {t(chapter.headlineKey)}
              </h3>
              <p className="mt-4 max-w-xs text-body-sm leading-relaxed text-muted-foreground">
                {t(chapter.bodyKey)}
              </p>
            </div>
          ))}
        </div>

        <JourneyStage side="right" />

        <div aria-hidden="true" className="sr-only" data-promotion-model>
          {journeyPromotionBeats.map((beat, index) => (
            <span
              data-chapter-advance={
                "chapterAdvance" in beat ? beat.chapterAdvance : undefined
              }
              data-promotion-beat={index + 1}
              data-promotion-mode="pair"
              data-promotes-stack-index={beat.promotesStackIndex}
              data-left-target-slot="left-front"
              data-left-card-id={`left-${beat.promotesStackIndex}`}
              data-right-target-slot="right-front"
              data-right-card-id={`right-${beat.promotesStackIndex}`}
              data-slot-interpolation="continuous"
              key={`pair-${index}`}
            />
          ))}
        </div>
      </div>

      <div
        className="flex flex-col gap-20 px-page-gutter lg:hidden"
        data-mobile-flow="vertical"
        data-journey-mobile
        data-reduced-motion-fallback="true"
      >
        {journeyChapters.map((chapter) => (
          <article
            className="flex flex-col text-center"
            data-mobile-chapter={chapter.id}
            key={chapter.id}
          >
            <p className="mb-3 text-caption text-brand">{t(chapter.labelKey)}</p>
            <h3 className="text-h3 font-semibold text-foreground">
              {t(chapter.headlineKey)}
            </h3>
            <p className="mx-auto mt-4 max-w-sm text-body-sm leading-relaxed text-muted-foreground">
              {t(chapter.bodyKey)}
            </p>
            <div className="mt-8 grid gap-6">
              {chapter.images.map((image, index) => (
                <figure
                  className={`relative w-full overflow-hidden rounded-xl shadow-xl ${
                    index === 0 ? "aspect-[3/4]" : "aspect-[4/5]"
                  }`}
                  key={image.src}
                >
                  <Image
                    alt={t(image.altKey)}
                    className="object-cover"
                    fill
                    loading="lazy"
                    sizes="(max-width: 47.999rem) 100vw, 1px"
                    src={image.src}
                  />
                </figure>
              ))}
            </div>
          </article>
        ))}
      </div>
    </div>
  </section>
  );
};

export { LayeredJourneyStory };
