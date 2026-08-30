"use client";

import type { ReactNode } from "react";
import { SearchX } from "lucide-react";
import { useLanguage } from "@/i18n/LanguageProvider";

const FlightResultsState = ({
  children,
  description,
  headingLevel = 1,
  title,
}: {
  children?: ReactNode;
  description: string;
  headingLevel?: 1 | 2;
  title: string;
}) => {
  const Heading = headingLevel === 1 ? "h1" : "h2";
  const { t } = useLanguage();

  return (
    <section className="flex min-h-[65svh] items-center py-section-md" aria-labelledby="results-state-heading">
      <div className="mx-auto max-w-xl text-center">
        <SearchX aria-hidden="true" className="mx-auto size-10 text-brand" />
        <p className="mt-6 text-label text-brand">{t("flightResults.label")}</p>
        <Heading className="mt-3 text-h2 uppercase" id="results-state-heading">
          {title}
        </Heading>
        <p className="mx-auto mt-5 max-w-md text-body-lg text-muted-foreground">
          {description}
        </p>
        {children}
      </div>
    </section>
  );
};

export { FlightResultsState };
