import { CalendarDays, UsersRound } from "lucide-react";

import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import { buttonVariants } from "@/components/ui/Button";

const cabinLabels: Record<FlightSearchFormValues["cabin"], string> = {
  business: "Business",
  economy: "Economy",
  first: "First",
  "premium-economy": "Premium Economy",
};

const formatSearchDate = (value: string) => {
  const [year, month, day] = value.split("-").map(Number);
  return new Intl.DateTimeFormat("en-GB", {
    day: "2-digit",
    month: "short",
    timeZone: "UTC",
    year: "numeric",
  })
    .format(new Date(Date.UTC(year, month - 1, day)))
    .toUpperCase();
};

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

  return (
    <header className="border-b border-border pb-8 lg:pb-10">
      <div className="flex flex-col gap-8 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <p className="text-label text-brand">Flight results</p>
          <h1 className="mt-4 text-h1 uppercase">
            {criteria.from?.code}
            <span aria-hidden="true" className="mx-[0.22em] text-brand">
              →
            </span>
            <span className="sr-only"> to </span>
            {criteria.to?.code}
          </h1>
          <p className="mt-3 text-body-lg text-muted-foreground">
            {criteria.from?.city} to {criteria.to?.city}
          </p>
        </div>
        <a
          className={buttonVariants({ size: "md", variant: "outline" })}
          href={`/?${query}#flight-search`}
        >
          Modify Search
        </a>
      </div>
      <div className="mt-8 flex flex-wrap gap-x-7 gap-y-4 text-label text-foreground">
        <span className="inline-flex items-center gap-2">
          <CalendarDays aria-hidden="true" className="size-4 text-brand" />
          {formatSearchDate(criteria.departure)}
        </span>
        {criteria.trip === "round-trip" ? (
          <span className="text-muted-foreground">
            Return {formatSearchDate(criteria.returnDate)}
          </span>
        ) : (
          <span className="text-muted-foreground">One way</span>
        )}
        <span className="inline-flex items-center gap-2">
          <UsersRound aria-hidden="true" className="size-4 text-brand" />
          {passengerTotal} {passengerTotal === 1 ? "passenger" : "passengers"}
        </span>
        <span>{cabinLabels[criteria.cabin]}</span>
      </div>
    </header>
  );
};

export { cabinLabels, formatSearchDate, SearchSummary };
