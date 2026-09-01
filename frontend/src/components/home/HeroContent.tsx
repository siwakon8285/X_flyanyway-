"use client";

import { ArrowDown } from "lucide-react";
import Link from "next/link";

import { Container } from "@/components/layout/Container";

import { SplitText } from "@/components/motion/SplitText";
import { useLanguage } from "@/i18n/LanguageProvider";

const HeroContent = () => {
  const { t } = useLanguage();

  return (
  <Container
    className="relative z-10 flex min-h-svh flex-col justify-between pb-[max(2rem,env(safe-area-inset-bottom))] pt-[calc(var(--header-height)+2rem)] sm:pb-10 lg:pb-12"
    data-hero-content
  >
    <div className="max-w-[72rem] pt-8 sm:pt-12">
      <div
        aria-hidden="true"
        className="mb-5 h-px w-16 origin-left bg-brand sm:mb-7 sm:w-20"
        data-hero-line
      />
      <p className="text-label text-brand" data-hero-eyebrow>
        {t("home.hero.eyebrow")}
      </p>
      <SplitText
        animate={false}
        as="h1"
        className="mt-4 max-w-[11ch] text-balance text-[clamp(3rem,8vw,7rem)] font-semibold uppercase leading-[0.84] tracking-[-0.07em] text-foreground sm:mt-6"
        data-hero-headline
        id="hero-heading"
        split="words"
        text={t("home.hero.headline")}
      />
      <div className="mt-7 max-w-2xl" data-hero-details>
        <p className="max-w-lg text-body-lg text-foreground/78">
          {t("home.hero.body")}
        </p>
      </div>
    </div>

    <Link
      aria-label={t("home.hero.scrollAria")}
      className="absolute bottom-[max(1.5rem,env(safe-area-inset-bottom))] left-page-gutter inline-flex items-center gap-3 rounded-sm text-caption text-foreground/65 outline-none transition-colors hover:text-brand focus-visible:ring-2 focus-visible:ring-focus lg:left-auto lg:right-page-gutter"
      data-hero-scroll-cue
      href="/#explore"
    >
      <span data-hero-scroll-label>{t("home.hero.scroll")}</span>
      <ArrowDown aria-hidden="true" className="size-4 text-brand" />
    </Link>
  </Container>
  );
};

export { HeroContent };
