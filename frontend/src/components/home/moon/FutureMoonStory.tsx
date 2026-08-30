"use client";

import { MoonVisual } from "@/components/home/moon/MoonVisual";
import { Container } from "@/components/layout/Container";
import { useLanguage } from "@/i18n/LanguageProvider";

const FutureMoonStory = () => {
  const { t } = useLanguage();

  return (
  <section
    aria-labelledby="future-moon-heading"
    className="relative isolate overflow-hidden border-t border-border/70 bg-[#030508] py-section-lg md:py-section-lg"
    data-moon-story
    id="future-moon"
  >
    {/* Deep Space Background Ambient Gradient */}
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_85%_50%,rgba(28,42,65,0.38)_0%,rgba(10,14,24,0.7)_45%,#030508_80%),linear-gradient(180deg,#06080d_0%,#030508_40%,#020305_100%)]"
    />

    {/* Starlight Ambient Starfield */}
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 opacity-70"
      data-moon-starfield
    >
      <div className="absolute left-[10%] top-[20%] h-1 w-1 rounded-full bg-white shadow-[0_0_6px_#fff]" />
      <div className="absolute left-[24%] top-[35%] h-0.5 w-0.5 rounded-full bg-white/60" />
      <div className="absolute left-[42%] top-[15%] h-1 w-1 rounded-full bg-white/40" />
      <div className="absolute left-[18%] top-[72%] h-0.5 w-0.5 rounded-full bg-white/70 shadow-[0_0_4px_#fff]" />
      <div className="absolute left-[52%] top-[82%] h-1 w-1 rounded-full bg-brand/60 shadow-[0_0_8px_#ffd400]" />
      <div className="absolute left-[68%] top-[18%] h-0.5 w-0.5 rounded-full bg-white/50" />
      <div className="absolute left-[88%] top-[75%] h-1 w-1 rounded-full bg-white/60" />
      <div className="absolute left-[34%] top-[88%] h-0.5 w-0.5 rounded-full bg-white/40" />
    </div>

    {/* Subtle Earth Atmospheric Horizon Glow (Bottom-Left) */}
    <div
      aria-hidden="true"
      className="pointer-events-none absolute -bottom-32 -left-32 h-80 w-80 rounded-full bg-[radial-gradient(circle,rgba(40,75,120,0.22),transparent_70%)] blur-3xl"
    />

    <Container className="relative z-10 grid items-center gap-12 lg:grid-cols-12 lg:gap-8">
      {/* Left Column: Future Moon Editorial Story */}
      <div className="lg:col-span-6 xl:col-span-5" data-moon-copy>
        <div className="flex items-center gap-3" data-moon-eyebrow>
          <span
            aria-hidden="true"
            className="h-px w-8 origin-left bg-brand"
            data-moon-eyebrow-line
          />
          <p
            className="font-mono text-xs font-bold uppercase tracking-widest text-brand"
            data-moon-label
          >
            {t("home.moon.label")}
          </p>
        </div>

        <h2
          className="mt-5 text-[clamp(2.75rem,7vw,6.25rem)] font-semibold uppercase leading-[0.88] tracking-[-0.065em] text-foreground text-balance"
          data-moon-headline
          id="future-moon-heading"
        >
          {t("home.moon.headingFirst")}
          <br />
          {t("home.moon.headingSecond")}
        </h2>

        <p
          className="mt-6 max-w-md text-body text-muted-foreground sm:text-body-lg"
          data-moon-body
        >
          {t("home.moon.body")}
        </p>

        {/* Coming Next Year Status Badge */}
        <div
          aria-hidden="true"
          className="mt-10 inline-flex items-center gap-3 rounded-full border border-white/15 bg-white/5 px-4 py-2 text-caption text-foreground/90 backdrop-blur-sm sm:mt-12"
          data-moon-badge
        >
          <span className="relative flex h-2 w-2">
            <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-brand opacity-75" />
            <span className="relative inline-flex h-2 w-2 rounded-full bg-brand" />
          </span>
          <span className="font-mono text-xs font-bold tracking-widest text-brand">
            {t("home.moon.badge")}
          </span>
        </div>
      </div>

      {/* Right Column: Centerpiece Moon Visual with Trajectory */}
      <div className="flex justify-center lg:col-span-6 lg:justify-end xl:col-span-7">
        <MoonVisual />
      </div>
    </Container>
  </section>
  );
};

export { FutureMoonStory };
