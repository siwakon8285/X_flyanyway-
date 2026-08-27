import { ArrowRight, Clock3, Plane } from "lucide-react";

import { AIRPORT_FIXTURES } from "@/components/booking/search/airportFixtures";
import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import type { FlightResult } from "@/components/booking/results/flightResultTypes";
import {
  formatDuration,
  getCabinPrice,
} from "@/components/booking/results/flightResultUtils";
import { cabinLabels } from "@/components/booking/results/SearchSummary";
import { buttonVariants } from "@/components/ui/Button";

const formatPrice = (amount: number) =>
  `THB ${new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 }).format(amount)}`;

const FlightResultCard = ({
  cabin,
  flight,
  query,
}: {
  cabin: FlightSearchFormValues["cabin"];
  flight: FlightResult;
  query: string;
}) => {
  const price = getCabinPrice(flight, cabin);
  const titleId = `${flight.id}-title`;
  const origin = AIRPORT_FIXTURES.find((airport) => airport.code === flight.originCode);
  const destination = AIRPORT_FIXTURES.find(
    (airport) => airport.code === flight.destinationCode,
  );

  if (price === null) return null;

  return (
    <article
      aria-labelledby={titleId}
      className="group rounded-surface border border-border bg-surface/75 p-5 transition-[border-color,background-color,box-shadow] duration-200 hover:border-border-strong hover:bg-surface focus-within:border-focus focus-within:ring-2 focus-within:ring-focus/20 sm:p-7 lg:p-8"
    >
      <div className="grid gap-7 xl:grid-cols-[minmax(0,1fr)_minmax(14rem,0.28fr)] xl:items-stretch xl:gap-10">
        <div>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <h3 className="text-label text-foreground" id={titleId}>
              {flight.flightNumber}
            </h3>
            <span className="inline-flex items-center gap-2 text-body-sm text-muted-foreground">
              <Plane aria-hidden="true" className="size-4" />
              {flight.aircraft}
            </span>
          </div>

          <div className="mt-7 grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-3 sm:gap-6">
            <div>
              <p className="text-[clamp(2.25rem,5vw,4.5rem)] font-semibold leading-none tracking-[-0.055em]">
                {flight.departureTime}
              </p>
              <p className="mt-2 text-h3 uppercase">{flight.originCode}</p>
              <p className="mt-1 text-body-sm text-muted-foreground">{origin?.city}</p>
            </div>

            <div className="flex min-w-20 flex-col items-center sm:min-w-36">
              <p className="inline-flex items-center gap-1.5 text-label text-foreground">
                <Clock3 aria-hidden="true" className="size-4 text-brand" />
                {formatDuration(flight.durationMinutes)}
              </p>
              <div aria-hidden="true" className="my-3 flex w-full items-center">
                <span className="h-px flex-1 bg-border-strong transition-colors group-hover:bg-brand/50" />
                <ArrowRight className="size-4 -translate-x-px text-brand" />
              </div>
              <p className="text-caption text-muted-foreground">
                {flight.stops === "direct" ? "Direct" : "1 stop"}
              </p>
            </div>

            <div className="text-right">
              <p className="text-[clamp(2.25rem,5vw,4.5rem)] font-semibold leading-none tracking-[-0.055em]">
                {flight.arrivalTime}
                {flight.arrivalDayOffset === 1 ? (
                  <sup className="ml-1 align-top text-sm tracking-normal text-brand">+1</sup>
                ) : null}
              </p>
              <p className="mt-2 text-h3 uppercase">{flight.destinationCode}</p>
              <p className="mt-1 text-body-sm text-muted-foreground">
                {destination?.city}
              </p>
            </div>
          </div>
        </div>

        <div className="flex flex-col justify-between gap-5 border-t border-border pt-6 xl:border-l xl:border-t-0 xl:pl-9 xl:pt-0">
          <div>
            <p className="text-caption text-muted-foreground">{cabinLabels[cabin]}</p>
            <p className="mt-2 text-2xl font-semibold tracking-[-0.035em] sm:text-3xl">
              {formatPrice(price)}
            </p>
            <p className="mt-1 text-xs text-muted-foreground">
              Sample fare · per passenger
            </p>
          </div>
          <a
            aria-label={`Select flight ${flight.flightNumber}`}
            className={buttonVariants({ size: "lg" })}
            href={`/flights/${flight.id}?${query}`}
          >
            Select Flight
            <ArrowRight aria-hidden="true" />
          </a>
        </div>
      </div>
    </article>
  );
};

export { FlightResultCard };
