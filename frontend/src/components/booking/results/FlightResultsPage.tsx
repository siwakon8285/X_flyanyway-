"use client";

import { ArrowLeft } from "lucide-react";
import { buttonVariants } from "@/components/ui/Button";
import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import { FLIGHT_RESULT_FIXTURES } from "@/components/booking/results/flightResultFixtures";
import { FlightResultsList } from "@/components/booking/results/FlightResultsList";
import { FlightResultsState } from "@/components/booking/results/FlightResultsState";
import { filterFlightResults } from "@/components/booking/results/flightResultUtils";
import { SearchSummary } from "@/components/booking/results/SearchSummary";
import { Container } from "@/components/layout/Container";
import { useLanguage } from "@/i18n/LanguageProvider";

const FlightResultsPage = ({
  criteria,
  query,
}: {
  criteria: FlightSearchFormValues | null;
  query: string;
}) => {
  const { t } = useLanguage();

  if (!criteria) {
    return (
      <Container>
        <FlightResultsState
          description={t("flightResults.requiredDescription")}
          title={t("flightResults.requiredTitle")}
        >
          {/* eslint-disable-next-line @next/next/no-html-link-for-pages */}
          <a className={`${buttonVariants({ size: "lg" })} mt-8`} href="/#flight-search">
            <ArrowLeft aria-hidden="true" />
            {t("flightResults.modifySearch")}
          </a>
        </FlightResultsState>
      </Container>
    );
  }

  const flights = filterFlightResults(FLIGHT_RESULT_FIXTURES, criteria);

  return (
    <section className="relative min-h-screen overflow-hidden py-section-sm pt-[calc(var(--header-height)+clamp(3rem,7vw,6rem))]">
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-x-0 top-0 h-[32rem] bg-[radial-gradient(circle_at_78%_10%,rgba(255,212,0,0.1),transparent_28rem)]"
      />
      <Container className="relative">
        <SearchSummary criteria={criteria} query={query} />
        <div className="pt-9 lg:pt-12">
          {flights.length > 0 ? (
            <FlightResultsList cabin={criteria.cabin} flights={flights} query={query} />
          ) : (
            <FlightResultsState
              description={t("flightResults.emptyDescription")}
              headingLevel={2}
              title={t("flightResults.emptyTitle")}
            />
          )}
        </div>
      </Container>
    </section>
  );
};

export { FlightResultsPage };
