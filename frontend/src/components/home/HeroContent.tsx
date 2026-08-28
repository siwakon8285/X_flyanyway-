import { ArrowDown } from "lucide-react";

import { Container } from "@/components/layout/Container";

import { SplitText } from "@/components/motion/SplitText";
import { buttonVariants } from "@/components/ui/Button";
import { cn } from "@/lib/utils/cn";

const HeroContent = () => (
  <Container
    className="relative z-10 flex min-h-svh flex-col justify-end pb-[max(2rem,env(safe-area-inset-bottom))] pt-[calc(var(--header-height)+3rem)] sm:pb-10 lg:pb-12"
    data-hero-content
  >
    <div className="max-w-[72rem] pb-16 sm:pb-20 lg:pb-14">
      <div
        aria-hidden="true"
        className="mb-5 h-px w-16 origin-left bg-brand sm:mb-7 sm:w-20"
        data-hero-line
      />
      <p className="text-label text-brand" data-hero-eyebrow>
        X-FLY ANYWAY · GLOBAL AVIATION
      </p>
      <SplitText
        animate={false}
        as="h1"
        className="mt-4 max-w-[11ch] text-balance text-[clamp(3.5rem,10vw,9rem)] font-semibold uppercase leading-[0.84] tracking-[-0.07em] text-foreground sm:mt-6"
        data-hero-headline
        id="hero-heading"
        split="words"
        text="Go anywhere. Fly different."
      />
      <div
        className="mt-7 flex max-w-2xl flex-col gap-6 sm:mt-9 sm:flex-row sm:items-end sm:justify-between sm:gap-10"
        data-hero-details
      >
        <p className="max-w-lg text-body-lg text-foreground/78">
          From the world’s great cities to what comes next. Travel without
          limits.
        </p>
        <div className="flex shrink-0 flex-col gap-3 sm:flex-row" data-hero-actions>
          <a
            className={cn(
              buttonVariants({ size: "lg", variant: "outline" }),
              "justify-start sm:justify-center",
            )}
            href="#journey"
          >
            Explore X-Fly
          </a>
        </div>
      </div>
    </div>

    <a
      aria-label="Scroll to explore X-Fly"
      className="absolute bottom-[max(1.5rem,env(safe-area-inset-bottom))] left-page-gutter inline-flex items-center gap-3 rounded-sm text-caption text-foreground/65 outline-none transition-colors hover:text-brand focus-visible:ring-2 focus-visible:ring-focus lg:left-auto lg:right-page-gutter"
      data-hero-scroll-cue
      href="#journey"
    >
      <span data-hero-scroll-label>Scroll</span>
      <ArrowDown aria-hidden="true" className="size-4 text-brand" />
    </a>
  </Container>
);

export { HeroContent };
