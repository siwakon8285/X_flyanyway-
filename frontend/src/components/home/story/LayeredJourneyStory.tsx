"use client";

import Image from "next/image";
import { useRef, type CSSProperties } from "react";

import { Container } from "@/components/layout/Container";
import { StoryMobilePagination } from "@/components/home/story/StoryMobilePagination";
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
  const mobileJourneyRef = useRef<HTMLDivElement>(null);

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
        className="relative px-page-gutter lg:hidden"
        data-journey-mobile-shell
      >
        <div
          className="relative overflow-hidden rounded-2xl border border-border/60 bg-[#0d1017]"
          data-journey-mobile-content-frame
          data-story-mobile-frame
        >
          <div
            className="flex snap-x snap-mandatory gap-0 overflow-x-auto overscroll-x-contain scrollbar-none scroll-smooth"
            data-journey-mobile
            data-journey-mobile-track
            data-mobile-flow="horizontal"
            data-reduced-motion-fallback="true"
            ref={mobileJourneyRef}
          >
            {journeyChapters.map((chapter) => (
              <article
                className="relative flex flex-[0_0_100%] snap-start flex-col overflow-hidden"
                data-mobile-chapter={chapter.id}
                data-story-mobile-slide={chapter.id}
                key={chapter.id}
              >
                <div className="flex w-full flex-col">
                  <div className="relative aspect-[4/3] w-full shrink-0 overflow-hidden">
                    <Image
                      alt={t(chapter.images[0].altKey)}
                      className="object-cover"
                      fill
                      loading="lazy"
                      sizes="(max-width: 63.999rem) 90vw, 1px"
                      src={chapter.images[0].src}
                    />
                    <div
                      aria-hidden="true"
                      className="absolute inset-0 bg-gradient-to-t from-[#0d1017] via-transparent to-black/10"
                    />
                    <div className="absolute right-5 top-5 aspect-[4/5] w-[29%] overflow-hidden rounded-lg border border-white/20 bg-[#0a0d14] shadow-xl">
                      <Image
                        alt={t(chapter.images[1].altKey)}
                        className="object-cover"
                        fill
                        loading="lazy"
                        sizes="(max-width: 63.999rem) 28vw, 1px"
                        src={chapter.images[1].src}
                      />
                    </div>
                  </div>
                  <div className="relative z-10 -mt-14 flex flex-1 flex-col justify-end bg-gradient-to-b from-transparent via-[#0d1017] to-[#0d1017] px-6 pb-6 pt-20">
                    <p className="text-caption text-brand">{t(chapter.labelKey)}</p>
                    <h3 className="mt-3 text-h3 font-semibold leading-tight text-foreground text-balance">
                      {t(chapter.headlineKey)}
                    </h3>
                    <p className="mt-4 max-w-sm text-body-sm leading-relaxed text-muted-foreground">
                      {t(chapter.bodyKey)}
                    </p>
                  </div>
                </div>
              </article>
            ))}
          </div>
          <StoryMobilePagination
            ariaLabel={t("home.journey.controlLabel")}
            containerRef={mobileJourneyRef}
            items={journeyChapters.map((chapter) => ({
              id: chapter.id,
              label: t("home.journey.goTo", { label: t(chapter.labelKey) }),
            }))}
            className="relative inset-auto bottom-auto z-auto w-full shrink-0 py-3 lg:hidden"
          />
        </div>
      </div>
    </div>
  </section>
  );
};

export { LayeredJourneyStory };
