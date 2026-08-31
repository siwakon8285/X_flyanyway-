"use client";

import { Plane } from "lucide-react";

import { AircraftPlanOverview } from "@/components/booking/seats/AircraftPlanOverview";
import { SeatRow } from "@/components/booking/seats/SeatRow";
import { SEAT_MAP_PRESENTATION } from "@/components/booking/seats/seatMapPresentation";
import type {
  AircraftPlanMarker,
  CabinSeatMap,
} from "@/components/booking/seats/seatMapTypes";
import { Reveal } from "@/components/motion/Reveal";
import { cn } from "@/lib/utils/cn";
import { cabinLabelKeys } from "@/components/booking/cabin/cabinPresentation";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

const markerAbbreviations = {
  door: "seatMap.abbreviations.door",
  exit: "seatMap.abbreviations.exit",
  galley: "seatMap.abbreviations.galley",
  lavatory: "seatMap.abbreviations.lavatory",
} as const satisfies Record<string, TranslationKey>;

type PositionedMarkerKind = keyof typeof markerAbbreviations;

const isPositionedMarker = (
  marker: AircraftPlanMarker,
): marker is AircraftPlanMarker & { kind: PositionedMarkerKind } =>
  marker.kind in markerAbbreviations;

const AircraftSeatMap = ({
  interactionLocked,
  map,
  onToggle,
  pendingSeatIds,
  selectedSeatIds,
}: {
  interactionLocked: boolean;
  map: CabinSeatMap;
  onToggle: (seatId: string) => void;
  pendingSeatIds: ReadonlySet<string>;
  selectedSeatIds: ReadonlySet<string>;
}) => {
  const presentation = SEAT_MAP_PRESENTATION[map.cabin];
  const { t } = useLanguage();
  const isBusiness = map.cabin === "business";
  const hasWingContext = map.section.markers.some((marker) => marker.kind === "wing");
  const hasBulkhead = map.section.markers.some((marker) => marker.kind === "bulkhead");

  return (
    <div
      className={cn(
        "relative isolate max-w-full overflow-hidden rounded-surface border",
        presentation.aircraftClass,
      )}
      data-contained-scroll={isBusiness}
      data-seat-map
    >
      <div aria-hidden="true" className={cn("absolute inset-0 -z-10", presentation.glowClass)} />
      <AircraftPlanOverview map={map} />
      {isBusiness ? (
        <p className="px-4 pt-3 text-center text-[0.625rem] text-muted-foreground sm:hidden">
          {t("seatMap.swipe")}
        </p>
      ) : null}
      <div
        className={cn(
          "overscroll-x-contain px-1 py-4 sm:px-4",
          isBusiness ? "overflow-x-auto" : "overflow-x-hidden",
        )}
      >
        <div
          className={cn(
            "relative mx-auto isolate w-full max-w-[48rem] overflow-hidden border-x border-white/14 bg-[#090d12]/82 px-1 pb-10 pt-7 shadow-[inset_18px_0_28px_-24px_rgb(255_255_255/0.18),inset_-18px_0_28px_-24px_rgb(255_255_255/0.18)] sm:px-6",
            isBusiness ? "min-w-[34rem]" : "min-w-fit",
            map.section.location === "foremost" && "rounded-t-[7rem] border-t",
            map.section.location === "mid-rear" && "rounded-b-[4rem] border-b",
          )}
          data-aircraft-family={map.aircraftFamily.id}
        >
          <div
            aria-hidden="true"
            className="absolute inset-y-0 left-1.5 w-px bg-[repeating-linear-gradient(to_bottom,rgba(255,255,255,0.16)_0_5px,transparent_5px_13px)] sm:left-3"
          />
          <div
            aria-hidden="true"
            className="absolute inset-y-0 right-1.5 w-px bg-[repeating-linear-gradient(to_bottom,rgba(255,255,255,0.16)_0_5px,transparent_5px_13px)] sm:right-3"
          />
          {map.section.markers
            .filter(isPositionedMarker)
            .map((marker) => (
              <div
                aria-hidden="true"
                className={cn(
                  "pointer-events-none absolute z-10 flex text-[0.48rem] font-semibold uppercase tracking-[0.08em] text-muted-foreground/75",
                  marker.side === "both"
                    ? "inset-x-0 justify-between"
                    : marker.side === "left"
                      ? "left-0"
                      : "right-0",
                )}
                key={`plan-position-${marker.kind}-${marker.labelKey}`}
                style={{ top: `${15 + marker.positionPercent * 0.72}%` }}
              >
                <span className="border-y border-white/10 bg-[#090d12] px-1 py-0.5">
                  {t(markerAbbreviations[marker.kind])}
                </span>
                {marker.side === "both" ? (
                  <span className="border-y border-white/10 bg-[#090d12] px-1 py-0.5">
                    {t(markerAbbreviations[marker.kind])}
                  </span>
                ) : null}
              </div>
            ))}
          {hasWingContext ? (
            <div aria-hidden="true" className="pointer-events-none absolute inset-x-0 top-[46%] -z-10 h-24">
              <span className="absolute -left-[8%] top-1/2 h-20 w-[36%] -translate-y-1/2 skew-y-[-14deg] border border-white/7 bg-white/[0.018]" />
              <span className="absolute -right-[8%] top-1/2 h-20 w-[36%] -translate-y-1/2 skew-y-[14deg] border border-white/7 bg-white/[0.018]" />
            </div>
          ) : null}

          <div className="mx-auto flex w-fit flex-col items-center">
            <Plane aria-hidden="true" className={cn("size-5 rotate-[-90deg]", presentation.accentClass)} />
            <p className="mt-2 text-[0.62rem] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
              {t("seatMap.aircraftFront")}
            </p>
            <p className={cn("mt-2 text-xs font-semibold uppercase tracking-[0.16em]", presentation.accentClass)}>
              {t(map.section.labelKey)}
            </p>
          </div>

          <ul
            aria-label={t("seatMap.aircraftSectionFeatures")}
            className="mx-auto mt-5 flex max-w-xl flex-wrap items-center justify-center gap-2"
          >
            {map.section.markers.map((marker) => (
              <li
                className="inline-flex items-center gap-1.5 rounded-full border border-white/10 bg-black/25 px-2.5 py-1 text-[0.58rem] font-semibold uppercase tracking-[0.1em] text-muted-foreground"
                data-plan-marker={marker.kind}
                key={`${marker.kind}-${marker.labelKey}`}
              >
                <span
                  aria-hidden="true"
                  className={cn(
                    "size-1.5 rounded-full border border-current",
                    marker.kind === "exit" || marker.kind === "door"
                      ? "bg-brand/60 text-brand"
                      : presentation.accentClass,
                  )}
                />
                {t(marker.labelKey)}
              </li>
            ))}
          </ul>

          {hasBulkhead ? (
            <div aria-hidden="true" className="mx-auto mt-5 flex max-w-xl items-center gap-2">
              <span className="h-px flex-1 bg-white/12" />
              <span className="text-[0.5rem] font-semibold uppercase tracking-[0.16em] text-muted-foreground/60">
                {t("seatMap.bulkhead")}
              </span>
              <span className="h-px flex-1 bg-white/12" />
            </div>
          ) : null}

          <p className="mt-5 text-center text-[0.625rem] font-semibold uppercase tracking-[0.1em] text-muted-foreground/75">
            {t(presentation.columnGuideKey)}
          </p>

          <Reveal
            as="ol"
            aria-label={t("seatMap.seatRowsAria", { cabin: t(cabinLabelKeys[map.cabin]) })}
            className="mt-5 space-y-3.5"
            stagger={0.035}
          >
            {map.rows.map((row) => (
              <SeatRow
                interactionLocked={interactionLocked}
                key={row.row}
                onToggle={onToggle}
                pendingSeatIds={pendingSeatIds}
                row={row}
                selectedSeatIds={selectedSeatIds}
              />
            ))}
          </Reveal>

          <div aria-hidden="true" className="mx-auto mt-8 h-px max-w-xl bg-gradient-to-r from-transparent via-white/15 to-transparent" />
        </div>
      </div>
    </div>
  );
};

export { AircraftSeatMap };
