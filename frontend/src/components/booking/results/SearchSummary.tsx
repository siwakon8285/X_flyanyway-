import { ArrowLeft, CalendarDays, UsersRound } from "lucide-react";
import { cabinLabels } from "@/components/booking/cabin/cabinPresentation";
import { formatSearchDate } from "@/components/booking/results/flightResultUtils";
import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import { buttonVariants } from "@/components/ui/Button";

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
      <a
        className={`${buttonVariants({ size: "sm", variant: "ghost" })} transition-[background-color,color,transform] duration-200 hover:text-brand motion-safe:hover:-translate-x-0.5 motion-safe:active:translate-x-0 motion-safe:active:scale-[0.985] motion-reduce:transition-none [&_svg]:transition-transform [&_svg]:duration-200 motion-safe:hover:[&_svg]:-translate-x-1`}
        href={`/?${query}#flight-search`}
      >
        <ArrowLeft aria-hidden="true" />
        Modify Search
      </a>
      <div className="mt-9">
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

export { SearchSummary };
