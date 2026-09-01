"use client";

import { Container } from "@/components/layout/Container";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

const journeyValueKeys = [
  "home.experience.comfort",
  "home.experience.control",
  "home.experience.choice",
] as const satisfies readonly TranslationKey[];

const JourneyExperienceStory = () => {
  const { t } = useLanguage();

  return (
  <section
    aria-labelledby="journey-experience-heading"
    className="relative isolate flex min-h-svh items-center overflow-hidden bg-[radial-gradient(circle_at_12%_82%,rgba(255,212,0,0.07),transparent_28rem),linear-gradient(180deg,#eee5d6_0%,#3b362d_7rem,#090909_16rem,#12100b_100%)] pb-section-lg pt-[clamp(16rem,24vw,24rem)]"
    data-journey-story
    id="experience"
  >
    <Container className="relative">
      <div className="grid gap-12 lg:grid-cols-[minmax(0,0.72fr)_minmax(28rem,1.28fr)] lg:items-end">
        <div data-journey-copy>
          <p className="text-label text-brand">{t("home.experience.label")}</p>
          <h2
            className="mt-4 max-w-3xl text-h1 text-balance"
            id="journey-experience-heading"
          >
            {t("home.experience.heading")}
          </h2>
          <p className="mt-6 max-w-md text-body-lg text-muted-foreground">
            {t("home.experience.body")}
          </p>
        </div>

        <ol className="border-t border-border-strong/80" data-journey-values>
          {journeyValueKeys.map((valueKey, index) => (
            <li
              className="flex items-baseline justify-between gap-6 border-b border-border-strong/80 py-5 sm:py-7"
              data-journey-value
              key={valueKey}
            >
              <span className="text-[clamp(2.8rem,7vw,7rem)] font-semibold uppercase leading-none tracking-[-0.065em]">
                {t(valueKey)}
              </span>
              <span className="text-caption text-brand">
                {String(index + 1).padStart(2, "0")}
              </span>
            </li>
          ))}
        </ol>
      </div>

    </Container>
  </section>
  );
};

export { JourneyExperienceStory };
