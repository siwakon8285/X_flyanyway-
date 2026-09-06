"use client";

import { Container } from "@/components/layout/Container";
import { useLanguage } from "@/i18n/LanguageProvider";

export default function ReviewLoading() {
  const { t } = useLanguage();
  return <main className="min-h-screen pt-[calc(var(--header-height)+4rem)]"><Container><div aria-label={t("review.loading")} className="min-h-72 animate-pulse rounded-surface border border-border bg-surface/60 motion-reduce:animate-none" role="status" /></Container></main>;
}
