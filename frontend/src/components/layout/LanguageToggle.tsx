"use client";

import { useLanguage } from "@/i18n/LanguageProvider";

const LanguageToggle = () => {
  const { locale, t, toggleLocale } = useLanguage();

  return (
    <button
      aria-label={t("navigation.languageToggle")}
      className="inline-flex min-h-11 min-w-11 items-center justify-center rounded-control border border-border/80 bg-background/60 px-2.5 text-xs font-semibold tracking-[0.12em] text-muted-foreground outline-none transition-colors hover:border-border-strong hover:bg-surface hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-2 focus-visible:ring-offset-background"
      onClick={toggleLocale}
      type="button"
    >
      {locale.toUpperCase()}
    </button>
  );
};

export { LanguageToggle };
