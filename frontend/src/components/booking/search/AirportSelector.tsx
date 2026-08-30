"use client";

import { ChevronDown, MapPin, Search } from "lucide-react";
import { useState } from "react";

import { AIRPORT_FIXTURES } from "@/components/booking/search/airportFixtures";
import type { AirportOption } from "@/components/booking/search/searchTypes";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/Dialog";
import { Input } from "@/components/ui/Input";
import { cn } from "@/lib/utils/cn";
import { useLanguage } from "@/i18n/LanguageProvider";

type AirportSelectorProps = {
  error?: string;
  kind: "from" | "to";
  onSelect: (airport: AirportOption) => void;
  value: AirportOption | null;
};

const AirportSelector = ({ error, kind, onSelect, value }: AirportSelectorProps) => {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const { t } = useLanguage();
  const label = t(kind === "from" ? "flightSearch.airport.from" : "flightSearch.airport.to");
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const airports = AIRPORT_FIXTURES.filter((airport) =>
    [airport.code, airport.city, airport.airport, airport.country].some((text) =>
      text.toLocaleLowerCase().includes(normalizedQuery),
    ),
  );
  const triggerLabel = value
    ? `${label} ${value.code}, ${value.city}, ${value.country}`
    : t(kind === "from" ? "flightSearch.airport.chooseOrigin" : "flightSearch.airport.chooseDestination");
  const errorId = `${kind}-error`;

  return (
    <div className="min-w-0">
      <Dialog
        onOpenChange={(nextOpen) => {
          setOpen(nextOpen);
          if (!nextOpen) setQuery("");
        }}
        open={open}
      >
        <DialogTrigger asChild>
          <button
            aria-describedby={error ? errorId : undefined}
            aria-label={triggerLabel}
            className={cn(
              "group flex min-h-40 w-full cursor-pointer flex-col justify-between rounded-surface border border-border bg-surface/65 px-5 py-5 text-left shadow-[inset_0_1px_0_rgb(255_255_255/0.025),0_10px_28px_rgb(0_0_0/0.18)] outline-none transition-[background-color,border-color,box-shadow,transform] duration-200 hover:border-brand/45 hover:bg-surface/90 hover:shadow-[inset_0_1px_0_rgb(255_255_255/0.04),0_14px_34px_rgb(0_0_0/0.26),0_0_24px_rgb(255_212_0/0.045)] motion-safe:hover:-translate-y-0.5 motion-safe:hover:scale-[1.01] motion-safe:active:translate-y-0 motion-safe:active:scale-[0.99] focus-visible:border-focus focus-visible:ring-2 focus-visible:ring-focus/45 focus-visible:ring-offset-2 focus-visible:ring-offset-background motion-reduce:transition-none sm:px-6",
              error && "border-destructive",
            )}
            data-invalid={Boolean(error)}
            type="button"
          >
            <span className="flex w-full items-center justify-between gap-4 text-label text-muted-foreground transition-colors group-hover:text-foreground">
              <span className="flex items-center gap-2">
                <MapPin
                  aria-hidden="true"
                  className="size-4 text-muted-foreground transition-colors group-hover:text-brand group-focus-visible:text-brand"
                />
                {label}
              </span>
              <ChevronDown
                aria-hidden="true"
                className="size-4 transition-colors group-hover:text-brand group-focus-visible:text-brand"
              />
            </span>
            {value ? (
              <span className="mt-5 block min-w-0">
                <span className="block text-[clamp(2.75rem,6vw,5.25rem)] font-semibold leading-[0.82] tracking-[-0.07em] text-foreground">
                  {value.code}
                </span>
                <span className="mt-3 block truncate text-body-lg text-foreground">
                  {value.city}
                </span>
                <span className="mt-1 block truncate text-caption text-muted-foreground">
                  {value.country}
                </span>
              </span>
            ) : (
              <span className="mt-8 block">
                <span className="block text-h3 text-foreground">
                  {t(kind === "from" ? "flightSearch.airport.selectOrigin" : "flightSearch.airport.selectDestination")}
                </span>
                <span className="mt-2 block text-body-sm text-muted-foreground">
                  {t("flightSearch.airport.search")}
                </span>
              </span>
            )}
          </button>
        </DialogTrigger>
        <DialogContent className="max-h-[min(46rem,calc(100dvh-2rem))] overflow-hidden p-0 sm:max-w-2xl">
          <DialogHeader className="px-6 pb-0 pt-6 md:px-8 md:pt-8">
            <DialogTitle>{t("flightSearch.airport.search")}</DialogTitle>
            <DialogDescription>
              {t("flightSearch.airport.description")}
            </DialogDescription>
          </DialogHeader>
          <div className="px-6 md:px-8">
            <div className="relative">
              <Search
                aria-hidden="true"
                className="pointer-events-none absolute left-4 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
              />
              <Input
                aria-label={t("flightSearch.airport.search")}
                autoFocus
                className="pl-11"
                onChange={(event) => setQuery(event.target.value)}
                placeholder={t("flightSearch.airport.placeholder")}
                value={query}
              />
            </div>
          </div>
          <div className="min-h-0 overflow-y-auto px-6 pb-6 md:px-8 md:pb-8">
            <p className="mb-3 text-caption text-muted-foreground">
              {normalizedQuery ? t("flightSearch.airport.matching") : t("flightSearch.airport.featured")}
            </p>
            {airports.length > 0 ? (
              <ul className="divide-y divide-border border-y border-border">
                {airports.map((airport) => (
                  <li key={airport.code}>
                    <button
                      className="grid min-h-20 w-full grid-cols-[4rem_minmax(0,1fr)] items-center gap-4 px-2 py-4 text-left outline-none transition-colors hover:bg-muted focus-visible:bg-muted focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-focus"
                      onClick={() => {
                        onSelect(airport);
                        setOpen(false);
                      }}
                      type="button"
                    >
                      <span className="text-xl font-semibold tracking-[-0.04em] text-brand">
                        {airport.code}
                      </span>
                      <span className="min-w-0">
                        <span className="block text-base text-foreground">
                          {airport.city}
                        </span>
                        <span className="block truncate text-body-sm text-muted-foreground">
                          {airport.airport} · {airport.country}
                        </span>
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="border-y border-border py-8 text-body text-muted-foreground">
                {t("flightSearch.airport.noMatch")}
              </p>
            )}
          </div>
        </DialogContent>
      </Dialog>
      {error ? (
        <p className="mt-2 text-body-sm text-destructive" id={errorId}>
          {error}
        </p>
      ) : null}
    </div>
  );
};

export { AirportSelector };
export type { AirportSelectorProps };
