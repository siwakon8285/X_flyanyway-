"use client";

import Image from "next/image";

import { Container } from "@/components/layout/Container";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

const cabinImages = {
  business: "/images/hero/x-fly-cabin-business-v1.png",
  economy: "/images/hero/x-fly-cabin-economy-v1.png",
  first: "/images/hero/x-fly-cabin-first-v1.png",
  "premium-economy": "/images/hero/x-fly-cabin-premium-economy-v1.png",
} as const;

const cabins = [
  {
    altKey: "home.cabins.economy.alt",
    atmosphere:
      "bg-[radial-gradient(circle_at_78%_48%,rgba(84,105,124,0.13),transparent_43%)]",
    copyKey: "home.cabins.economy.copy",
    id: "economy",
    image: cabinImages.economy,
    labelKey: "common.cabins.economy",
    position: "object-[52%_48%]",
  },
  {
    altKey: "home.cabins.premiumEconomy.alt",
    atmosphere:
      "bg-[radial-gradient(circle_at_76%_46%,rgba(185,151,91,0.12),transparent_45%)]",
    copyKey: "home.cabins.premiumEconomy.copy",
    id: "premium-economy",
    image: cabinImages["premium-economy"],
    labelKey: "common.cabins.premiumEconomy",
    position: "object-[50%_50%]",
  },
  {
    altKey: "home.cabins.business.alt",
    atmosphere:
      "bg-[linear-gradient(116deg,transparent_34%,rgba(72,91,112,0.14)_56%,transparent_78%)]",
    copyKey: "home.cabins.business.copy",
    id: "business",
    image: cabinImages.business,
    labelKey: "common.cabins.business",
    position: "object-[48%_50%]",
  },
  {
    altKey: "home.cabins.first.alt",
    atmosphere:
      "bg-[radial-gradient(ellipse_at_82%_50%,rgba(255,212,0,0.085),rgba(95,73,35,0.055)_35%,transparent_67%)]",
    copyKey: "home.cabins.first.copy",
    id: "first",
    image: cabinImages.first,
    labelKey: "common.cabins.first",
    position: "object-[50%_50%]",
  },
] as const satisfies readonly {
  altKey: TranslationKey;
  atmosphere: string;
  copyKey: TranslationKey;
  id: string;
  image: string;
  labelKey: TranslationKey;
  position: string;
}[];

const CabinStory = () => {
  const { t } = useLanguage();

  return (
  <section
    aria-labelledby="cabins-heading"
    className="relative isolate border-y border-border/70 bg-[#07080b] py-section-md md:py-0"
    data-cabin-story
    id="cabins"
  >
    <div className="relative min-h-svh overflow-hidden" data-cabin-frame>
      {/* Decorative subtle ambient aperture behind photography */}
      <div
        aria-hidden="true"
        className="pointer-events-none absolute -right-[18rem] top-1/2 hidden aspect-[0.68] w-[42rem] -translate-y-1/2 rounded-[50%] border border-border-strong/20 bg-[radial-gradient(ellipse_at_center,rgba(255,212,0,0.03),transparent_64%)] opacity-30 md:block xl:-right-[10rem] xl:w-[50rem]"
        data-cabin-aperture
      />
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-y-0 right-0 hidden w-[58%] bg-[linear-gradient(118deg,transparent_12%,rgba(255,255,255,0.01)_38%,rgba(255,212,0,0.025)_54%,transparent_76%)] opacity-35 md:block"
        data-cabin-light
      />

      {/* Stage specific subtle color atmosphere */}
      <div aria-hidden="true" className="pointer-events-none absolute inset-0">
        {cabins.map((cabin, index) => (
          <div
            className={`absolute inset-0 ${cabin.atmosphere} ${index === 0 ? "opacity-70" : "opacity-0"} motion-reduce:opacity-25`}
            data-cabin-atmosphere
            key={`${cabin.id}-atmosphere`}
          />
        ))}
      </div>

      {/* Desktop Immersive Cabin Visual Layer (Edge-to-Edge Atmospheric Right Background) */}
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-y-0 right-0 z-0 hidden w-[60vw] lg:w-[64vw] xl:w-[68vw] overflow-hidden md:block [mask-image:linear-gradient(to_right,transparent_0%,rgba(0,0,0,0.4)_6%,#000_20%)] [-webkit-mask-image:linear-gradient(to_right,transparent_0%,rgba(0,0,0,0.4)_6%,#000_20%)]"
        data-cabin-media-frame
      >
        <div className="relative h-full w-full brightness-[1.02] contrast-[1.02]">
          {cabins.map((cabin, index) => (
            <div
              className={`absolute inset-0 ${index === 0 ? "opacity-100" : "opacity-0"}`}
              data-cabin-image={cabin.id}
              key={`desktop-media-${cabin.id}`}
            >
              <Image
                alt={t(cabin.altKey)}
                className={`object-cover ${cabin.position}`}
                fill
                loading={index === 0 ? "eager" : "lazy"}
                sizes="(min-width: 1024px) 64vw, 100vw"
                src={cabin.image}
              />
            </div>
          ))}

          {/* Calibrated Left-to-Right Blending Overlay — covers roughly left ~40% for clear seat visibility */}
          <div className="pointer-events-none absolute inset-0 z-10 bg-[linear-gradient(to_right,rgba(7,8,11,0.98)_0%,rgba(7,8,11,0.88)_12%,rgba(7,8,11,0.62)_24%,rgba(7,8,11,0.32)_34%,rgba(7,8,11,0.10)_42%,rgba(7,8,11,0.02)_50%,transparent_56%)]" />

          {/* Top & Bottom Soft Atmosphere Blends */}
          <div className="pointer-events-none absolute inset-0 z-10 bg-[linear-gradient(to_bottom,rgba(7,8,11,0.35)_0%,rgba(7,8,11,0.15)_10%,transparent_20%,transparent_82%,rgba(7,8,11,0.7)_100%)]" />
        </div>
      </div>

      <Container className="relative z-10 flex min-h-svh flex-col justify-between pt-4 md:pt-header-safe">
        {/* Section Header */}
        <div className="grid gap-5 border-b border-border/80 pb-8 md:grid-cols-[auto_1fr] md:items-end md:justify-between">
          <div>
            <p className="text-label text-brand">{t("home.cabins.label")}</p>
            <h2 className="mt-4 max-w-4xl text-h1 text-balance" id="cabins-heading">
              {t("home.cabins.heading")}
            </h2>
          </div>
          <p className="max-w-sm text-body text-muted-foreground md:justify-self-end">
            {t("home.cabins.intro")}
          </p>
        </div>

        {/* Main Cabin Content Stack (Lower-Left Placement on Desktop, Sequential on Mobile) */}
        <div
          className="relative my-auto w-full max-w-xl space-y-12 py-8 md:my-0 md:mb-16 md:space-y-0 md:py-0"
          data-cabin-stage-stack
        >
          {cabins.map((cabin, index) => (
            <article
              className="relative flex flex-col justify-between rounded-xl border border-border/60 bg-[#0d0f14]/90 p-6 sm:p-8 md:w-full md:min-h-0 md:rounded-none md:border-0 md:bg-transparent md:p-0 md:absolute md:inset-x-0 md:bottom-0 md:flex md:flex-col md:justify-end"
              data-cabin-id={cabin.id}
              data-cabin-stage
              key={cabin.id}
            >
              <div>
                <p className="text-caption text-muted-foreground">
                  {String(index + 1).padStart(2, "0")} / 04
                </p>
                <h3 className="mt-4 text-[clamp(2.75rem,7vw,7rem)] font-semibold leading-[0.86] tracking-[-0.07em] text-foreground text-balance">
                  {t(cabin.labelKey)}
                </h3>
                <p className="mt-6 max-w-[26rem] text-body text-muted-foreground sm:text-body-lg">
                  {t(cabin.copyKey)}
                </p>
              </div>

              {/* Mobile-only sequential inline image */}
              <div
                className="mt-6 relative aspect-[16/10] w-full overflow-hidden rounded-xl border border-border/50 bg-[#0a0d14] shadow-xl md:hidden"
                data-cabin-mobile-image
              >
                <Image
                  alt={t(cabin.altKey)}
                  className={`object-cover ${cabin.position}`}
                  fill
                  loading="lazy"
                  sizes="100vw"
                  src={cabin.image}
                />
              </div>
            </article>
          ))}
        </div>

        {/* Progress indicator (Desktop only: embedded over bottom-right visual region) */}
        <div
          aria-hidden="true"
          className="pointer-events-none absolute bottom-10 right-page-gutter z-20 hidden items-center gap-3 text-caption text-muted-foreground md:flex"
          data-cabin-progress
        >
          <span className="font-mono text-[0.6875rem]">01</span>
          <span className="relative h-px w-24 overflow-hidden bg-border-strong">
            <span
              className="absolute inset-y-0 left-0 w-full origin-left bg-brand"
              data-cabin-progress-line
            />
          </span>
          <span className="font-mono text-[0.6875rem]">04</span>
        </div>
      </Container>
    </div>
  </section>
  );
};

export { CabinStory, cabinImages, cabins };
