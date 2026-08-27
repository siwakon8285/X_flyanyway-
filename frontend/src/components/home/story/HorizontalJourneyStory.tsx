import Image from "next/image";

import { HorizontalJourneyControls } from "@/components/home/story/HorizontalJourneyControls";
import { Container } from "@/components/layout/Container";

const discoverStageImage = "/images/hero/x-fly-journey-discover-v1.jpg";
const bookStageImage = "/images/hero/x-fly-journey-book-v1.jpg";
const flyStageImage = "/images/hero/x-fly-journey-fly-v1.jpg";
const arriveStageImage = "/images/hero/x-fly-journey-arrive-v1.jpg";
const beyondStageImage = "/images/hero/x-fly-journey-beyond-v1.jpg";

type JourneyStage = {
  copy: string;
  eyebrow: string;
  headline: string;
  id: "discover" | "book" | "fly" | "arrive" | "beyond";
  meta: string;
  number: string;
};

const journeyStages: JourneyStage[] = [
  {
    copy: "A destination chosen, a route imagined. The horizon opens the moment you decide to go.",
    eyebrow: "Stage 01",
    headline: "Where the journey begins.",
    id: "discover",
    meta: "ORIGIN // POINT 01",
    number: "01",
  },
  {
    copy: "From thought to departure in one considered path. Effortless, transparent, shaped entirely around your time.",
    eyebrow: "Stage 02",
    headline: "One considered path.",
    id: "book",
    meta: "WAYPOINT // DIRECT ROUTE",
    number: "02",
  },
  {
    copy: "Comfort and stillness shaped around the journey ahead. Time slows when every detail is taken care of.",
    eyebrow: "Stage 03",
    headline: "Space at 38,000 feet.",
    id: "fly",
    meta: "ALTITUDE // 38,000 FT",
    number: "03",
  },
  {
    copy: "From altitude to destination in continuous stride. Arrive restored, confident, ready for the world below.",
    eyebrow: "Stage 04",
    headline: "A seamless arrival.",
    id: "arrive",
    meta: "DESCENT // FINAL APPROACH",
    number: "04",
  },
  {
    copy: "Every journey is only the prelude to the next. The distance ahead is always waiting.",
    eyebrow: "Stage 05",
    headline: "The next horizon.",
    id: "beyond",
    meta: "FORWARD // ALWAYS AHEAD",
    number: "05",
  },
];

const HorizontalJourneyStory = () => (
  <section
    aria-labelledby="journey-path-heading"
    className="relative isolate overflow-hidden border-t border-border/70 bg-[#07080b] py-section-md md:py-0"
    data-journey-path
    id="journey-path"
  >
    {/* Background ambient lighting */}
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_20%_30%,rgba(255,212,0,0.035),transparent_40%),radial-gradient(ellipse_at_80%_70%,rgba(40,60,85,0.12),transparent_45%)]"
    />

    {/* Section header: visible on mobile, accessible sr-only on desktop */}
    <Container className="mb-8 md:sr-only">
      <div className="flex items-center gap-3 md:hidden">
        <span className="h-px w-8 bg-brand" />
        <p className="text-label text-brand">X-FLY JOURNEY</p>
      </div>
      <h2
        className="mt-3 text-h2 uppercase tracking-tight text-foreground text-balance"
        id="journey-path-heading"
      >
        From first thought to final step.
      </h2>
      <p className="mt-2 text-body-sm text-muted-foreground md:hidden">
        A continuous journey through five considered stages.
      </p>
    </Container>

    {/* Pinned viewport wrapper on desktop */}
    <div
      className="relative flex flex-col justify-center"
      data-journey-viewport
    >
      {/* Route Stage Progress Navigation (Desktop HUD: positioned safely below global header) */}
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-x-0 top-[calc(var(--header-height)+1.25rem)] z-30 hidden px-page-gutter md:block"
        data-journey-hud
      >
        <Container className="flex items-center justify-end">
          <div className="flex items-center gap-7">
            {journeyStages.map((stage) => (
              <div
                className="flex items-center gap-2 text-caption text-muted-foreground transition-opacity"
                data-journey-node={stage.id}
                key={stage.id}
              >
                <span className="font-mono text-[0.6875rem] text-brand">
                  {stage.number}
                </span>
                <span className="uppercase tracking-wider text-[0.6875rem]">{stage.id}</span>
              </div>
            ))}
          </div>
        </Container>
      </div>

      {/* Horizontal Track Container (Swipeable on mobile/tablet, Pinned translation on desktop) */}
      <div
        className="relative"
        data-journey-track
      >
        <ol
          className="contents list-none"
          data-journey-stages
        >
          {/* STAGE 01 — DISCOVER */}
          <li
            aria-label="Stage 01: Discover"
            className="relative flex flex-col justify-center rounded-xl border border-border/60 bg-[#0d0f14]/90 p-6 sm:p-8 md:rounded-none md:border-0 md:bg-transparent md:p-0"
            data-journey-stage="discover"
          >
            <div
              className="mx-auto grid w-full max-w-7xl items-center gap-8 lg:grid-cols-[minmax(0,1.05fr)_minmax(0,1.15fr)] lg:gap-14"
              data-journey-content
            >
              <div className="w-full max-w-xl">
                <div className="flex items-center gap-3" data-journey-eyebrow>
                  <span className="h-px w-8 origin-left bg-brand" data-journey-eyebrow-line />
                  <span className="text-caption text-brand" data-journey-eyebrow-text>01 / 05 · DISCOVER</span>
                </div>

                <div className="mt-5 overflow-hidden" data-journey-headline-mask>
                  <h3
                    className="text-[clamp(2.15rem,4.5vw,4.5rem)] font-semibold leading-[0.92] tracking-[-0.055em] text-foreground text-balance"
                    data-journey-headline
                  >
                    Where the journey begins.
                  </h3>
                </div>

                <p
                  className="mt-6 text-body text-muted-foreground sm:text-body-lg"
                  data-journey-body
                >
                  A destination chosen, a route imagined. The horizon opens the moment
                  you decide to go.
                </p>

                {/* Editorial coordinate detail */}
                <div
                  aria-hidden="true"
                  className="mt-10 grid grid-cols-2 gap-6 text-caption text-muted-foreground/80 sm:mt-12"
                  data-journey-meta
                >
                  <div>
                    <p className="font-mono text-foreground/90">156 DESTINATIONS</p>
                    <p className="mt-1 text-[0.6875rem] text-muted-foreground">GLOBAL REACH</p>
                  </div>
                  <div>
                    <p className="font-mono text-foreground/90">NON-STOP // ANYWHERE</p>
                    <p className="mt-1 text-[0.6875rem] text-muted-foreground">DIRECT PRECISION</p>
                  </div>
                </div>
              </div>

              {/* Stage 01 Visual: Departure Terminal Architecture */}
              <div
                className="relative aspect-[16/10] w-full overflow-hidden rounded-xl border border-border/50 bg-[#090d14] shadow-2xl"
                data-journey-discover-frame
                data-journey-visual="discover"
              >
                <div
                  className="absolute inset-0"
                  data-journey-discover-image
                >
                  <Image
                    alt="Expansive modern airport departure terminal overlooking the airfield at blue hour"
                    className="object-cover object-[55%_50%]"
                    fill
                    priority={false}
                    sizes="(min-width: 1536px) 52rem, (min-width: 768px) 50vw, 92vw"
                    src={discoverStageImage}
                  />
                </div>
                <div className="pointer-events-none absolute inset-0 bg-gradient-to-t from-black/50 via-transparent to-black/20" />
                <span
                  aria-hidden="true"
                  className="absolute bottom-3 left-3 font-mono text-[0.6875rem] text-white/80 sm:bottom-4 sm:left-4"
                >
                  GLOBAL EXPLORATION // 156 DESTINATIONS
                </span>
              </div>
            </div>

            {/* Mobile swipe hint on first card only */}
            <div
              aria-hidden="true"
              className="mt-8 flex items-center justify-end pt-3 text-caption text-brand md:hidden"
            >
              <span className="font-mono text-[0.6875rem] tracking-wider opacity-85">
                SWIPE TO EXPLORE →
              </span>
            </div>
          </li>

          {/* STAGE 02 — BOOK */}
          <li
            aria-label="Stage 02: Book"
            className="relative flex flex-col justify-center rounded-xl border border-border/60 bg-[#0d0f14]/90 p-6 sm:p-8 md:rounded-none md:border-0 md:bg-transparent md:p-0"
            data-journey-stage="book"
          >
            <div
              className="mx-auto grid w-full max-w-7xl items-center gap-8 lg:grid-cols-[minmax(0,1.05fr)_minmax(0,1.15fr)] lg:gap-14"
              data-journey-content
            >
              <div className="w-full max-w-lg">
                <div className="flex items-center gap-3" data-journey-eyebrow>
                  <span className="h-px w-8 origin-left bg-brand" data-journey-eyebrow-line />
                  <span className="text-caption text-brand" data-journey-eyebrow-text>02 / 05 · BOOK</span>
                </div>

                <div className="mt-5 overflow-hidden" data-journey-headline-mask>
                  <h3
                    className="text-[clamp(2.15rem,4.5vw,4.5rem)] font-semibold leading-[0.92] tracking-[-0.055em] text-foreground text-balance"
                    data-journey-headline
                  >
                    One considered path.
                  </h3>
                </div>

                <p
                  className="mt-6 text-body text-muted-foreground sm:text-body-lg"
                  data-journey-body
                >
                  From thought to departure in one considered path. Effortless, transparent,
                  shaped entirely around your time.
                </p>

                {/* Minimalist supporting data */}
                <div
                  aria-hidden="true"
                  className="mt-10 grid grid-cols-2 gap-6 text-caption text-muted-foreground/80 sm:mt-12"
                  data-journey-meta
                >
                  <div>
                    <p className="font-mono text-foreground/90">ZERO FRICTION</p>
                    <p className="mt-1 text-[0.6875rem] text-muted-foreground">CONSIDERED ROUTING</p>
                  </div>
                  <div>
                    <p className="font-mono text-foreground/90">GUARANTEED ACCESS</p>
                    <p className="mt-1 text-[0.6875rem] text-muted-foreground">PRECISION SCHEDULE</p>
                  </div>
                </div>
              </div>

              {/* Stage 02 Visual: Private Lounge Departure Planning */}
              <div
                className="relative aspect-[16/10] w-full overflow-hidden rounded-xl border border-border/50 bg-[#0a0e16] shadow-2xl"
                data-journey-book-frame
                data-journey-visual="book"
              >
                <div
                  className="absolute inset-0"
                  data-journey-book-image
                >
                  <Image
                    alt="Private luxury airport lounge table setting with travel documents overlooking the tarmac"
                    className="object-cover object-[50%_50%]"
                    fill
                    priority={false}
                    sizes="(min-width: 1536px) 52rem, (min-width: 768px) 50vw, 92vw"
                    src={bookStageImage}
                  />
                </div>
                <div className="pointer-events-none absolute inset-0 bg-gradient-to-t from-black/50 via-transparent to-black/20" />
                <span
                  aria-hidden="true"
                  className="absolute bottom-3 left-3 font-mono text-[0.6875rem] text-white/80 sm:bottom-4 sm:left-4"
                >
                  CONSIDERED ROUTING // DIRECT ACCESS
                </span>
              </div>
            </div>
          </li>

          {/* STAGE 03 — FLY (Hero stage) */}
          <li
            aria-label="Stage 03: Fly"
            className="relative flex flex-col justify-center rounded-xl border border-border/60 bg-[#0d0f14]/90 p-6 sm:p-8 md:rounded-none md:border-0 md:bg-transparent md:p-0"
            data-journey-stage="fly"
          >
            <div
              className="mx-auto grid w-full max-w-7xl items-center gap-8 lg:grid-cols-[minmax(0,1.05fr)_minmax(0,1.15fr)] lg:gap-14"
              data-journey-content
            >
              <div>
                <div className="flex items-center gap-3" data-journey-eyebrow>
                  <span className="h-px w-8 origin-left bg-brand" data-journey-eyebrow-line />
                  <span className="text-caption text-brand" data-journey-eyebrow-text>03 / 05 · FLY</span>
                </div>

                <div className="mt-5 overflow-hidden" data-journey-headline-mask>
                  <h3
                    className="text-[clamp(2.15rem,4.8vw,4.85rem)] font-semibold leading-[0.9] tracking-[-0.06em] text-foreground text-balance"
                    data-journey-headline
                  >
                    Space at 38,000 feet.
                  </h3>
                </div>

                <p
                  className="mt-6 max-w-md text-body text-muted-foreground sm:text-body-lg"
                  data-journey-body
                >
                  Comfort and stillness shaped around the journey ahead. Time slows when
                  every detail is taken care of.
                </p>

                {/* Supporting labels */}
                <div
                  aria-hidden="true"
                  className="mt-8 flex items-center gap-6 text-caption sm:mt-10"
                  data-journey-meta
                >
                  <div className="flex items-center gap-2">
                    <span className="h-1.5 w-1.5 rounded-full bg-brand" />
                    <span className="text-muted-foreground">CABIN ATMOSPHERE</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="h-1.5 w-1.5 rounded-full bg-brand" />
                    <span className="text-muted-foreground">INTENTIONAL SERVICE</span>
                  </div>
                </div>
              </div>

              {/* Large Window Panel Visual */}
              <div
                className="relative aspect-[16/10] w-full overflow-hidden rounded-xl border border-border/50 bg-[#12141a] shadow-2xl"
                data-journey-fly-frame
                data-journey-visual="fly"
              >
                <div
                  className="absolute inset-0"
                  data-journey-fly-image
                >
                  <Image
                    alt="View from X-Fly aircraft window looking over golden sunset clouds"
                    className="object-cover object-center"
                    fill
                    priority={false}
                    sizes="(min-width: 1536px) 52rem, (min-width: 768px) 50vw, 92vw"
                    src={flyStageImage}
                  />
                </div>
                <div className="pointer-events-none absolute inset-0 bg-gradient-to-t from-black/50 via-transparent to-black/20" />
                <span
                  aria-hidden="true"
                  className="absolute bottom-3 left-3 font-mono text-[0.6875rem] text-white/80 sm:bottom-4 sm:left-4"
                >
                  CRUISING ALTITUDE // 38,000 FT
                </span>
              </div>
            </div>
          </li>

          {/* STAGE 04 — ARRIVE */}
          <li
            aria-label="Stage 04: Arrive"
            className="relative flex flex-col justify-center rounded-xl border border-border/60 bg-[#0d0f14]/90 p-6 sm:p-8 md:rounded-none md:border-0 md:bg-transparent md:p-0"
            data-journey-stage="arrive"
          >
            <div
              className="mx-auto grid w-full max-w-7xl items-center gap-8 lg:grid-cols-[minmax(0,1.05fr)_minmax(0,1.15fr)] lg:gap-14"
              data-journey-content
            >
              <div>
                <div className="flex items-center gap-3" data-journey-eyebrow>
                  <span className="h-px w-8 origin-left bg-brand" data-journey-eyebrow-line />
                  <span className="text-caption text-brand" data-journey-eyebrow-text>04 / 05 · ARRIVE</span>
                </div>

                <div className="mt-5 overflow-hidden" data-journey-headline-mask>
                  <h3
                    className="text-[clamp(2.15rem,4.5vw,4.5rem)] font-semibold leading-[0.92] tracking-[-0.055em] text-foreground text-balance"
                    data-journey-headline
                  >
                    A seamless arrival.
                  </h3>
                </div>

                <p
                  className="mt-6 text-body text-muted-foreground sm:text-body-lg"
                  data-journey-body
                >
                  From altitude to destination in continuous stride. Arrive restored,
                  confident, ready for the world below.
                </p>

                {/* Supporting touchdown callout */}
                <div
                  aria-hidden="true"
                  className="mt-8 border-l-2 border-brand/80 pl-4 text-caption text-muted-foreground sm:mt-10"
                  data-journey-meta
                >
                  <p className="font-mono text-foreground/90">TOUCHDOWN PRECISION</p>
                  <p className="mt-1 text-[0.75rem]">The ground welcomes you without delay.</p>
                </div>
              </div>

              {/* City Arrival Image Panel */}
              <div
                className="relative aspect-[16/10] w-full overflow-hidden rounded-xl border border-border/50 bg-[#101217] shadow-2xl"
                data-journey-arrive-frame
                data-journey-visual="arrive"
              >
                <div
                  className="absolute inset-0"
                  data-journey-arrive-image
                >
                  <Image
                    alt="Aerial view of illuminated modern city skyline during airport descent"
                    className="object-cover object-center"
                    fill
                    priority={false}
                    sizes="(min-width: 1536px) 52rem, (min-width: 768px) 50vw, 92vw"
                    src={arriveStageImage}
                  />
                </div>
                <div className="pointer-events-none absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-black/20" />
                <span
                  aria-hidden="true"
                  className="absolute bottom-3 left-3 font-mono text-[0.6875rem] text-white/80 sm:bottom-4 sm:left-4"
                >
                  APPROACH // DESTINATION LIGHTS
                </span>
              </div>
            </div>
          </li>

          {/* STAGE 05 — BEYOND */}
          <li
            aria-label="Stage 05: Beyond"
            className="relative flex flex-col justify-center rounded-xl border border-border/60 bg-[#0d0f14]/90 p-6 sm:p-8 md:rounded-none md:border-0 md:bg-transparent md:p-0"
            data-journey-stage="beyond"
          >
            <div
              className="mx-auto grid w-full max-w-7xl items-center gap-8 lg:grid-cols-[minmax(0,1.05fr)_minmax(0,1.15fr)] lg:gap-14"
              data-journey-content
            >
              <div className="w-full max-w-xl">
                <div className="flex items-center gap-3" data-journey-eyebrow>
                  <span className="h-px w-8 origin-left bg-brand" data-journey-eyebrow-line />
                  <span className="text-caption text-brand" data-journey-eyebrow-text>05 / 05 · BEYOND</span>
                </div>

                <div className="mt-5 overflow-hidden" data-journey-headline-mask>
                  <h3
                    className="text-[clamp(2.4rem,5.2vw,5.25rem)] font-semibold leading-[0.88] tracking-[-0.065em] text-foreground text-balance"
                    data-journey-headline
                  >
                    The next horizon.
                  </h3>
                </div>

                <p
                  className="mt-6 text-body text-muted-foreground sm:text-body-lg"
                  data-journey-body
                >
                  Every journey is only the prelude to the next. The distance ahead
                  is always waiting.
                </p>

                <div
                  aria-hidden="true"
                  className="mt-10 flex items-center gap-4 text-caption text-brand sm:mt-12"
                  data-journey-meta
                >
                  <span className="h-px w-12 bg-brand" />
                  <span className="font-mono uppercase tracking-wider">ALWAYS MOVING FORWARD</span>
                </div>
              </div>

              {/* Stage 05 Visual Spread: Forward Runway Twilight Horizon Photographic Panel */}
              <div
                className="relative aspect-[16/10] w-full overflow-hidden rounded-xl border border-border/50 bg-[#0b0e17] shadow-2xl"
                data-journey-beyond-frame
                data-journey-visual="beyond"
              >
                <div
                  className="absolute inset-0"
                  data-journey-beyond-image
                >
                  <Image
                    alt="Illuminated runway centerline lights extending toward a deep starlit twilight horizon"
                    className="object-cover object-center"
                    fill
                    priority={false}
                    sizes="(min-width: 1536px) 52rem, (min-width: 768px) 50vw, 92vw"
                    src={beyondStageImage}
                  />
                </div>
                <div className="pointer-events-none absolute inset-0 bg-gradient-to-t from-black/50 via-transparent to-black/20" />
                <span
                  aria-hidden="true"
                  className="absolute bottom-3 left-3 font-mono text-[0.6875rem] text-white/80 sm:bottom-4 sm:left-4"
                >
                  THE NEXT HORIZON // CONTINUOUS FORWARD
                </span>
              </div>
            </div>
          </li>
        </ol>
      </div>

      {/* Mobile Stage Dot Pagination */}
      <HorizontalJourneyControls />
    </div>
  </section>
);

export { HorizontalJourneyStory, arriveStageImage, beyondStageImage, bookStageImage, discoverStageImage, flyStageImage, journeyStages };
