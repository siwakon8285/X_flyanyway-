import type { Metadata } from "next";
import { notFound } from "next/navigation";

import {
  resolveSeatSelectionRequest,
  type RouteQuery,
} from "@/components/booking/detail/flightDetailUtils";
import { SeatMapPage } from "@/components/booking/seats/SeatMapPage";
import { getSeatMapFixture } from "@/components/booking/seats/seatMapFixtures";
import { isCompleteFlightSearchQuery, parseFlightSearch } from "@/components/booking/search/searchState";
import { fetchPublicAirports, fetchPublicFlightDetail } from "@/lib/flights/publicFlightBackend";

export const metadata: Metadata = {
  description: "Choose seats for an X-Fly flight.",
  title: "Seat Selection · X-Fly Anyway",
};

export default async function SeatSelectionBoundary({
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
  const raw=new URLSearchParams();Object.entries(query).forEach(([key,value])=>{if(typeof value==="string")raw.set(key,value);});
  if(!isCompleteFlightSearchQuery(raw, airports))notFound();
  const authoritative=await fetchPublicFlightDetail(flightId,parseFlightSearch(raw, airports));
  if(!authoritative)notFound();
  const request = resolveSeatSelectionRequest(flightId, query, airports, authoritative);

  if (!request) notFound();

  const seatMap = getSeatMapFixture(
    request.flight.aircraft,
    request.selectedCabin,
  );

  return <SeatMapPage request={request} seatMap={seatMap} />;
}
