import { ArrowLeft } from "lucide-react";
import { notFound } from "next/navigation";

import {
  isCompleteFlightSearchQuery,
  parseFlightSearch,
  serializeFlightSearch,
} from "@/components/booking/search/searchState";
import { FLIGHT_RESULT_FIXTURES } from "@/components/booking/results/flightResultFixtures";
import { Container } from "@/components/layout/Container";
import { buttonVariants } from "@/components/ui/Button";

type RouteQuery = Record<string, string | string[] | undefined>;

const createParams = (query: RouteQuery) => {
  const params = new URLSearchParams();
  Object.entries(query).forEach(([key, value]) => {
    if (typeof value === "string") params.set(key, value);
  });
  return params;
};

export default async function FlightSelectionBoundary({
  params,
  searchParams,
}: {
  params: Promise<{ flightId: string }>;
  searchParams: Promise<RouteQuery>;
}) {
  const { flightId } = await params;
  const flight = FLIGHT_RESULT_FIXTURES.find((item) => item.id === flightId);

  if (!flight) notFound();

  const queryParams = createParams(await searchParams);
  const criteria = isCompleteFlightSearchQuery(queryParams)
    ? parseFlightSearch(queryParams)
    : null;
  const resultsHref = criteria
    ? `/flights?${serializeFlightSearch(criteria)}`
    : "/flights";

  return (
    <Container>
      <section className="flex min-h-[70svh] items-center py-section-md pt-header-safe">
        <div className="max-w-2xl">
          <p className="text-label text-brand">Flight selected</p>
          <h1 className="mt-4 text-h1 uppercase">{flight.flightNumber}</h1>
          <p className="mt-5 text-body-lg text-muted-foreground">
            Flight details continue in the next step.
          </p>
          <a
            className={`${buttonVariants({ size: "lg", variant: "outline" })} mt-8`}
            href={resultsHref}
          >
            <ArrowLeft aria-hidden="true" />
            Back to Flight Results
          </a>
        </div>
      </section>
    </Container>
  );
}
