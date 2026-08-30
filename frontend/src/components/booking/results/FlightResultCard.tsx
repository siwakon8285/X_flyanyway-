import { ArrowRight, Clock3, Plane } from "lucide-react";

import { AIRPORT_FIXTURES } from "@/components/booking/search/airportFixtures";
import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import type { FlightResult } from "@/components/booking/results/flightResultTypes";
import { getCabinPrice } from "@/components/booking/results/flightResultUtils";
import { cabinLabelKeys } from "@/components/booking/cabin/cabinPresentation";
import { buttonVariants } from "@/components/ui/Button";
import { useLanguage } from "@/i18n/LanguageProvider";
import { formatDuration, formatPrice } from "@/i18n/formatters";

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
  const { locale, t } = useLanguage();

  if (price === null) return null;

  return (
    <article
      aria-labelledby={titleId}
      className="group rounded-surface border border-border bg-surface/75 p-5 transition-[border-color,background-color,box-shadow,transform] duration-300 hover:border-brand/30 hover:bg-surface hover:shadow-[0_18px_48px_rgb(0_0_0/0.28),0_0_30px_rgb(255_212_0/0.035)] motion-safe:hover:-translate-y-1 motion-safe:hover:scale-[1.005] focus-within:border-focus focus-within:ring-2 focus-within:ring-focus/20 motion-reduce:transition-none sm:p-7 lg:p-8"
    >
      <div className="grid gap-7 xl:grid-cols-[minmax(0,1fr)_minmax(14rem,0.28fr)] xl:items-stretch xl:gap-10">
        <div>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <h3 className="text-label text-foreground" id={titleId}>
              {flight.flightNumber}
            </h3>
            <span className="inline-flex items-center gap-2 text-body-sm text-muted-foreground transition-colors duration-200 group-hover:text-foreground">
              <Plane
                aria-hidden="true"
                className="size-4 transition-colors duration-200 group-hover:text-brand"
              />
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
                {formatDuration(flight.durationMinutes, locale)}
              </p>
              <div aria-hidden="true" className="my-3 flex w-full items-center">
                <span className="h-px flex-1 bg-border-strong transition-colors duration-200 group-hover:bg-brand/60" />
                <ArrowRight className="size-4 -translate-x-px text-brand transition-transform duration-200 motion-safe:group-hover:translate-x-1 motion-reduce:transition-none" />
              </div>
              <p className="text-caption text-muted-foreground">
                {t(flight.stops === "direct" ? "common.direct" : "common.oneStop")}
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
            <p className="text-caption text-muted-foreground">{t(cabinLabelKeys[cabin])}</p>
            <p className="mt-2 text-2xl font-semibold tracking-[-0.035em] transition-[color,text-shadow] duration-200 group-hover:text-white group-hover:[text-shadow:0_0_20px_rgb(255_255_255/0.08)] sm:text-3xl motion-reduce:transition-none">
              {formatPrice(price, locale)}
            </p>
            <p className="mt-1 text-xs text-muted-foreground">
              {t("common.farePerPassenger")}
            </p>
          </div>
          <a
            aria-label={t("flightResults.selectFlightAria", { flight: flight.flightNumber })}
            className={`${buttonVariants({ size: "lg" })} transition-[background-color,box-shadow,transform] duration-200 hover:shadow-[0_12px_30px_rgb(255_212_0/0.2)] motion-safe:hover:-translate-y-0.5 motion-safe:hover:scale-[1.02] motion-safe:active:translate-y-0 motion-safe:active:scale-[0.985] motion-reduce:transition-none [&_svg]:transition-transform [&_svg]:duration-200 motion-safe:hover:[&_svg]:translate-x-1.5`}
            href={`/flights/${flight.id}?${query}`}
          >
            {t("flightResults.selectFlight")}
            <ArrowRight aria-hidden="true" />
          </a>
        </div>
      </div>
    </article>
  );
};

export { FlightResultCard };
