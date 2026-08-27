"use client";

import { ArrowRight, ArrowUpDown } from "lucide-react";
import { useRef, useState, type FormEvent } from "react";

import { AirportSelector } from "@/components/booking/search/AirportSelector";
import { CabinSelector } from "@/components/booking/search/CabinSelector";
import { DateSelector } from "@/components/booking/search/DateSelector";
import { PassengerSelector } from "@/components/booking/search/PassengerSelector";
import {
  getTodayDateInputValue,
  validateFlightSearch,
} from "@/components/booking/search/searchState";
import type {
  AirportOption,
  CabinClass,
  FlightSearchErrors,
  FlightSearchFormValues,
  PassengerCounts,
  TripType,
} from "@/components/booking/search/searchTypes";
import { Button } from "@/components/ui/Button";
import { IconButton } from "@/components/ui/IconButton";
import { cn } from "@/lib/utils/cn";

type FlightSearchFormProps = {
  initialValues: FlightSearchFormValues;
  onValidSubmit: (values: FlightSearchFormValues) => void;
};

const tripTypes = [
  { label: "Round Trip", value: "round-trip" },
  { label: "One Way", value: "one-way" },
] as const satisfies readonly { label: string; value: TripType }[];

const FlightSearchForm = ({ initialValues, onValidSubmit }: FlightSearchFormProps) => {
  const form = useRef<HTMLFormElement>(null);
  const [errors, setErrors] = useState<FlightSearchErrors>({});
  const [submitted, setSubmitted] = useState(false);
  const [values, setValues] = useState<FlightSearchFormValues>(() => ({
    ...initialValues,
    passengers: { ...initialValues.passengers },
  }));
  const today = getTodayDateInputValue();

  const clearError = (field: keyof FlightSearchErrors) => {
    setErrors((current) => {
      if (!current[field]) return current;
      const next = { ...current };
      delete next[field];
      return next;
    });
    setSubmitted(false);
  };

  const setAirport = (field: "from" | "to", airport: AirportOption) => {
    clearError(field);
    setValues((current) => ({ ...current, [field]: airport }));
  };
  const setCabin = (cabin: CabinClass) => {
    setSubmitted(false);
    setValues((current) => ({ ...current, cabin }));
  };
  const setPassengers = (passengers: PassengerCounts) => {
    clearError("passengers");
    setValues((current) => ({ ...current, passengers }));
  };

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const nextErrors = validateFlightSearch(values, today);
    if (Object.keys(nextErrors).length > 0) {
      setErrors(nextErrors);
      setSubmitted(false);
      window.requestAnimationFrame(() => {
        form.current
          ?.querySelector<HTMLElement>('[data-invalid="true"], [aria-invalid="true"]')
          ?.focus();
      });
      return;
    }

    setErrors({});
    onValidSubmit(values);
    setSubmitted(true);
  };

  return (
    <form
      aria-label="Search Earth flights"
      noValidate
      onSubmit={handleSubmit}
      ref={form}
    >
      <fieldset>
        <legend className="sr-only">Trip type</legend>
        <div
          className="inline-flex rounded-control border border-border bg-background/55 p-1 shadow-[inset_0_1px_0_rgb(255_255_255/0.025)]"
          role="presentation"
        >
          {tripTypes.map((tripType) => (
            <label className="relative cursor-pointer" key={tripType.value}>
              <input
                checked={values.trip === tripType.value}
                className="peer sr-only"
                name="trip-type"
                onChange={() => {
                  clearError("returnDate");
                  setValues((current) => ({ ...current, trip: tripType.value }));
                }}
                type="radio"
                value={tripType.value}
              />
              <span className="inline-flex min-h-10 items-center rounded-[calc(var(--radius-control)-0.2rem)] border border-transparent px-4 text-sm font-medium text-muted-foreground outline-none transition-[color,background-color,border-color,box-shadow] duration-150 hover:bg-surface/70 hover:text-foreground peer-checked:border-brand/45 peer-checked:bg-surface-elevated peer-checked:text-foreground peer-checked:shadow-[inset_0_-2px_0_var(--brand),0_4px_14px_rgb(0_0_0/0.2)] peer-focus-visible:ring-2 peer-focus-visible:ring-focus peer-focus-visible:ring-offset-2 peer-focus-visible:ring-offset-background motion-reduce:transition-none">
                {tripType.label}
              </span>
            </label>
          ))}
        </div>
      </fieldset>

      <div className="mt-7 grid items-center gap-3 md:grid-cols-[minmax(0,1fr)_3rem_minmax(0,1fr)] md:gap-5">
        <AirportSelector
          error={errors.from}
          label="From"
          onSelect={(airport) => setAirport("from", airport)}
          value={values.from}
        />
        <div className="flex h-12 items-center justify-center md:h-full">
          <IconButton
            className="size-11 rounded-full border-border-strong bg-surface-elevated text-muted-foreground shadow-[0_6px_18px_rgb(0_0_0/0.24)] transition-[color,background-color,border-color,transform,box-shadow] duration-150 hover:border-brand/70 hover:bg-surface-highlight hover:text-brand hover:shadow-[0_8px_22px_rgb(0_0_0/0.32)] active:rotate-180 focus-visible:border-brand motion-reduce:transition-none"
            label="Swap origin and destination"
            onClick={() => {
              clearError("from");
              clearError("to");
              setValues((current) => ({ ...current, from: current.to, to: current.from }));
            }}
            variant="outline"
          >
            <ArrowUpDown aria-hidden="true" className="size-4 md:rotate-90" />
          </IconButton>
        </div>
        <AirportSelector
          error={errors.to}
          label="To"
          onSelect={(airport) => setAirport("to", airport)}
          value={values.to}
        />
      </div>

      <div
        className={cn(
          "mt-8 grid gap-x-5 gap-y-5 sm:grid-cols-2 lg:grid-cols-4",
          values.trip === "one-way" && "lg:grid-cols-3",
        )}
      >
        <DateSelector
          error={errors.departure}
          id="departure-date"
          label="Departure date"
          min={today}
          onChange={(departure) => {
            clearError("departure");
            clearError("returnDate");
            setValues((current) => ({
              ...current,
              departure,
              returnDate:
                current.returnDate && current.returnDate < departure
                  ? ""
                  : current.returnDate,
            }));
          }}
          value={values.departure}
        />
        {values.trip === "round-trip" ? (
          <DateSelector
            error={errors.returnDate}
            id="return-date"
            label="Return date"
            min={values.departure || today}
            onChange={(returnDate) => {
              clearError("returnDate");
              setValues((current) => ({ ...current, returnDate }));
            }}
            value={values.returnDate}
          />
        ) : null}
        <PassengerSelector
          error={errors.passengers}
          onChange={setPassengers}
          value={values.passengers}
        />
        <CabinSelector onChange={setCabin} value={values.cabin} />
      </div>

      <div className="mt-8 flex justify-end border-t border-border pt-6">
        <Button
          className="h-14 w-full px-8 text-base shadow-[0_10px_30px_rgb(255_212_0/0.12)] transition-[background-color,transform,box-shadow] duration-150 hover:-translate-y-0.5 hover:shadow-[0_14px_36px_rgb(255_212_0/0.2)] sm:w-auto motion-reduce:transform-none motion-reduce:transition-none"
          size="lg"
          type="submit"
        >
          Search Flights
          <ArrowRight aria-hidden="true" />
        </Button>
      </div>
      {Object.keys(errors).length > 0 ? (
        <p className="mt-5 text-body-sm text-destructive" role="alert">
          Review the highlighted fields before searching.
        </p>
      ) : null}
      {submitted ? (
        <p className="mt-5 text-body-sm text-muted-foreground" role="status">
          Search criteria ready. Flight results will be available in the next step.
        </p>
      ) : null}
    </form>
  );
};

export { FlightSearchForm };
export type { FlightSearchFormProps };
