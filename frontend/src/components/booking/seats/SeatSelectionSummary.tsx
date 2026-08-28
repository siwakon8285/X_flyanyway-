import { ArrowRight } from "lucide-react";
import Link from "next/link";

import { cabinLabels } from "@/components/booking/cabin/cabinPresentation";
import type { SeatSelectionRequest } from "@/components/booking/detail/flightDetailUtils";
import { SeatLegend } from "@/components/booking/seats/SeatLegend";
import { buildPassengerDetailsHref, sortSeatNumbers } from "@/components/booking/seats/seatMapUtils";
import { Button, buttonVariants } from "@/components/ui/Button";
import { cn } from "@/lib/utils/cn";

const SeatSelectionSummary = ({
  limitReached,
  request,
  requiredSeatCount,
  selectedSeatIds,
}: {
  limitReached: boolean;
  request: SeatSelectionRequest;
  requiredSeatCount: number;
  selectedSeatIds: ReadonlySet<string>;
}) => {
  const selectedSeats = sortSeatNumbers([...selectedSeatIds]);
  const isComplete = selectedSeats.length === requiredSeatCount;
  const continueHref = buildPassengerDetailsHref({
    flightId: request.flight.id,
    query: request.query,
    seats: selectedSeats,
    selectedCabin: request.selectedCabin,
  });

  return (
    <aside className="rounded-surface border border-border bg-surface/85 p-6 backdrop-blur-sm lg:sticky lg:top-[calc(var(--header-height)+1.5rem)]">
      <SeatLegend />
      <div className="my-6 h-px bg-border" />
      <section aria-labelledby="selected-seats-heading">
        <p className="text-caption" id="selected-seats-heading">
          Your seats
        </p>
        <div className="mt-4 flex min-h-11 flex-wrap gap-2">
          {selectedSeats.length > 0 ? (
            selectedSeats.map((seat) => (
              <span
                className="inline-flex h-10 min-w-12 items-center justify-center rounded-control border border-brand/55 bg-brand/10 px-3 text-sm font-semibold text-brand"
                key={seat}
              >
                {seat}
              </span>
            ))
          ) : (
            <span className="self-center text-sm text-muted-foreground">No seats selected</span>
          )}
        </div>
        <p aria-live="polite" className="mt-4 text-sm font-medium">
          {selectedSeats.length} of {requiredSeatCount} seats selected
        </p>
        <p className="mt-1 text-xs text-muted-foreground">
          {cabinLabels[request.selectedCabin]} · {request.flight.flightNumber}
        </p>
        <p aria-live="polite" className="mt-4 min-h-5 text-xs text-brand">
          {limitReached ? "Deselect a seat before choosing another." : ""}
        </p>
        {isComplete ? (
          <Link
            aria-label={`Continue with ${requiredSeatCount} ${requiredSeatCount === 1 ? "seat" : "seats"}`}
            className={cn(
              buttonVariants({ size: "lg" }),
              "mt-5 w-full transition-[background-color,box-shadow,transform] duration-200 hover:shadow-[0_12px_30px_rgb(255_212_0/0.22)] motion-safe:hover:-translate-y-0.5 motion-safe:hover:scale-[1.02] motion-safe:active:translate-y-0 motion-safe:active:scale-[0.985] motion-reduce:transition-none [&_svg]:transition-transform motion-safe:hover:[&_svg]:translate-x-1.5",
            )}
            href={continueHref}
          >
            Continue
            <ArrowRight aria-hidden="true" />
          </Link>
        ) : (
          <Button
            aria-label={`Continue, select ${requiredSeatCount} ${requiredSeatCount === 1 ? "seat" : "seats"} to proceed`}
            className="mt-5 w-full"
            disabled
            size="lg"
          >
            Continue
            <ArrowRight aria-hidden="true" />
          </Button>
        )}
      </section>
    </aside>
  );
};

export { SeatSelectionSummary };
