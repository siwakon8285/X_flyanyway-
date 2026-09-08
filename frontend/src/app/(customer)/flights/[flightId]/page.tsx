import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { FlightDetailPage } from "@/components/booking/detail/FlightDetailPage";
import {
  resolveFlightDetailRequest,
  type RouteQuery,
} from "@/components/booking/detail/flightDetailUtils";
import { isCompleteFlightSearchQuery, parseFlightSearch } from "@/components/booking/search/searchState";
import { fetchPublicAirports, fetchPublicFlightDetail } from "@/lib/flights/publicFlightBackend";

export const metadata: Metadata = {
  description: "Explore an X-Fly flight and compare Business and First cabin experiences.",
  title: "Flight Detail · X-Fly Anyway",
};

export default async function FlightDetailRoute({
  params,
  searchParams,
}: {
  params: Promise<{ flightId: string }>;
  searchParams: Promise<RouteQuery>;
}) {
  const [{ flightId }, query, airportsResult] = await Promise.all([
    params,
    searchParams,
    fetchPublicAirports(),
  ]);
  const airports = airportsResult ?? [];
  const raw = new URLSearchParams();
  Object.entries(query).forEach(([key, value]) => {
    if (typeof value === "string") raw.set(key, value);
  });
  if (!isCompleteFlightSearchQuery(raw, airports)) notFound();
  const authoritative = await fetchPublicFlightDetail(
    flightId,
    parseFlightSearch(raw, airports),
  );
  if (!authoritative) notFound();
  const request = resolveFlightDetailRequest(flightId, query, airports, authoritative);

  if (!request) notFound();

  return <FlightDetailPage {...request} />;
}
