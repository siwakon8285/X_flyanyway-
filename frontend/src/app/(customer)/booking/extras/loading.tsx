"use client";

import { Container } from "@/components/layout/Container";
import { Skeleton } from "@/components/ui/Skeleton";
import { useLanguage } from "@/i18n/LanguageProvider";

export default function TravelExtrasLoading() {
  const { t } = useLanguage();

  return (
    <Container>
      <main
        aria-label={t("travelExtras.loading")}
        className="min-h-screen py-section-sm pt-[calc(var(--header-height)+clamp(2rem,5vw,4rem))]"
        role="status"
      >
        <span className="sr-only">{t("travelExtras.loading")}</span>
        <Skeleton className="h-5 w-32" />
        <Skeleton className="mt-8 h-14 w-full max-w-xl" />
        <Skeleton className="mt-5 h-6 w-full max-w-2xl" />
        <div className="mt-10 grid gap-8 lg:grid-cols-[minmax(0,1fr)_20rem]">
          <Skeleton className="h-[42rem] rounded-surface" />
          <Skeleton className="h-96 rounded-surface" />
        </div>
      </main>
    </Container>
  );
}
