"use client";

import Image from "next/image";
import { ArrowUpRight, ChevronLeft, ChevronRight } from "lucide-react";
import { useRef, useState } from "react";

import { Container } from "@/components/layout/Container";
import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";
import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { cn } from "@/lib/utils/cn";
import { useLanguage } from "@/i18n/LanguageProvider";

const promotionSlides = [
  {
    id: "premium",
    image: "/images/cabins/x-fly-cabin-business-v1.png",
    imageClassName: "object-cover object-center",
  },
  {
    id: "earlyBooking",
    image: "/images/hero/x-fly-journey-book-v1.jpg",
    imageClassName: "object-cover object-center",
  },
  {
    id: "family",
    image: "/images/hero/x-fly-journey-arrive-v1.jpg",
    imageClassName: "object-cover object-center",
  },
] as const;

type PromotionSlideId = (typeof promotionSlides)[number]["id"];

const HomePromoCarousel = () => {
  const section = useRef<HTMLElement>(null);
  const { t } = useLanguage();
  const reducedMotion = useReducedMotion();
  const [activeIndex, setActiveIndex] = useState(0);
  const activeSlide = promotionSlides[activeIndex];
  const activeId = activeSlide.id as PromotionSlideId;

  const moveTo = (nextIndex: number) => {
    setActiveIndex((nextIndex + promotionSlides.length) % promotionSlides.length);
  };

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
        <div className="overflow-hidden rounded-surface border border-white/15 bg-surface shadow-[0_24px_70px_rgb(0_0_0/0.36)]">
          <article
            aria-label={t("home.promo.slideAria", {
              current: String(activeIndex + 1),
              total: String(promotionSlides.length),
            })}
            aria-roledescription="slide"
            className="group/signature grid min-h-[30rem] lg:min-h-[34rem] lg:grid-cols-[minmax(0,0.95fr)_minmax(22rem,0.75fr)]"
            data-promo-slide={activeSlide.id}
          >
            <div className="relative min-h-[18rem] overflow-hidden lg:min-h-0 lg:order-2">
              <Image
                alt=""
                className={activeSlide.imageClassName}
                fill
                priority={activeIndex === 0}
                sizes="(min-width: 1024px) 52vw, 100vw"
                src={activeSlide.image}
              />
              <div className="absolute inset-0 bg-gradient-to-t from-black/55 via-transparent to-transparent lg:bg-gradient-to-l lg:from-transparent lg:via-black/10 lg:to-black/35" />
            </div>

            <div className="relative flex flex-col justify-between p-6 sm:p-9 lg:p-12">
              <div className="max-w-xl">
                <p
                  className={cn(
                    "text-label text-brand",
                    activeId === "premium" &&
                      "relative inline-flex origin-left overflow-hidden rounded-sm transition-[color,background-color] duration-300 after:pointer-events-none after:absolute after:inset-y-0 after:-left-1/2 after:w-1/4 after:-translate-x-full after:skew-x-[-18deg] after:bg-gradient-to-r after:from-transparent after:via-white/55 after:to-transparent after:opacity-0 motion-safe:transition-transform motion-safe:hover:scale-[1.025] motion-safe:hover:bg-brand/10 motion-safe:hover:after:translate-x-[650%] motion-safe:hover:after:opacity-100 motion-safe:group-focus-within/signature:scale-[1.025] motion-safe:group-focus-within/signature:bg-brand/10 motion-safe:group-focus-within/signature:after:translate-x-[650%] motion-safe:group-focus-within/signature:after:opacity-100 motion-reduce:transform-none motion-reduce:after:hidden",
                  )}
                  data-x-fly-signature={activeId === "premium" ? "" : undefined}
                >
                  {t(`home.promo.slides.${activeId}.label`)}
                </p>
                <p className="mt-7 text-caption text-muted-foreground">
                  {t(`home.promo.slides.${activeId}.meta`)}
                </p>
                <h3 className="mt-3 max-w-[11ch] text-h2 text-balance">
                  {t(`home.promo.slides.${activeId}.heading`)}
                </h3>
                <p className="mt-5 max-w-md text-body-lg text-muted-foreground">
                  {t(`home.promo.slides.${activeId}.body`)}
                </p>
                <a
                  className="mt-7 inline-flex items-center gap-2 rounded-control border border-brand/60 px-4 py-2.5 text-sm font-medium text-foreground transition-colors hover:border-brand hover:bg-brand hover:text-brand-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus"
                  href="#flight-search"
                >
                  {t(`home.promo.slides.${activeId}.action`)}
                  <ArrowUpRight aria-hidden="true" className="size-4" />
                </a>
              </div>

              <div className="mt-10 flex items-center justify-between gap-4">
                <div aria-label={t("home.promo.paginationLabel")} className="flex items-center gap-2" role="group">
                  {promotionSlides.map((slide, index) => (
                    <button
                      aria-current={index === activeIndex ? "true" : undefined}
                      aria-label={t("home.promo.goToSlide", { number: String(index + 1) })}
                      className="group inline-flex min-h-10 min-w-10 items-center justify-center rounded-full outline-none focus-visible:ring-2 focus-visible:ring-focus"
                      key={slide.id}
                      onClick={() => setActiveIndex(index)}
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
                <div className="flex gap-2">
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
          </article>
        </div>
      </Container>
    </section>
  );
};

export { HomePromoCarousel };
