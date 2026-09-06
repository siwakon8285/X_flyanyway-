import type { Metadata } from "next";
import { notFound } from "next/navigation";

import {
  resolveSeatSelectionRequest,
  type RouteQuery,
} from "@/components/booking/detail/flightDetailUtils";
import { SeatMapPage } from "@/components/booking/seats/SeatMapPage";
import { getSeatMapFixture } from "@/components/booking/seats/seatMapFixtures";

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
  const [{ flightId }, query] = await Promise.all([params, searchParams]);
  const request = resolveSeatSelectionRequest(flightId, query);

  if (!request) notFound();

  const seatMap = getSeatMapFixture(
    request.flight.aircraft,
    request.selectedCabin,
  );

  return <SeatMapPage request={request} seatMap={seatMap} />;
}
