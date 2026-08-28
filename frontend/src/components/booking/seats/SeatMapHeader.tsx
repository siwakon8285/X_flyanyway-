import { ArrowLeft } from "lucide-react";
import Link from "next/link";

import { cabinLabels } from "@/components/booking/cabin/cabinPresentation";
import { buildFlightDetailHref } from "@/components/booking/detail/flightDetailUtils";
import type { SeatSelectionRequest } from "@/components/booking/detail/flightDetailUtils";
import { cn } from "@/lib/utils/cn";
import { SEAT_MAP_PRESENTATION } from "@/components/booking/seats/seatMapPresentation";

const pluralize = (count: number, singular: string) =>
  `${count} ${singular}${count === 1 ? "" : "s"}`;

const SeatMapHeader = ({
  request,
  requiredSeatCount,
}: {
  request: SeatSelectionRequest;
  requiredSeatCount: number;
}) => {
  const { criteria, flight, query, selectedCabin } = request;
  const { adults, children, infants } = criteria.passengers;
  const totalPassengers = adults + children + infants;
  const presentation = SEAT_MAP_PRESENTATION[selectedCabin];
  const backHref = buildFlightDetailHref({
    flightId: flight.id,
    query,
    selectedCabin,
  });

  return (
    <header>
      <Link
        className="group/back inline-flex min-h-11 items-center gap-2 text-sm text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-2 focus-visible:ring-offset-background"
        href={backHref}
      >
        <ArrowLeft aria-hidden="true" className="size-4 transition-transform duration-200 motion-safe:group-hover/back:-translate-x-1 motion-reduce:transition-none" />
        Back to Flight
      </Link>
      <p className={cn("mt-10 text-label", presentation.accentClass)}>
        {cabinLabels[selectedCabin]}
      </p>
      <h1 className="mt-3 text-h1 uppercase">Select your seat</h1>
      <div className="mt-6 flex flex-wrap items-center gap-x-4 gap-y-2 text-body text-muted-foreground">
        <span className="font-medium text-foreground">
          {flight.flightNumber} · {flight.originCode} → {flight.destinationCode}
        </span>
        <span aria-hidden="true" className="hidden size-1 rounded-full bg-border-strong sm:block" />
        <span>
          {pluralize(totalPassengers, "passenger")}
          {infants > 0
            ? ` · ${pluralize(infants, "lap infant")}`
            : ""}
        </span>
        <span aria-hidden="true" className="hidden size-1 rounded-full bg-border-strong sm:block" />
        <span>Select {pluralize(requiredSeatCount, "seat")}</span>
      </div>
    </header>
  );
};

export { SeatMapHeader };
