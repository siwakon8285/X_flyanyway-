"use client";

import { Container } from "@/components/layout/Container";
import { Skeleton } from "@/components/ui/Skeleton";
import { useLanguage } from "@/i18n/LanguageProvider";

export default function FlightDetailLoading() {
  const { t } = useLanguage();

  return (
    <Container>
      <main
        aria-label={t("loading.flightDetail")}
        className="min-h-screen py-section-sm pt-[calc(var(--header-height)+clamp(2rem,5vw,4rem))]"
        role="status"
      >
        <span className="sr-only">{t("loading.flightDetail")}</span>
        <section className="border-b border-border pb-10" data-detail-skeleton>
          <Skeleton className="h-9 w-36" />
          <Skeleton className="mt-8 h-5 w-28" />
          <Skeleton className="mt-5 h-24 w-full max-w-2xl" />
          <div className="mt-10 grid grid-cols-3 gap-4">
            <Skeleton className="h-28" />
            <Skeleton className="h-20 self-center" />
            <Skeleton className="h-28" />
          </div>
        </section>
        <section className="border-b border-border py-9" data-detail-skeleton>
          <Skeleton className="h-12 w-full" />
        </section>
        <section className="py-section-md" data-detail-skeleton>
          <Skeleton className="h-16 w-full max-w-xl" />
          <Skeleton className="mt-8 h-14 w-full" />
          <Skeleton className="mt-8 h-[34rem] w-full" />
        </section>
      </main>
    </Container>
  );
}
