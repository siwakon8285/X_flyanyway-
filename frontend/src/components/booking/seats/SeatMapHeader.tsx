"use client";

import { ArrowLeft } from "lucide-react";
import Link from "next/link";

import { cabinLabelKeys } from "@/components/booking/cabin/cabinPresentation";
import { buildFlightDetailHref } from "@/components/booking/detail/flightDetailUtils";
import type { SeatSelectionRequest } from "@/components/booking/detail/flightDetailUtils";
import { cn } from "@/lib/utils/cn";
import { SEAT_MAP_PRESENTATION } from "@/components/booking/seats/seatMapPresentation";
import { useLanguage } from "@/i18n/LanguageProvider";

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
  const { t } = useLanguage();
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
        {t("seatMap.back")}
      </Link>
      <p className={cn("mt-10 text-label", presentation.accentClass)}>
        {t(cabinLabelKeys[selectedCabin])}
      </p>
      <h1 className="mt-3 text-h1 uppercase">{t("seatMap.selectYourSeat")}</h1>
      <div className="mt-6 flex flex-wrap items-center gap-x-4 gap-y-2 text-body text-muted-foreground">
        <span className="font-medium text-foreground">
          {flight.flightNumber} · {flight.originCode} → {flight.destinationCode}
        </span>
        <span aria-hidden="true" className="hidden size-1 rounded-full bg-border-strong sm:block" />
        <span>
          {totalPassengers === 1 ? t("seatMap.passengerCountOne") : t("seatMap.passengerCount", { count: totalPassengers })}
          {infants > 0
            ? ` · ${infants} ${t("seatMap.lapInfant")}`
            : ""}
        </span>
        <span aria-hidden="true" className="hidden size-1 rounded-full bg-border-strong sm:block" />
        <span>{requiredSeatCount === 1 ? t("seatMap.selectRequirementOne") : t("seatMap.selectRequirement", { count: requiredSeatCount })}</span>
      </div>
    </header>
  );
};

export { SeatMapHeader };
