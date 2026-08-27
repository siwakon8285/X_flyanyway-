"use client";

import { ChevronDown, Minus, Plus, Users } from "lucide-react";

import { PASSENGER_UI_SAFETY_LIMIT } from "@/components/booking/search/searchState";
import type { PassengerCounts } from "@/components/booking/search/searchTypes";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/Dialog";
import { IconButton } from "@/components/ui/IconButton";
import { cn } from "@/lib/utils/cn";

type PassengerSelectorProps = {
  error?: string;
  onChange: (passengers: PassengerCounts) => void;
  value: PassengerCounts;
};

const passengerRows = [
  { description: "12+ years", key: "adults", label: "Adults", minimum: 1 },
  { description: "2–11 years", key: "children", label: "Children", minimum: 0 },
  { description: "Under 2 years", key: "infants", label: "Infants", minimum: 0 },
] as const;

const PassengerSelector = ({ error, onChange, value }: PassengerSelectorProps) => {
  const total = value.adults + value.children + value.infants;
  const errorId = "passengers-error";

  return (
    <div className="min-w-0">
      <Dialog>
        <DialogTrigger asChild>
          <button
            aria-describedby={error ? errorId : undefined}
            aria-label={`Passengers, ${total} total`}
            className={cn(
              "group flex min-h-24 w-full cursor-pointer flex-col justify-between rounded-control border border-border bg-surface/45 px-4 py-4 text-left shadow-[inset_0_1px_0_rgb(255_255_255/0.02)] outline-none transition-[background-color,border-color,box-shadow] duration-150 hover:border-border-strong hover:bg-surface/75 focus-visible:border-focus focus-visible:bg-surface/75 focus-visible:ring-2 focus-visible:ring-focus/35 focus-visible:ring-offset-2 focus-visible:ring-offset-background motion-reduce:transition-none",
              error && "border-destructive",
            )}
            data-invalid={Boolean(error)}
            type="button"
          >
            <span className="flex w-full items-center justify-between gap-3 text-label text-muted-foreground">
              Passengers
              <span className="flex items-center gap-2">
                <Users
                  aria-hidden="true"
                  className="size-4 transition-colors group-hover:text-brand group-focus-visible:text-brand"
                />
                <ChevronDown
                  aria-hidden="true"
                  className="size-4 transition-colors group-hover:text-brand group-focus-visible:text-brand"
                />
              </span>
            </span>
            <span className="mt-4 text-base text-foreground">
              {total} {total === 1 ? "passenger" : "passengers"}
            </span>
          </button>
        </DialogTrigger>
        <DialogContent className="sm:max-w-xl">
          <DialogHeader>
            <DialogTitle>Passengers</DialogTitle>
            <DialogDescription>
              Choose who is travelling. At least one adult is required.
            </DialogDescription>
          </DialogHeader>
          <ul className="divide-y divide-border border-y border-border">
            {passengerRows.map((row) => {
              const count = value[row.key];
              return (
                <li className="flex items-center justify-between gap-6 py-5" key={row.key}>
                  <span>
                    <span className="block text-base text-foreground">{row.label}</span>
                    <span className="block text-body-sm text-muted-foreground">
                      {row.description}
                    </span>
                  </span>
                  <span className="flex items-center gap-3">
                    <IconButton
                      disabled={count <= row.minimum}
                      label={`Decrease ${row.label.toLocaleLowerCase()}`}
                      onClick={() => onChange({ ...value, [row.key]: count - 1 })}
                      size="sm"
                      variant="outline"
                    >
                      <Minus aria-hidden="true" />
                    </IconButton>
                    <output
                      aria-label={`${row.label} count`}
                      className="min-w-8 text-center font-mono text-base"
                    >
                      {count}
                    </output>
                    <IconButton
                      disabled={count >= PASSENGER_UI_SAFETY_LIMIT}
                      label={`Increase ${row.label.toLocaleLowerCase()}`}
                      onClick={() => onChange({ ...value, [row.key]: count + 1 })}
                      size="sm"
                      variant="outline"
                    >
                      <Plus aria-hidden="true" />
                    </IconButton>
                  </span>
                </li>
              );
            })}
          </ul>
          <p className="text-caption text-muted-foreground">
            99 per category is a UI safety guard, not an airline policy.
          </p>
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

export { PassengerSelector };
export type { PassengerSelectorProps };
