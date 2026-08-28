import type { Metadata } from "next";
import { notFound } from "next/navigation";

import { FlightDetailPage } from "@/components/booking/detail/FlightDetailPage";
import {
  resolveFlightDetailRequest,
  type RouteQuery,
} from "@/components/booking/detail/flightDetailUtils";

export const metadata: Metadata = {
  description: "Explore an X-Fly flight and compare four cabin experiences.",
  title: "Flight Detail · X-Fly Anyway",
};

export default async function FlightDetailRoute({
  params,
  searchParams,
}: {
  params: Promise<{ flightId: string }>;
  searchParams: Promise<RouteQuery>;
}) {
  const [{ flightId }, query] = await Promise.all([params, searchParams]);
  const request = resolveFlightDetailRequest(flightId, query);

  if (!request) notFound();

  return <FlightDetailPage {...request} />;
}
