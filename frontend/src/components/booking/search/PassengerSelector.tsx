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
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

type PassengerSelectorProps = {
  error?: string;
  onChange: (passengers: PassengerCounts) => void;
  value: PassengerCounts;
};

const passengerRows = [
  { descriptionKey: "flightSearch.passenger.adultDescription", key: "adults", labelKey: "flightSearch.passenger.adults", minimum: 1 },
  { descriptionKey: "flightSearch.passenger.childDescription", key: "children", labelKey: "flightSearch.passenger.children", minimum: 0 },
  { descriptionKey: "flightSearch.passenger.underTwo", key: "infants", labelKey: "flightSearch.passenger.infants", minimum: 0 },
] as const satisfies readonly { descriptionKey: TranslationKey; key: keyof PassengerCounts; labelKey: TranslationKey; minimum: number }[];

const PassengerSelector = ({ error, onChange, value }: PassengerSelectorProps) => {
  const total = value.adults + value.children + value.infants;
  const errorId = "passengers-error";
  const { t } = useLanguage();

  return (
    <div className="min-w-0">
      <Dialog>
        <DialogTrigger asChild>
          <button
            aria-describedby={error ? errorId : undefined}
            aria-label={t("flightSearch.passenger.totalAria", { count: total })}
            className={cn(
              "group flex min-h-24 w-full cursor-pointer flex-col justify-between rounded-control border border-border bg-surface/45 px-4 py-4 text-left shadow-[inset_0_1px_0_rgb(255_255_255/0.02)] outline-none transition-[background-color,border-color,box-shadow,transform] duration-200 hover:border-brand/40 hover:bg-surface/75 hover:shadow-[0_8px_24px_rgb(255_212_0/0.045)] motion-safe:hover:-translate-y-0.5 motion-safe:hover:scale-[1.01] motion-safe:active:translate-y-0 motion-safe:active:scale-[0.99] focus-visible:border-focus focus-visible:bg-surface/75 focus-visible:ring-2 focus-visible:ring-focus/35 focus-visible:ring-offset-2 focus-visible:ring-offset-background motion-reduce:transition-none",
              error && "border-destructive",
            )}
            data-invalid={Boolean(error)}
            type="button"
          >
            <span className="flex w-full items-center justify-between gap-3 text-label text-muted-foreground">
              {t("flightSearch.passenger.label")}
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
              {total === 1 ? t("flightSearch.passenger.countOne") : t("flightSearch.passenger.count", { count: total })}
            </span>
          </button>
        </DialogTrigger>
        <DialogContent className="sm:max-w-xl">
          <DialogHeader>
            <DialogTitle>{t("flightSearch.passenger.label")}</DialogTitle>
            <DialogDescription>
              {t("flightSearch.passenger.description")}
            </DialogDescription>
          </DialogHeader>
          <ul className="divide-y divide-border border-y border-border">
            {passengerRows.map((row) => {
              const count = value[row.key];
              return (
                <li className="flex items-center justify-between gap-6 py-5" key={row.key}>
                  <span>
                    <span className="block text-base text-foreground">{t(row.labelKey)}</span>
                    <span className="block text-body-sm text-muted-foreground">
                      {t(row.descriptionKey)}
                    </span>
                  </span>
                  <span className="flex items-center gap-3">
                    <IconButton
                      disabled={count <= row.minimum}
                      label={t("flightSearch.passenger.decrease", { label: t(row.labelKey) })}
                      onClick={() => onChange({ ...value, [row.key]: count - 1 })}
                      size="sm"
                      variant="outline"
                    >
                      <Minus aria-hidden="true" />
                    </IconButton>
                    <output
                      aria-label={`${t(row.labelKey)} ${count}`}
                      className="min-w-8 text-center font-mono text-base"
                    >
                      {count}
                    </output>
                    <IconButton
                      disabled={count >= PASSENGER_UI_SAFETY_LIMIT}
                      label={t("flightSearch.passenger.increase", { label: t(row.labelKey) })}
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
            {t("flightSearch.passenger.safety")}
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
