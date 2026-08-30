"use client";

import { ArrowLeft, ArrowRight, CalendarDays, Clock3, UsersRound } from "lucide-react";
import Link from "next/link";

import { cabinLabelKeys } from "@/components/booking/cabin/cabinPresentation";
import type { FlightResult } from "@/components/booking/results/flightResultTypes";
import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import { Badge } from "@/components/ui/Badge";
import { buttonVariants } from "@/components/ui/Button";
import { useLanguage } from "@/i18n/LanguageProvider";
import { formatDate, formatDuration } from "@/i18n/formatters";

const FlightDetailSummary = ({
  criteria,
  flight,
  query,
}: {
  criteria: FlightSearchFormValues;
  flight: FlightResult;
  query: string;
}) => {
  const passengerTotal = Object.values(criteria.passengers).reduce(
    (total, count) => total + count,
    0,
  );
  const { locale, t } = useLanguage();

  return (
    <header className="border-b border-border pb-10 lg:pb-14">
      <Link
        className={`${buttonVariants({ size: "sm", variant: "ghost" })} transition-[background-color,color,transform] duration-200 hover:text-brand motion-safe:hover:-translate-x-0.5 motion-safe:active:translate-x-0 motion-safe:active:scale-[0.985] motion-reduce:transition-none [&_svg]:transition-transform [&_svg]:duration-200 motion-safe:hover:[&_svg]:-translate-x-1`}
        href={`/flights?${query}`}
      >
        <ArrowLeft aria-hidden="true" />
        {t("flightDetail.back")}
      </Link>

      <div className="mt-9 flex flex-wrap items-center gap-3">
        <p className="text-label text-brand">{flight.flightNumber}</p>
        <Badge variant="outline">{t("flightDetail.onSchedule")}</Badge>
      </div>

      <h1 className="mt-4 text-h1 uppercase">
        {flight.originCode}
        <span aria-hidden="true" className="mx-[0.22em] text-brand">
          →
        </span>
        <span className="sr-only"> {t("common.to")} </span>
        {flight.destinationCode}
      </h1>

      <div className="mt-9 grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-3 sm:gap-7 lg:max-w-5xl">
        <div>
          <p className="text-[clamp(2.65rem,7vw,6.5rem)] font-semibold leading-none tracking-[-0.06em]">
            {flight.departureTime}
          </p>
          <p className="mt-3 text-h3">{criteria.from?.city}</p>
          <p className="mt-1 text-body-sm text-muted-foreground">
            {criteria.from?.airport}
          </p>
        </div>

        <div className="flex min-w-20 flex-col items-center sm:min-w-40">
          <span className="inline-flex items-center gap-2 text-label">
            <Clock3 aria-hidden="true" className="size-4 text-brand" />
            {formatDuration(flight.durationMinutes, locale)}
          </span>
          <span aria-hidden="true" className="my-3 flex w-full items-center">
            <span className="h-px flex-1 bg-border-strong" />
            <ArrowRight className="size-4 -translate-x-px text-brand" />
          </span>
          <span className="text-caption text-muted-foreground">
            {t(flight.stops === "direct" ? "common.direct" : "common.oneStop")}
          </span>
        </div>

        <div className="min-w-0 text-right">
          <p className="text-[clamp(2.65rem,7vw,6.5rem)] font-semibold leading-none tracking-[-0.06em]">
            {flight.arrivalTime}
            {flight.arrivalDayOffset === 1 ? (
              <sup className="ml-1 align-top text-sm tracking-normal text-brand">+1</sup>
            ) : null}
          </p>
          <p className="mt-3 text-h3">{criteria.to?.city}</p>
          <p className="mt-1 text-body-sm text-muted-foreground">
            {criteria.to?.airport}
          </p>
        </div>
      </div>

      <p className="mt-5 text-body-sm text-muted-foreground">
        {t("flightDetail.localTimes")}
      </p>

      <div className="mt-8 flex flex-wrap gap-x-7 gap-y-4 border-t border-border pt-6 text-label">
        <span className="inline-flex items-center gap-2">
          <CalendarDays aria-hidden="true" className="size-4 text-brand" />
          {formatDate(criteria.departure, locale)}
        </span>
        <span className="text-muted-foreground">
          {criteria.trip === "round-trip"
            ? t("common.returnDate", { date: formatDate(criteria.returnDate, locale) })
            : t("common.oneWay")}
        </span>
        <span className="inline-flex items-center gap-2">
          <UsersRound aria-hidden="true" className="size-4 text-brand" />
          {passengerTotal} {t(passengerTotal === 1 ? "common.passenger" : "common.passengers")}
        </span>
        <span>{t(cabinLabelKeys[criteria.cabin])}</span>
      </div>
    </header>
  );
};

export { FlightDetailSummary };
