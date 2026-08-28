import { ArrowLeft } from "lucide-react";
import type { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";

import { cabinLabels } from "@/components/booking/cabin/cabinPresentation";
import {
  resolveSeatSelectionRequest,
  type RouteQuery,
} from "@/components/booking/detail/flightDetailUtils";
import { Container } from "@/components/layout/Container";
import { buttonVariants } from "@/components/ui/Button";

export const metadata: Metadata = {
  description: "Continue toward seat selection for an X-Fly flight.",
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

  return (
    <Container>
      <main className="flex min-h-[70svh] items-center py-section-md pt-header-safe">
        <section aria-labelledby="seat-boundary-heading" className="max-w-2xl">
          <p className="text-label text-brand">
            {request.flight.flightNumber} · {request.flight.originCode} to {request.flight.destinationCode}
          </p>
          <h1 className="mt-4 text-h1 uppercase" id="seat-boundary-heading">
            Seat selection
          </h1>
          <p className="mt-5 text-body-lg text-muted-foreground">
            {cabinLabels[request.selectedCabin]} cabin
          </p>
          <p className="mt-3 max-w-xl text-body text-muted-foreground">
            Your flight and cabin are ready. Seat choices continue in the next step.
          </p>
          <Link
            className={`${buttonVariants({ size: "lg", variant: "outline" })} mt-8`}
            href={`/flights/${request.flight.id}?${request.query}`}
          >
            <ArrowLeft aria-hidden="true" />
            Back to Flight Detail
          </Link>
        </section>
      </main>
    </Container>
  );
}
