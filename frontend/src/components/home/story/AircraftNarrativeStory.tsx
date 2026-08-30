import Image from "next/image";

import { AircraftNarrativeControls } from "@/components/home/story/AircraftNarrativeControls";
import { Container } from "@/components/layout/Container";

const flythroughAircraft =
  "/images/hero/x-fly-aircraft-flythrough-v1.png";

const narrativeBeats = [
  {
    copy: "One connected journey across continents and time zones.",
    headline: "GO FURTHER.",
    id: "global",
    label: "GLOBAL",
    position:
      "motion-safe:[@media(min-height:52rem)]:lg:col-start-1 motion-safe:[@media(min-height:52rem)]:lg:row-start-1 motion-safe:[@media(min-height:52rem)]:lg:w-[82%] motion-safe:[@media(min-height:52rem)]:lg:max-w-[23rem] motion-safe:[@media(min-height:52rem)]:lg:self-start motion-safe:[@media(min-height:52rem)]:lg:justify-self-start",
  },
  {
    copy: "Space shaped around how you choose to travel.",
    headline: "FLY YOUR WAY.",
    id: "personal",
    label: "PERSONAL",
    position:
      "motion-safe:[@media(min-height:52rem)]:lg:col-start-1 motion-safe:[@media(min-height:52rem)]:lg:row-start-2 motion-safe:[@media(min-height:52rem)]:lg:w-[82%] motion-safe:[@media(min-height:52rem)]:lg:max-w-[24rem] motion-safe:[@media(min-height:52rem)]:lg:self-end motion-safe:[@media(min-height:52rem)]:lg:justify-self-center",
  },
  {
    copy: "Thoughtful comfort, considered at every altitude.",
    headline: "ABOVE THE ORDINARY.",
    id: "premium",
    label: "PREMIUM",
    position:
      "motion-safe:[@media(min-height:52rem)]:lg:col-start-2 motion-safe:[@media(min-height:52rem)]:lg:row-start-1 motion-safe:[@media(min-height:52rem)]:lg:w-[82%] motion-safe:[@media(min-height:52rem)]:lg:max-w-[24rem] motion-safe:[@media(min-height:52rem)]:lg:self-start motion-safe:[@media(min-height:52rem)]:lg:justify-self-end",
  },
  {
    copy: "Every detail considered from departure to arrival.",
    headline: "FROM HERE TO THERE.",
    id: "seamless",
    label: "SEAMLESS",
    position:
      "motion-safe:[@media(min-height:52rem)]:lg:col-start-2 motion-safe:[@media(min-height:52rem)]:lg:row-start-2 motion-safe:[@media(min-height:52rem)]:lg:w-[82%] motion-safe:[@media(min-height:52rem)]:lg:max-w-[25rem] motion-safe:[@media(min-height:52rem)]:lg:self-end motion-safe:[@media(min-height:52rem)]:lg:justify-self-end",
  },
] as const;

const AircraftNarrativeStory = () => (
  <section
    aria-labelledby="aircraft-story-heading"
    className="relative isolate overflow-clip border-y border-border/70 bg-[linear-gradient(180deg,#07090c_0%,#0a121b_48%,#080a0d_100%)] py-section-lg"
    data-aircraft-story
    id="aircraft-story"
  >
    <Container className="relative z-20">
      <div className="flex items-center gap-4">
        <span className="h-px w-10 bg-brand" />
        <p className="text-label text-brand">The X-Fly way</p>
      </div>
      <h2
        className="mt-5 max-w-[11ch] text-h1 text-balance"
        id="aircraft-story-heading"
      >
        Movement, made personal.
      </h2>
      <p className="mt-6 max-w-xl text-body-lg text-muted-foreground">
        More than a route between two places. A journey with one considered
        rhythm from takeoff to arrival.
      </p>
    </Container>

    <div
      className="relative mt-12 motion-safe:[@media(min-height:52rem)]:lg:min-h-[212svh]"
      data-aircraft-flight-corridor
    >
      <div
        className="relative overflow-hidden motion-safe:[@media(min-height:52rem)]:lg:sticky motion-safe:[@media(min-height:52rem)]:lg:top-0 motion-safe:[@media(min-height:52rem)]:lg:h-svh motion-safe:[@media(min-height:52rem)]:lg:min-h-[52rem]"
        data-aircraft-sticky-canvas
      >
        <div
          aria-hidden="true"
          className="pointer-events-none absolute inset-0"
          data-aircraft-atmosphere
        >
          <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_52%_46%,rgba(61,81,99,0.28),transparent_45%),radial-gradient(ellipse_at_48%_72%,rgba(255,212,0,0.055),transparent_38%)]" />
          <div className="absolute inset-x-0 top-[58%] h-px bg-gradient-to-r from-transparent via-foreground/20 to-transparent" />
          <div className="absolute inset-x-[-12%] bottom-[4%] h-[42%] bg-[radial-gradient(ellipse_at_center,rgba(135,151,164,0.11),transparent_67%)]" />
        </div>

        <svg
          aria-hidden="true"
          className="absolute inset-x-0 top-[24%] z-[5] h-[48%] w-full"
          fill="none"
          preserveAspectRatio="none"
          viewBox="0 0 1440 520"
        >
          <path
            className="stroke-brand/70"
            d="M-80 386C260 314 488 104 796 166C1048 217 1192 334 1520 112"
            data-aircraft-route
            pathLength="1"
            strokeLinecap="round"
            strokeWidth="1.5"
          />
        </svg>

        <div
          aria-hidden="true"
          className="relative z-10 mx-auto aspect-[3/2] w-[88vw] max-w-[42rem] motion-safe:[@media(min-height:52rem)]:lg:absolute motion-safe:[@media(min-height:52rem)]:lg:left-0 motion-safe:[@media(min-height:52rem)]:lg:top-[4%] motion-safe:[@media(min-height:52rem)]:lg:mx-0 motion-safe:[@media(min-height:52rem)]:lg:w-[clamp(40rem,65vw,76rem)] motion-safe:[@media(min-height:52rem)]:lg:max-w-none"
          data-story-aircraft
        >
          <Image
            alt=""
            className="object-contain drop-shadow-[0_36px_38px_rgba(0,0,0,0.42)]"
            fill
            sizes="(min-width: 1536px) 76rem, (min-width: 768px) 65vw, 88vw"
            src={flythroughAircraft}
          />
        </div>

        <Container
          className="relative z-20 pb-4 pt-8 motion-safe:[@media(min-height:52rem)]:lg:h-full motion-safe:[@media(min-height:52rem)]:lg:pb-[clamp(3.5rem,7vh,5.5rem)] motion-safe:[@media(min-height:52rem)]:lg:pt-[calc(var(--header-height)+clamp(3rem,6vh,4.5rem))]"
          data-aircraft-safe-canvas
        >
          <ol
            className="flex overflow-x-auto scrollbar-none gap-4 scroll-smooth [scroll-snap-type:x_mandatory] motion-safe:[@media(min-height:52rem)]:lg:grid motion-safe:[@media(min-height:52rem)]:lg:overflow-visible motion-safe:[@media(min-height:52rem)]:lg:h-full motion-safe:[@media(min-height:52rem)]:lg:grid-cols-2 motion-safe:[@media(min-height:52rem)]:lg:grid-rows-2 motion-safe:[@media(min-height:52rem)]:lg:gap-x-12 motion-safe:[@media(min-height:52rem)]:lg:gap-y-10"
            data-aircraft-milestone-canvas
          >
            {narrativeBeats.map((beat, index) => (
              <li
                className={`flex-none w-[clamp(18rem,84vw,26rem)] [scroll-snap-align:start] rounded-xl border border-border/60 bg-[#0c1017]/90 p-6 backdrop-blur-sm motion-safe:[@media(min-height:52rem)]:lg:flex-auto motion-safe:[@media(min-height:52rem)]:lg:w-auto motion-safe:[@media(min-height:52rem)]:lg:rounded-none motion-safe:[@media(min-height:52rem)]:lg:border-0 motion-safe:[@media(min-height:52rem)]:lg:border-t motion-safe:[@media(min-height:52rem)]:lg:border-border-strong/75 motion-safe:[@media(min-height:52rem)]:lg:bg-transparent motion-safe:[@media(min-height:52rem)]:lg:p-0 motion-safe:[@media(min-height:52rem)]:lg:pt-5 ${beat.position}`}
                data-aircraft-beat
                data-aircraft-zone={beat.id}
                key={beat.id}
              >
                <p className="text-caption text-brand" data-aircraft-beat-meta>
                  {String(index + 1).padStart(2, "0")} / 04 · {beat.label}
                </p>
                <h3
                  className="mt-3 text-[clamp(2rem,3.5vw,4.25rem)] font-semibold leading-[0.9] tracking-[-0.055em]"
                  data-aircraft-beat-headline
                >
                  {beat.headline}
                </h3>
                <p
                  className="mt-4 max-w-sm text-body text-muted-foreground"
                  data-aircraft-beat-copy
                >
                  {beat.copy}
                </p>
              </li>
            ))}
          </ol>

          {/* Mobile dot pagination */}
          <AircraftNarrativeControls />
        </Container>
      </div>
    </div>
  </section>
);

export { AircraftNarrativeStory, flythroughAircraft };
