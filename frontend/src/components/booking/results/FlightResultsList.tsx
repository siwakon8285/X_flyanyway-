"use client";

import { useState } from "react";

import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import { FlightResultCard } from "@/components/booking/results/FlightResultCard";
import type {
  FlightResult,
  FlightSortOption,
} from "@/components/booking/results/flightResultTypes";
import { sortFlightResults } from "@/components/booking/results/flightResultUtils";
import { Reveal } from "@/components/motion/Reveal";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

const sortOptions = [
  { labelKey: "flightResults.recommended", value: "recommended" },
  { labelKey: "flightResults.departureTime", value: "departure" },
  { labelKey: "flightResults.duration", value: "duration" },
  { labelKey: "flightResults.price", value: "price" },
] as const satisfies readonly { labelKey: TranslationKey; value: FlightSortOption }[];

const FlightResultsList = ({
  cabin,
  flights,
  query,
}: {
  cabin: FlightSearchFormValues["cabin"];
  flights: readonly FlightResult[];
  query: string;
}) => {
  const [sort, setSort] = useState<FlightSortOption>("recommended");
  const { t } = useLanguage();
  const sortedFlights = sortFlightResults(flights, sort, cabin);

  return (
    <>
      <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-label text-brand">{t("flightResults.outbound")}</p>
          <h2 className="mt-2 text-h3">{t("flightResults.selectOutbound")}</h2>
          <p aria-live="polite" className="mt-2 text-body-sm text-muted-foreground">
            {flights.length === 1 ? t("flightResults.flightCountOne") : t("flightResults.flightCount", { count: flights.length })}
          </p>
        </div>
        <div className="w-full sm:w-56">
          <label className="mb-2 block text-caption text-muted-foreground" htmlFor="flight-sort">
            {t("flightResults.sortFlights")}
          </label>
          <select
            className="h-12 w-full cursor-pointer rounded-control border border-border bg-surface px-4 text-sm text-foreground outline-none transition-[border-color,background-color,box-shadow] duration-200 hover:border-brand/35 hover:bg-surface-elevated hover:shadow-[0_6px_18px_rgb(0_0_0/0.18)] focus-visible:border-focus focus-visible:ring-2 focus-visible:ring-focus/25 motion-reduce:transition-none"
            id="flight-sort"
            onChange={(event) => {
              const nextSort = sortOptions.find(
                (option) => option.value === event.target.value,
              )?.value;
              if (nextSort) setSort(nextSort);
            }}
            value={sort}
          >
            {sortOptions.map((option) => (
              <option key={option.value} value={option.value}>
                {t(option.labelKey)}
              </option>
            ))}
          </select>
        </div>
      </div>
      <Reveal
        aria-label={t("flightResults.availableFlightsLabel")}
        as="ol"
        className="space-y-4"
        stagger={0.07}
      >
        {sortedFlights.map((flight) => (
          <li key={flight.id}>
            <FlightResultCard cabin={cabin} flight={flight} query={query} />
          </li>
        ))}
      </Reveal>
      <p className="mt-6 text-body-sm text-muted-foreground">
        {t("flightResults.localTimes")}
      </p>
    </>
  );
};

export { FlightResultsList };
