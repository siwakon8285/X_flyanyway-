import Image from "next/image";

import { Container } from "@/components/layout/Container";

const globalReachImage = "/images/hero/x-fly-global-reach-luxury.jpg";

const GlobalReachStory = () => (
  <section
    aria-labelledby="global-reach-heading"
    className="relative isolate flex min-h-[90svh] items-center overflow-hidden border-t border-border/70 bg-[#07090c] py-section-lg md:min-h-[100svh]"
    data-global-story
    data-travel-network="cartographic"
    id="global-reach"
  >
    <div
      className="absolute inset-0 overflow-hidden brightness-[0.78] contrast-[1.08]"
      data-global-visual
    >
      <div className="absolute -inset-[12%]" data-global-image>
        <Image
          alt="Dark world map connecting international cities with illuminated airline routes"
          className="object-cover object-center"
          fill
          loading="lazy"
          sizes="100vw"
          src={globalReachImage}
        />
      </div>
    </div>

    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 z-10 bg-[linear-gradient(to_right,rgba(7,9,12,0.96)_0%,rgba(7,9,12,0.72)_42%,rgba(7,9,12,0.12)_78%,rgba(7,9,12,0.36)_100%)]"
    />
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 z-10 bg-[linear-gradient(to_bottom,rgba(7,9,12,0.8)_0%,transparent_20%,transparent_80%,rgba(7,9,12,0.8)_100%)]"
    />
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 z-10 bg-[radial-gradient(circle_at_20%_50%,rgba(7,9,12,0.7),transparent_60%)]"
    />

    <Container className="relative z-20">
      <div className="w-full max-w-2xl" data-global-copy>
        <div className="mb-6 flex items-center gap-4" data-global-eyebrow>
          <span className="h-px w-10 origin-left bg-brand" data-global-line />
          <p className="text-label text-brand" data-global-eyebrow-text>Global network</p>
        </div>

        <div className="overflow-hidden" data-global-heading-wrapper>
          <h2
            className="text-[clamp(3.5rem,6vw,5.5rem)] font-semibold leading-[0.9] tracking-[-0.055em] text-foreground text-balance"
            id="global-reach-heading"
          >
            <span className="block" data-global-heading-line>The world,</span>
            <span className="block" data-global-heading-line>within reach.</span>
          </h2>
        </div>

        <p className="mt-6 max-w-lg text-body text-muted-foreground sm:text-body-lg" data-global-body>
          Connected across every horizon. Iconic destinations, global hubs,
          and one considered journey between them.
        </p>

        <div
          className="mt-10 inline-flex items-center gap-6 rounded-lg border border-border/60 bg-surface/20 px-5 py-3 text-caption text-muted-foreground backdrop-blur-md sm:mt-12"
          data-global-hubs-pill
        >
          <div className="flex items-center gap-2">
            <span className="h-1.5 w-1.5 rounded-full bg-brand shadow-[0_0_8px_rgba(255,212,0,0.6)]" />
            <span className="font-mono text-foreground/90">7 GLOBAL HUBS</span>
          </div>
          <div className="h-3 w-px bg-border/60" />
          <div className="flex items-center gap-2">
            <span className="h-1.5 w-1.5 rounded-full bg-brand shadow-[0_0_8px_rgba(255,212,0,0.6)]" />
            <span className="font-mono text-foreground/90">24/7 OPERATIONS</span>
          </div>
        </div>
      </div>
    </Container>

    <div
      className="absolute bottom-8 right-8 z-20 text-right md:bottom-12 md:right-12"
      data-global-metric
    >
      <div className="flex items-baseline justify-end gap-3">
        <span className="font-mono text-xs font-bold uppercase tracking-widest text-brand drop-shadow-sm">
          COUNTRIES
        </span>
        <p className="text-[clamp(4rem,8vw,7rem)] font-semibold leading-[0.8] tracking-[-0.075em] text-foreground drop-shadow-[0_4px_16px_rgba(0,0,0,0.8)]">
          156
        </p>
      </div>
      <p className="mt-2 font-mono text-[0.6875rem] tracking-wider text-muted-foreground/95 drop-shadow-sm">
        ONE CONNECTED JOURNEY
      </p>
    </div>
  </section>
);

export { GlobalReachStory, globalReachImage };
