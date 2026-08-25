import { Container } from "@/components/layout/Container";

const regions = ["Americas", "Europe", "Africa", "Middle East", "Asia Pacific"] as const;

const routeNodes = [
  { id: "north-america", x: 178, y: 145 },
  { id: "south-america", x: 229, y: 267 },
  { id: "europe", x: 472, y: 137 },
  { id: "africa", x: 481, y: 220 },
  { id: "asia", x: 690, y: 145 },
  { id: "australia", x: 748, y: 300 },
] as const;

const GlobalReachStory = () => (
  <section
    aria-labelledby="global-reach-heading"
    className="relative isolate flex min-h-svh items-center overflow-hidden border-t border-border/70 bg-[radial-gradient(circle_at_78%_44%,rgba(27,43,59,0.72),transparent_34rem),linear-gradient(180deg,#090909_0%,#0a1017_58%,#090909_100%)] py-section-lg md:min-h-[135svh]"
    data-global-story
    id="global-reach"
  >
    <div
      aria-hidden="true"
      className="absolute inset-x-0 top-0 h-36 bg-gradient-to-b from-black/45 to-transparent"
    />

    <Container className="relative grid items-center gap-16 md:grid-cols-[minmax(0,0.82fr)_minmax(22rem,1.18fr)] md:gap-8">
      <div className="relative z-10" data-global-copy>
        <div className="mb-8 flex items-center gap-4">
          <span className="h-px w-10 origin-left bg-brand" data-global-line />
          <p className="text-label text-brand">Global reach</p>
        </div>
        <h2 className="max-w-3xl text-h1 text-balance" id="global-reach-heading">
          The world, closer.
        </h2>
        <p className="mt-6 max-w-lg text-body-lg text-muted-foreground">
          Across continents and time zones, X-Fly brings every horizon into one
          connected journey.
        </p>
      </div>

      <div
        className="relative min-h-[29rem] overflow-hidden md:min-h-[42rem]"
        data-global-visual
      >
        <svg
          aria-hidden="true"
          className="absolute inset-x-0 top-1/2 h-auto w-full -translate-y-1/2"
          data-world-map
          fill="none"
          viewBox="0 0 880 460"
        >
          <g className="fill-[#152231] stroke-[#526171]/55" strokeWidth="1">
            <path
              d="M83 132L112 94L171 73L233 80L276 110L263 139L231 148L214 182L177 198L146 174L119 179L100 151Z"
              data-world-region="north-america"
            />
            <path
              d="M220 215L257 225L276 258L264 302L245 326L238 365L219 395L203 362L198 324L181 286L188 247Z"
              data-world-region="south-america"
            />
            <path
              d="M429 128L455 110L493 114L512 132L494 148L462 145L448 160L422 151Z"
              data-world-region="europe"
            />
            <path
              d="M447 168L500 163L536 194L527 244L503 290L479 320L456 284L439 238L416 201Z"
              data-world-region="africa"
            />
            <path
              d="M513 124L563 92L637 80L717 101L773 137L754 167L709 174L675 202L626 190L591 166L544 172L505 147Z"
              data-world-region="asia"
            />
            <path
              d="M697 287L735 269L781 283L798 315L774 337L724 333L695 312Z"
              data-world-region="australia"
            />
            <path d="M323 64L354 41L385 49L373 78L340 88Z" opacity="0.66" />
          </g>

          <g fill="none" strokeLinecap="round">
            <path
              className="stroke-brand/85"
              d="M178 145Q326 38 472 137"
              data-route-path
              pathLength="1"
              strokeWidth="2"
            />
            <path
              className="stroke-border-strong"
              d="M472 137Q596 46 690 145"
              data-route-path
              pathLength="1"
              strokeWidth="1.25"
            />
            <path
              className="stroke-brand/70"
              d="M690 145Q772 211 748 300"
              data-route-path
              pathLength="1"
              strokeWidth="1.5"
            />
            <path
              className="stroke-border-strong/85"
              d="M229 267Q348 196 481 220"
              data-route-path
              pathLength="1"
              strokeWidth="1.25"
            />
            <path
              className="stroke-border/75"
              d="M178 145Q439 -42 690 145"
              data-route-path
              pathLength="1"
              strokeWidth="1"
            />
          </g>

          <g>
            {routeNodes.map((node) => (
              <g data-route-node key={node.id}>
                <circle
                  className="fill-background/80 stroke-brand/70"
                  cx={node.x}
                  cy={node.y}
                  r="7"
                  strokeWidth="1"
                />
                <circle className="fill-brand" cx={node.x} cy={node.y} r="2.5" />
              </g>
            ))}
          </g>
        </svg>

        <div
          className="absolute bottom-[10%] left-[4%] border-l border-brand/70 pl-4 sm:bottom-[8%] sm:left-[8%]"
          data-global-metric
        >
          <p className="text-[clamp(4.75rem,10vw,8.5rem)] font-semibold leading-[0.72] tracking-[-0.075em] text-foreground">
            156
          </p>
          <p className="mt-4 text-label text-brand">Countries</p>
          <p className="mt-1 text-body-sm text-muted-foreground">
            One connected journey
          </p>
        </div>

        <ul
          aria-label="Global regions"
          className="absolute inset-x-2 bottom-0 flex flex-wrap justify-end gap-x-5 gap-y-2 text-caption text-muted-foreground md:inset-x-4"
          data-global-regions
        >
          {regions.map((region) => (
            <li key={region}>{region}</li>
          ))}
        </ul>
      </div>
    </Container>
  </section>
);

export { GlobalReachStory };
