import { Container } from "@/components/layout/Container";

const cabins = [
  {
    atmosphere:
      "bg-[radial-gradient(circle_at_78%_48%,rgba(84,105,124,0.13),transparent_43%)]",
    copy: "Ease, considered from departure to arrival.",
    id: "economy",
    label: "Economy",
  },
  {
    atmosphere:
      "bg-[radial-gradient(circle_at_76%_46%,rgba(185,151,91,0.12),transparent_45%)]",
    copy: "More room for the distance ahead.",
    id: "premium-economy",
    label: "Premium Economy",
  },
  {
    atmosphere:
      "bg-[linear-gradient(116deg,transparent_34%,rgba(72,91,112,0.14)_56%,transparent_78%)]",
    copy: "Space to focus. Freedom to arrive ready.",
    id: "business",
    label: "Business",
  },
  {
    atmosphere:
      "bg-[radial-gradient(ellipse_at_82%_50%,rgba(255,212,0,0.085),rgba(95,73,35,0.055)_35%,transparent_67%)]",
    copy: "A private expression of flight.",
    id: "first",
    label: "First",
  },
] as const;

const CabinStory = () => (
  <section
    aria-labelledby="cabins-heading"
    className="relative border-y border-border/70 bg-surface/35"
    data-cabin-story
    id="cabins"
  >
    <div className="relative overflow-hidden" data-cabin-frame>
      <div
        aria-hidden="true"
        className="pointer-events-none absolute -right-[18rem] top-1/2 hidden aspect-[0.68] w-[42rem] -translate-y-1/2 rounded-[50%] border border-border-strong/45 bg-[radial-gradient(ellipse_at_center,rgba(255,212,0,0.055),transparent_64%)] md:block xl:-right-[10rem] xl:w-[50rem]"
        data-cabin-aperture
      />
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-y-0 right-0 hidden w-[58%] bg-[linear-gradient(118deg,transparent_12%,rgba(255,255,255,0.018)_38%,rgba(255,212,0,0.045)_54%,transparent_76%)] opacity-70 md:block"
        data-cabin-light
      />
      <div aria-hidden="true" className="pointer-events-none absolute inset-0">
        {cabins.map((cabin, index) => (
          <div
            className={`absolute inset-0 ${cabin.atmosphere} ${index === 0 ? "opacity-70" : "opacity-0"} motion-reduce:opacity-25`}
            data-cabin-atmosphere
            key={`${cabin.id}-atmosphere`}
          />
        ))}
      </div>

      <Container className="relative pt-section-sm md:min-h-svh md:pt-header-safe">
        <div className="grid gap-5 border-b border-border/80 pb-8 md:grid-cols-[auto_1fr] md:items-end md:justify-between">
          <div>
            <p className="text-label text-brand">Four ways forward</p>
            <h2 className="mt-4 max-w-4xl text-h1 text-balance" id="cabins-heading">
              Choose your way to fly.
            </h2>
          </div>
          <p className="max-w-sm text-body text-muted-foreground md:justify-self-end">
            One standard of confidence, expressed across four distinct journeys.
          </p>
        </div>

        <div className="relative" data-cabin-stage-stack>
          {cabins.map((cabin, index) => (
            <article
              className="relative grid min-h-[68svh] content-center border-b border-border/50 py-16 last:border-b-0 md:grid-cols-[minmax(0,1fr)_minmax(16rem,0.38fr)] md:items-end md:gap-12 md:border-b-0 md:py-12"
              data-cabin-stage
              key={cabin.id}
            >
              <div>
                <p className="text-caption text-muted-foreground">
                  {String(index + 1).padStart(2, "0")} / 04
                </p>
                <h3 className="mt-5 max-w-5xl text-[clamp(3.5rem,9vw,9.5rem)] font-semibold leading-[0.86] tracking-[-0.07em] text-balance">
                  {cabin.label}
                </h3>
              </div>
              <p className="mt-8 max-w-sm text-body-lg text-muted-foreground md:mt-0 md:pb-3">
                {cabin.copy}
              </p>
            </article>
          ))}
        </div>

        <div
          aria-hidden="true"
          className="absolute bottom-10 right-page-gutter hidden items-center gap-3 text-caption text-muted-foreground"
          data-cabin-progress
        >
          <span>01</span>
          <span className="relative h-px w-24 overflow-hidden bg-border-strong">
            <span
              className="absolute inset-y-0 left-0 w-full origin-left bg-brand"
              data-cabin-progress-line
            />
          </span>
          <span>04</span>
        </div>
      </Container>
    </div>
  </section>
);

export { CabinStory };
