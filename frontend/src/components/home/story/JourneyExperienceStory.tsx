import { ArrowRight } from "lucide-react";

import { Container } from "@/components/layout/Container";
import { buttonVariants } from "@/components/ui/Button";
import { cn } from "@/lib/utils/cn";

const journeyValues = ["Comfort", "Control", "Choice"] as const;

const JourneyExperienceStory = () => (
  <section
    aria-labelledby="journey-experience-heading"
    className="relative isolate flex min-h-svh items-center overflow-hidden bg-[radial-gradient(circle_at_12%_82%,rgba(255,212,0,0.07),transparent_28rem),linear-gradient(180deg,#eee5d6_0%,#3b362d_7rem,#090909_16rem,#12100b_100%)] pb-section-lg pt-[clamp(16rem,24vw,24rem)]"
    data-journey-story
    id="journey-experience"
  >
    <Container className="relative">
      <div className="grid gap-12 lg:grid-cols-[minmax(0,0.72fr)_minmax(28rem,1.28fr)] lg:items-end">
        <div data-journey-copy>
          <p className="text-label text-brand">The X-Fly experience</p>
          <h2
            className="mt-4 max-w-3xl text-h1 text-balance"
            id="journey-experience-heading"
          >
            Designed around the journey.
          </h2>
          <p className="mt-6 max-w-md text-body-lg text-muted-foreground">
            Every detail is shaped to give the journey back to you—from the
            first decision to the final arrival.
          </p>
        </div>

        <ol className="border-t border-border-strong/80" data-journey-values>
          {journeyValues.map((value, index) => (
            <li
              className="flex items-baseline justify-between gap-6 border-b border-border-strong/80 py-5 sm:py-7"
              data-journey-value
              key={value}
            >
              <span className="text-[clamp(2.8rem,7vw,7rem)] font-semibold uppercase leading-none tracking-[-0.065em]">
                {value}
              </span>
              <span className="text-caption text-brand">
                {String(index + 1).padStart(2, "0")}
              </span>
            </li>
          ))}
        </ol>
      </div>

      <div
        className="mt-16 grid gap-7 border-t border-border-strong/80 pt-8 md:grid-cols-[minmax(0,1fr)_auto] md:items-end md:gap-10 lg:mt-24"
        data-journey-cta
      >
        <div>
          <p className="text-label text-brand">Your next horizon</p>
          <h3 className="mt-3 max-w-xl text-h2 uppercase text-balance">
            Ready to go anywhere?
          </h3>
          <p className="mt-3 text-body text-muted-foreground">
            Your next journey starts here.
          </p>
        </div>
        <div className="flex flex-col gap-3 sm:flex-row">
          <a className={buttonVariants({ size: "lg" })} href="#journey">
            Book a Flight
            <ArrowRight aria-hidden="true" />
          </a>
          <a
            className={cn(
              buttonVariants({ size: "lg", variant: "outline" }),
              "bg-background/25",
            )}
            href="#cabins"
          >
            Explore Cabins
          </a>
        </div>
      </div>
    </Container>
  </section>
);

export { JourneyExperienceStory };
