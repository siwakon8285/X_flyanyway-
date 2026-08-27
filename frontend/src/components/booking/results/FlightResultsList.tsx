"use client";

import { useState } from "react";

import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import { FlightResultCard } from "@/components/booking/results/FlightResultCard";
import type {
  FlightResult,
  FlightSortOption,
} from "@/components/booking/results/flightResultTypes";
import { sortFlightResults } from "@/components/booking/results/flightResultUtils";

const sortOptions = [
  { label: "Recommended", value: "recommended" },
  { label: "Departure Time", value: "departure" },
  { label: "Duration", value: "duration" },
  { label: "Price", value: "price" },
] as const satisfies readonly { label: string; value: FlightSortOption }[];

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
  const sortedFlights = sortFlightResults(flights, sort, cabin);

  return (
    <>
      <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-label text-brand">Outbound</p>
          <h2 className="mt-2 text-h3">Select your outbound flight</h2>
          <p aria-live="polite" className="mt-2 text-body-sm text-muted-foreground">
            {flights.length} {flights.length === 1 ? "flight" : "flights"} available
          </p>
        </div>
        <div className="w-full sm:w-56">
          <label className="mb-2 block text-caption text-muted-foreground" htmlFor="flight-sort">
            Sort flights
          </label>
          <select
            className="h-12 w-full cursor-pointer rounded-control border border-border bg-surface px-4 text-sm text-foreground outline-none transition-colors hover:border-border-strong focus-visible:border-focus focus-visible:ring-2 focus-visible:ring-focus/25"
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
                {option.label}
              </option>
            ))}
          </select>
        </div>
      </div>
      <ol className="space-y-4" aria-label="Available outbound flights">
        {sortedFlights.map((flight) => (
          <li key={flight.id}>
            <FlightResultCard cabin={cabin} flight={flight} query={query} />
          </li>
        ))}
      </ol>
      <p className="mt-6 text-body-sm text-muted-foreground">
        All times shown in local airport time.
      </p>
    </>
  );
};

export { FlightResultsList };
