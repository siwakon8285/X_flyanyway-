import type { Metadata } from "next";

import {
  isCompleteFlightSearchQuery,
  parseFlightSearch,
  serializeFlightSearch,
} from "@/components/booking/search/searchState";
import { FlightResultsPage } from "@/components/booking/results/FlightResultsPage";
import { fetchPublicAirports, fetchPublicFlights } from "@/lib/flights/publicFlightBackend";

type FlightQuery = Record<string, string | string[] | undefined>;

const toUrlSearchParams = (query: FlightQuery) => {
  const params = new URLSearchParams();

  Object.entries(query).forEach(([key, value]) => {
    if (typeof value === "string") params.set(key, value);
  });

  return params;
};

export const metadata: Metadata = {
  description: "Compare available X-Fly flight schedules and premium cabin fares.",
  title: "Flight Results · X-Fly Anyway",
};

export default async function FlightsRoute({
  searchParams,
}: {
  searchParams: Promise<FlightQuery>;
}) {
  const [queryValues, airportsResult] = await Promise.all([
    searchParams,
    fetchPublicAirports(),
  ]);
  const params = toUrlSearchParams(queryValues);
  const airports = airportsResult ?? [];
  const criteria = isCompleteFlightSearchQuery(params, airports)
    ? parseFlightSearch(params, airports)
    : null;
  const query = criteria ? serializeFlightSearch(criteria) : "";
  const flights = criteria ? await fetchPublicFlights(criteria) : [];

  return <FlightResultsPage criteria={criteria} flights={flights} query={query} />;
}
