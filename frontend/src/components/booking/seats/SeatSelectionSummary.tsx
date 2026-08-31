"use client";

import { ArrowRight } from "lucide-react";

import { cabinLabelKeys } from "@/components/booking/cabin/cabinPresentation";
import type { SeatSelectionRequest } from "@/components/booking/detail/flightDetailUtils";
import { SeatLegend } from "@/components/booking/seats/SeatLegend";
import { sortSeatNumbers } from "@/components/booking/seats/seatMapUtils";
import { Button } from "@/components/ui/Button";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

const formatCountdown = (milliseconds: number) => {
  const totalSeconds = Math.max(0, Math.ceil(milliseconds / 1_000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
};

const SeatSelectionSummary = ({
  continuePending,
  holdActive,
  limitReached,
  messageKey,
  mutationPending,
  onContinue,
  remainingMilliseconds,
  request,
  requiredSeatCount,
  selectedSeatIds,
}: {
  continuePending: boolean;
  holdActive: boolean;
  limitReached: boolean;
  messageKey: TranslationKey | null;
  mutationPending: boolean;
  onContinue: () => void;
  remainingMilliseconds: number | null;
  request: SeatSelectionRequest;
  requiredSeatCount: number;
  selectedSeatIds: ReadonlySet<string>;
}) => {
  const selectedSeats = sortSeatNumbers([...selectedSeatIds]);
  const isComplete = selectedSeats.length === requiredSeatCount;
  const { t } = useLanguage();
  const holdState = mutationPending ? "pending" : holdActive ? "confirmed" : "none";

  return (
    <aside
      className="rounded-surface border border-border bg-surface/85 p-6 backdrop-blur-sm lg:sticky lg:top-[calc(var(--header-height)+1.5rem)]"
      data-hold-state={holdState}
    >
      <SeatLegend />
      <div className="my-6 h-px bg-border" />
      <section aria-labelledby="selected-seats-heading">
        <p className="text-caption" id="selected-seats-heading">
          {t("seatMap.yourSeats")}
        </p>
        <div className="mt-4 flex min-h-11 flex-wrap gap-2">
          {selectedSeats.length > 0 ? (
            selectedSeats.map((seat) => (
              <span
                className="inline-flex h-10 min-w-12 items-center justify-center rounded-control border border-brand/55 bg-brand/10 px-3 text-sm font-semibold text-brand"
                key={seat}
              >
                {seat}
              </span>
            ))
          ) : (
            <span className="self-center text-sm text-muted-foreground">
              {t("seatMap.noSeats")}
            </span>
          )}
        </div>
        <p aria-live="polite" className="mt-4 text-sm font-medium">
          {requiredSeatCount === 1
            ? t("seatMap.selectedCountOne", { selected: selectedSeats.length })
            : t("seatMap.selectedCount", {
                required: requiredSeatCount,
                selected: selectedSeats.length,
              })}
        </p>
        <p className="mt-1 text-xs text-muted-foreground">
          {t(cabinLabelKeys[request.selectedCabin])} · {request.flight.flightNumber}
        </p>
        {holdActive && remainingMilliseconds !== null ? (
          <p
            aria-live="polite"
            className="mt-4 text-sm font-semibold tabular-nums text-brand"
          >
            {t("seatMap.holdCountdown", {
              time: formatCountdown(remainingMilliseconds),
            })}
          </p>
        ) : null}
        <p aria-live="polite" className="mt-4 min-h-5 text-xs text-brand">
          {limitReached ? t("seatMap.deselect") : messageKey ? t(messageKey) : ""}
        </p>
        {isComplete ? (
          <Button
            aria-label={
              requiredSeatCount === 1
                ? t("seatMap.continueAriaOne")
                : t("seatMap.continueAria", { count: requiredSeatCount })
            }
            className="mt-5 w-full transition-[background-color,box-shadow,transform] duration-200 hover:shadow-[0_12px_30px_rgb(255_212_0/0.22)] motion-safe:hover:-translate-y-0.5 motion-safe:hover:scale-[1.02] motion-safe:active:translate-y-0 motion-safe:active:scale-[0.985] motion-reduce:transition-none [&_svg]:transition-transform motion-safe:hover:[&_svg]:translate-x-1.5"
            disabled={continuePending || mutationPending || !holdActive}
            onClick={onContinue}
            size="lg"
          >
            {continuePending ? t("seatMap.continueValidating") : t("seatMap.continue")}
            <ArrowRight aria-hidden="true" />
          </Button>
        ) : (
          <Button
            aria-label={
              requiredSeatCount === 1
                ? t("seatMap.selectSeatToProceed")
                : t("seatMap.selectSeatsToProceed", { count: requiredSeatCount })
            }
            className="mt-5 w-full"
            disabled
            size="lg"
          >
            {t("seatMap.continue")}
            <ArrowRight aria-hidden="true" />
          </Button>
        )}
      </section>
    </aside>
  );
};

export { SeatSelectionSummary };
