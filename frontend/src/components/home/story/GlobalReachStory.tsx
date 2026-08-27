import Image from "next/image";

import { Container } from "@/components/layout/Container";

const globalReachImage = "/images/hero/x-fly-global-reach-bg-v1.jpg";

const GlobalReachStory = () => (
  <section
    aria-labelledby="global-reach-heading"
    className="relative isolate flex min-h-svh items-center overflow-hidden border-t border-border/70 bg-[#07090c] py-section-lg md:min-h-[110svh]"
    data-global-story
    id="global-reach"
  >
    {/* Atmospheric background lighting */}
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_80%_45%,rgba(32,52,74,0.38),transparent_48rem),radial-gradient(circle_at_65%_55%,rgba(255,212,0,0.03),transparent_36rem),linear-gradient(180deg,#07090c_0%,#090d14_50%,#07090c_100%)]"
    />
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-x-0 top-0 h-40 bg-gradient-to-b from-black/70 to-transparent"
    />
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-x-0 bottom-0 h-40 bg-gradient-to-t from-black/70 to-transparent"
    />

    {/* Desktop Ambient Glow behind right visual */}
    <div
      aria-hidden="true"
      className="pointer-events-none absolute right-[10%] top-1/2 hidden h-[32rem] w-[32rem] -translate-y-1/2 rounded-full bg-[radial-gradient(circle,rgba(56,92,130,0.2),transparent_70%)] blur-3xl md:block"
    />

    <Container className="relative grid items-center gap-12 lg:grid-cols-[minmax(0,0.88fr)_minmax(0,1.12fr)] lg:gap-16">
      {/* Left copy column */}
      <div className="relative z-20 w-full max-w-xl" data-global-copy>
        <div className="mb-6 flex items-center gap-4" data-global-eyebrow>
          <span className="h-px w-10 origin-left bg-brand" data-global-line />
          <p className="text-label text-brand" data-global-eyebrow-text>Global reach</p>
        </div>

        <div className="overflow-hidden" data-global-heading-wrapper>
          <h2
            className="text-h1 text-balance tracking-[-0.055em]"
            id="global-reach-heading"
          >
            <span className="block" data-global-heading-line>The world,</span>
            <span className="block" data-global-heading-line>closer.</span>
          </h2>
        </div>

        <p className="mt-6 max-w-lg text-body text-muted-foreground sm:text-body-lg" data-global-body>
          Across continents and time zones, X-Fly brings every horizon into one
          connected journey.
        </p>

        {/* Global Hub stats pill */}
        <div
          aria-hidden="true"
          className="mt-10 inline-flex items-center gap-6 rounded-lg border border-border/60 bg-surface/40 px-5 py-3 text-caption text-muted-foreground backdrop-blur-sm sm:mt-12"
          data-global-hubs-pill
        >
          <div className="flex items-center gap-2">
            <span className="h-1.5 w-1.5 rounded-full bg-brand" />
            <span className="font-mono text-foreground/90">7 GLOBAL HUBS</span>
          </div>
          <div className="h-3 w-px bg-border/60" />
          <div className="flex items-center gap-2">
            <span className="h-1.5 w-1.5 rounded-full bg-brand" />
            <span className="font-mono text-foreground/90">24/7 OPERATIONS</span>
          </div>
        </div>
      </div>

      {/* Right visual column: Cinematic Global Reach Photography with Left-to-Right Dissolve */}
      <div
        className="relative aspect-[16/10] w-full overflow-hidden rounded-xl md:rounded-2xl [mask-image:linear-gradient(to_right,transparent_0%,rgba(0,0,0,0.18)_6%,rgba(0,0,0,0.7)_18%,#000_32%)] [-webkit-mask-image:linear-gradient(to_right,transparent_0%,rgba(0,0,0,0.18)_6%,rgba(0,0,0,0.7)_18%,#000_32%)]"
        data-global-visual
      >
        {/* Photographic Visual Layer */}
        <div
          className="absolute inset-0 brightness-[1.03] contrast-[1.02]"
          data-global-image
        >
          <Image
            alt="Global destination view representing X-Fly's international network"
            className="object-cover object-[52%_48%]"
            fill
            priority={false}
            sizes="(min-width: 1280px) 58vw, (min-width: 768px) 52vw, 100vw"
            src={globalReachImage}
          />
        </div>

        {/* Atmospheric Left-to-Right Dark Blending Overlay — gentle dissolve into stage base */}
        <div
          aria-hidden="true"
          className="pointer-events-none absolute inset-0 z-10 bg-[linear-gradient(to_right,rgba(7,9,12,0.96)_0%,rgba(7,9,12,0.72)_8%,rgba(7,9,12,0.28)_20%,transparent_36%)]"
        />

        {/* Top & Bottom Atmosphere Vignettes */}
        <div
          aria-hidden="true"
          className="pointer-events-none absolute inset-0 z-10 bg-[linear-gradient(to_bottom,rgba(7,9,12,0.35)_0%,transparent_16%,transparent_80%,rgba(7,9,12,0.75)_100%)]"
        />

        {/* Subtle Brand Route Motif Accent */}
        <svg
          aria-hidden="true"
          className="pointer-events-none absolute inset-0 z-10 h-full w-full select-none opacity-85"
          data-global-route
          fill="none"
          viewBox="0 0 700 420"
        >
          <path
            className="stroke-brand/80"
            d="M 90 280 Q 260 110 470 145 T 640 190"
            data-route-path
            strokeDasharray="4 4"
            strokeWidth="1.75"
          />
          <circle cx="90" cy="280" fill="#FFD400" r="3.5" />
          <circle cx="470" cy="145" fill="#FFD400" r="4" />
          <circle cx="640" cy="190" fill="#FFD400" r="3.5" />
        </svg>

        {/* 156 Countries Integrated Editorial Metric Overlay */}
        <div
          className="absolute bottom-4 left-4 z-20 sm:bottom-6 sm:left-6"
          data-global-metric
        >
          <div className="flex items-baseline gap-2.5">
            <p className="text-[clamp(3.5rem,7vw,5.75rem)] font-semibold leading-[0.8] tracking-[-0.075em] text-foreground drop-shadow-[0_4px_12px_rgba(0,0,0,0.8)]">
              156
            </p>
            <span className="font-mono text-xs font-bold uppercase tracking-widest text-brand drop-shadow-sm">
              COUNTRIES
            </span>
          </div>
          <p className="mt-1 font-mono text-[0.6875rem] tracking-wider text-muted-foreground/95 drop-shadow-sm">
            ONE CONNECTED JOURNEY
          </p>
        </div>
      </div>
    </Container>
  </section>
);

export { GlobalReachStory, globalReachImage };
