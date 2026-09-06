import type { Metadata } from "next";

import {
  isCompleteFlightSearchQuery,
  parseFlightSearch,
  serializeFlightSearch,
} from "@/components/booking/search/searchState";
import { FlightResultsPage } from "@/components/booking/results/FlightResultsPage";

type FlightQuery = Record<string, string | string[] | undefined>;

const toUrlSearchParams = (query: FlightQuery) => {
  const params = new URLSearchParams();

  Object.entries(query).forEach(([key, value]) => {
    if (typeof value === "string") params.set(key, value);
  });

  return params;
};

export const metadata: Metadata = {
  description: "Compare sample X-Fly flight schedules and cabin fares.",
  title: "Flight Results · X-Fly Anyway",
};

export default async function FlightsRoute({
  searchParams,
}: {
  searchParams: Promise<FlightQuery>;
}) {
  const params = toUrlSearchParams(await searchParams);
  const criteria = isCompleteFlightSearchQuery(params)
    ? parseFlightSearch(params)
    : null;
  const query = criteria ? serializeFlightSearch(criteria) : "";

  return <FlightResultsPage criteria={criteria} query={query} />;
}
