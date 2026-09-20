"use client";

import Image from "next/image";
import { ArrowUpRight, ChevronLeft, ChevronRight } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { Container } from "@/components/layout/Container";
import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";
import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { cn } from "@/lib/utils/cn";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

type PromotionSlide = {
  id: "premium" | "earlyBooking" | "family" | "moon";
  image: string;
  imageClassName?: string;
  visual: "image" | "moon";
  ctaKey?: TranslationKey;
  badgeKey?: TranslationKey;
};

const promotionSlides: readonly PromotionSlide[] = [
  {
    id: "premium",
    image: "/images/cabins/x-fly-cabin-business-v1.png",
    imageClassName: "object-cover object-center",
    visual: "image",
    badgeKey: "home.promo.slides.premium.badge",
    ctaKey: "home.promo.slides.premium.action",
  },
  {
    id: "earlyBooking",
    image: "/images/hero/x-fly-journey-book-v1.jpg",
    imageClassName: "object-cover object-center",
    visual: "image",
    ctaKey: "home.promo.slides.earlyBooking.action",
  },
  {
    id: "family",
    image: "/images/hero/x-fly-journey-arrive-v1.jpg",
    imageClassName: "object-cover object-center",
    visual: "image",
    ctaKey: "home.promo.slides.family.action",
  },
  {
    id: "moon",
    image: "/images/campaigns/x-fly-moon-v1.jpg",
    imageClassName: "object-cover object-center",
    visual: "moon",
    badgeKey: "home.promo.slides.moon.badge",
  },
];

const HomePromoCarousel = () => {
  const section = useRef<HTMLElement>(null);
  const promotionTrack = useRef<HTMLDivElement>(null);
  const { t } = useLanguage();
  const reducedMotion = useReducedMotion();
  const [activeIndex, setActiveIndex] = useState(0);
  const activeSlide = promotionSlides[activeIndex];

  const scrollToSlide = (index: number) => {
    const track = promotionTrack.current;
    const slide = track?.querySelectorAll<HTMLElement>("[data-promo-slide]")[index];
    if (!track || !slide) {
      setActiveIndex(index);
      return;
    }

    setActiveIndex(index);
    if (typeof track.scrollTo !== "function") return;

    track.scrollTo({
      behavior: reducedMotion ? "auto" : "smooth",
      left: slide.offsetLeft,
    });
  };

  const moveTo = (nextIndex: number) => {
    const normalizedIndex =
      (nextIndex + promotionSlides.length) % promotionSlides.length;
    scrollToSlide(normalizedIndex);
  };

  useEffect(() => {
    const track = promotionTrack.current;
    if (!track || !("IntersectionObserver" in window)) return;

    const slides = Array.from(
      track.querySelectorAll<HTMLElement>("[data-promo-slide]"),
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
      { root: track, threshold: 0.6 },
    );

    slides.forEach((slide) => observer.observe(slide));
    return () => observer.disconnect();
  }, []);

  useGSAP(
    () => {
      const signature = section.current?.querySelector(
        "[data-x-fly-signature]",
      );
      if (!signature || reducedMotion) return;

      gsap.fromTo(
        signature,
        { autoAlpha: 0, y: 6 },
        {
          autoAlpha: 1,
          duration: motionDurations.ui,
          ease: gsapEasings.enter,
          scrollTrigger: {
            once: true,
            start: "top 92%",
            trigger: signature,
          },
          y: 0,
        },
      );
    },
    {
      dependencies: [reducedMotion],
      revertOnUpdate: true,
      scope: section,
    },
  );

  return (
    <section
      aria-labelledby="home-promo-carousel-heading"
      className="border-y border-border bg-[#0a0b0e] py-section-sm"
      id="offers"
      ref={section}
    >
      <Container>
        <h2 className="sr-only" id="home-promo-carousel-heading">
          {t("home.promo.carouselLabel")}
        </h2>
        <div className="relative overflow-hidden rounded-surface border border-white/15 bg-surface shadow-[0_24px_70px_rgb(0_0_0/0.36)]">
          <div
            className="flex snap-x snap-mandatory overflow-x-auto overscroll-x-contain scrollbar-none scroll-smooth motion-reduce:scroll-auto lg:overflow-hidden lg:snap-none"
            data-promo-active-slide={activeSlide.id}
            data-promo-track
            ref={promotionTrack}
          >
            {promotionSlides.map((slide, index) => (
              <article
                aria-label={t("home.promo.slideAria", {
                  current: String(index + 1),
                  total: String(promotionSlides.length),
                })}
                aria-roledescription="slide"
                className="group/signature grid min-w-0 flex-[0_0_100%] snap-start min-h-[30rem] lg:min-h-[34rem] lg:grid-cols-[minmax(0,0.95fr)_minmax(22rem,0.75fr)]"
                data-promo-slide={slide.id}
                key={slide.id}
              >
                <div
                  className="relative min-h-[18rem] overflow-hidden lg:min-h-0 lg:order-2"
                  data-promo-visual={slide.visual}
                >
                  <Image
                    alt=""
                    className={slide.imageClassName}
                    fill
                    priority={index === 0}
                    sizes="(min-width: 1024px) 52vw, 100vw"
                    src={slide.image}
                  />
                  <div className="absolute inset-0 bg-gradient-to-t from-black/55 via-transparent to-transparent lg:bg-gradient-to-l lg:from-transparent lg:via-black/10 lg:to-black/35" />
                </div>

                <div className="relative flex flex-col justify-start p-6 pb-32 sm:p-9 sm:pb-32 lg:p-12 lg:pb-32">
                  <div className="max-w-xl">
                    <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
                      <p
                        className={cn(
                          "text-label text-brand",
                          slide.id === "premium" &&
                            "relative inline-flex origin-left overflow-hidden rounded-sm transition-[color,background-color] duration-300 after:pointer-events-none after:absolute after:inset-y-0 after:-left-1/2 after:w-1/4 after:-translate-x-full after:skew-x-[-18deg] after:bg-gradient-to-r after:from-transparent after:via-white/55 after:to-transparent after:opacity-0 motion-safe:transition-transform motion-safe:hover:scale-[1.025] motion-safe:hover:bg-brand/10 motion-safe:hover:after:translate-x-[650%] motion-safe:hover:after:opacity-100 motion-safe:group-focus-within/signature:scale-[1.025] motion-safe:group-focus-within/signature:bg-brand/10 motion-safe:group-focus-within/signature:after:translate-x-[650%] motion-safe:group-focus-within/signature:after:opacity-100 motion-reduce:transform-none motion-reduce:after:hidden",
                        )}
                        data-x-fly-signature={slide.id === "premium" ? "" : undefined}
                      >
                        {t(`home.promo.slides.${slide.id}.label`)}
                      </p>
                      {slide.badgeKey ? (
                        <span
                          className="inline-flex items-center rounded-full bg-brand px-3 py-1 text-[0.68rem] font-bold uppercase tracking-[0.12em] text-brand-foreground"
                          data-promo-campaign-badge
                        >
                          {t(slide.badgeKey)}
                        </span>
                      ) : null}
                    </div>
                    <p className="mt-7 text-caption text-muted-foreground">
                      {t(`home.promo.slides.${slide.id}.meta`)}
                    </p>
                    <h3 className="mt-3 max-w-[11ch] whitespace-pre-line text-h2 text-balance">
                      {t(`home.promo.slides.${slide.id}.heading`)}
                    </h3>
                    <p className="mt-5 max-w-md whitespace-pre-line text-body-lg text-muted-foreground">
                      {t(`home.promo.slides.${slide.id}.body`)}
                    </p>
                    {slide.ctaKey ? (
                      <a
                        className="mt-7 inline-flex items-center gap-2 rounded-control border border-brand/60 px-4 py-2.5 text-sm font-medium text-foreground transition-colors hover:border-brand hover:bg-brand hover:text-brand-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
                        href="#flight-search"
                      >
                        {t(slide.ctaKey)}
                        <ArrowUpRight aria-hidden="true" className="size-4" />
                      </a>
                    ) : null}
                  </div>
                </div>
              </article>
            ))}
          </div>

          <div
            className="pointer-events-none absolute inset-x-0 bottom-0 z-10 grid bg-gradient-to-t from-surface via-surface/95 to-transparent pt-16 lg:grid-cols-[minmax(0,0.95fr)_minmax(22rem,0.75fr)] lg:bg-transparent lg:pt-0"
            data-promo-controls
          >
            <div className="pointer-events-auto flex items-center justify-between gap-4 px-6 pb-6 sm:px-9 sm:pb-9 lg:px-12 lg:pb-12">
              <div aria-label={t("home.promo.paginationLabel")} className="flex items-center gap-2" role="group">
                {promotionSlides.map((slide, index) => (
                  <button
                    aria-current={index === activeIndex ? "true" : undefined}
                    aria-label={t("home.promo.goToSlide", { number: String(index + 1) })}
                    className="group inline-flex min-h-10 min-w-10 items-center justify-center rounded-full outline-none focus-visible:ring-2 focus-visible:ring-focus"
                    key={slide.id}
                    onClick={() => scrollToSlide(index)}
                    type="button"
                  >
                    <span
                      className={`h-1.5 rounded-full transition-all duration-200 motion-reduce:transition-none ${
                        index === activeIndex
                          ? "w-7 bg-brand"
                          : "w-1.5 bg-white/35 group-hover:bg-white/70"
                      }`}
                    />
                  </button>
                ))}
              </div>
              <div
                className="hidden gap-2 lg:flex"
                data-promo-arrow-controls
              >
                <button
                  aria-label={t("home.promo.previous")}
                  className="inline-flex size-10 items-center justify-center rounded-full border border-white/20 text-foreground transition-colors hover:border-brand hover:text-brand focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
                  onClick={() => moveTo(activeIndex - 1)}
                  type="button"
                >
                  <ChevronLeft aria-hidden="true" className="size-5" />
                </button>
                <button
                  aria-label={t("home.promo.next")}
                  className="inline-flex size-10 items-center justify-center rounded-full border border-white/20 text-foreground transition-colors hover:border-brand hover:text-brand focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
                  onClick={() => moveTo(activeIndex + 1)}
                  type="button"
                >
                  <ChevronRight aria-hidden="true" className="size-5" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </Container>
    </section>
  );
};

export { HomePromoCarousel };
