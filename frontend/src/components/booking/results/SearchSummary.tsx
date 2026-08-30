"use client";

import { ArrowLeft, CalendarDays, UsersRound } from "lucide-react";
import { cabinLabelKeys } from "@/components/booking/cabin/cabinPresentation";
import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import { buttonVariants } from "@/components/ui/Button";
import { useLanguage } from "@/i18n/LanguageProvider";
import { formatDate } from "@/i18n/formatters";

const SearchSummary = ({
  criteria,
  query,
}: {
  criteria: FlightSearchFormValues;
  query: string;
}) => {
  const passengerTotal = Object.values(criteria.passengers).reduce(
    (total, count) => total + count,
    0,
  );
  const { locale, t } = useLanguage();

  return (
    <header className="border-b border-border pb-8 lg:pb-10">
      <a
        className={`${buttonVariants({ size: "sm", variant: "ghost" })} transition-[background-color,color,transform] duration-200 hover:text-brand motion-safe:hover:-translate-x-0.5 motion-safe:active:translate-x-0 motion-safe:active:scale-[0.985] motion-reduce:transition-none [&_svg]:transition-transform [&_svg]:duration-200 motion-safe:hover:[&_svg]:-translate-x-1`}
        href={`/?${query}#flight-search`}
      >
        <ArrowLeft aria-hidden="true" />
        {t("flightResults.modifySearch")}
      </a>
      <div className="mt-9">
        <div>
          <p className="text-label text-brand">{t("flightResults.label")}</p>
          <h1 className="mt-4 text-h1 uppercase">
            {criteria.from?.code}
            <span aria-hidden="true" className="mx-[0.22em] text-brand">
              →
            </span>
            <span className="sr-only"> {t("common.to")} </span>
            {criteria.to?.code}
          </h1>
          <p className="mt-3 text-body-lg text-muted-foreground">
            {t("flightResults.routeCities", { from: criteria.from?.city ?? "", to: criteria.to?.city ?? "" })}
          </p>
        </div>
      </div>
      <div className="mt-8 flex flex-wrap gap-x-7 gap-y-4 text-label text-foreground">
        <span className="inline-flex items-center gap-2">
          <CalendarDays aria-hidden="true" className="size-4 text-brand" />
          {formatDate(criteria.departure, locale)}
        </span>
        {criteria.trip === "round-trip" ? (
          <span className="text-muted-foreground">
            {t("common.returnDate", { date: formatDate(criteria.returnDate, locale) })}
          </span>
        ) : (
          <span className="text-muted-foreground">{t("common.oneWay")}</span>
        )}
        <span className="inline-flex items-center gap-2">
          <UsersRound aria-hidden="true" className="size-4 text-brand" />
          {passengerTotal} {t(passengerTotal === 1 ? "common.passenger" : "common.passengers")}
        </span>
        <span>{t(cabinLabelKeys[criteria.cabin])}</span>
      </div>
    </header>
  );
};

export { SearchSummary };
