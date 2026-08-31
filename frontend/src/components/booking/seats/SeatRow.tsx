"use client";

import { SeatButton } from "@/components/booking/seats/SeatButton";
import { SEAT_MAP_PRESENTATION } from "@/components/booking/seats/seatMapPresentation";
import type { SeatRow as SeatRowData } from "@/components/booking/seats/seatMapTypes";
import { cn } from "@/lib/utils/cn";
import { useLanguage } from "@/i18n/LanguageProvider";

const SeatRow = ({
  interactionLocked,
  onToggle,
  pendingSeatIds,
  row,
  selectedSeatIds,
}: {
  interactionLocked: boolean;
  onToggle: (seatId: string) => void;
  pendingSeatIds: ReadonlySet<string>;
  row: SeatRowData;
  selectedSeatIds: ReadonlySet<string>;
}) => {
  const { t } = useLanguage();
  const cabin = row.groups[0]?.[0]?.cabin;
  if (!cabin) return null;
  const presentation = SEAT_MAP_PRESENTATION[cabin];

  return (
    <li
      aria-label={t("seatMap.rowAria", { row: row.row })}
      className={cn(
        "flex items-center justify-center gap-1",
        cabin === "economy"
          ? "min-[26rem]:gap-2.5"
          : cabin === "first"
            ? "min-[23rem]:gap-2"
            : "min-[23rem]:gap-2.5",
      )}
    >
      <span
        aria-hidden="true"
        className={cn(
          "w-5 shrink-0 text-center text-[0.68rem] font-semibold tabular-nums text-muted-foreground",
          cabin === "economy" ? "min-[26rem]:w-6" : "min-[23rem]:w-6",
        )}
        data-row-number
      >
        {row.row}
      </span>
      <div className={cn("flex items-center", presentation.rowClass)}>
        {row.groups.map((group, groupIndex) => (
          <div className="contents" key={`${row.row}-${groupIndex}`}>
            {groupIndex > 0 ? (
              <span
                aria-hidden="true"
                className={cn(
                  "relative h-px shrink-0 bg-current/20",
                  cabin === "business"
                    ? "mx-2 w-7"
                    : cabin === "first"
                      ? "mx-1 w-10 min-[23rem]:w-14"
                      : "mx-0.5 w-2 min-[26rem]:mx-2 min-[26rem]:w-5",
                  presentation.accentClass,
                )}
              />
            ) : null}
            {group.map((seat) => (
              <SeatButton
                interactionLocked={interactionLocked}
                key={seat.id}
                onToggle={onToggle}
                pending={pendingSeatIds.has(seat.id)}
                seat={seat}
                selected={selectedSeatIds.has(seat.id)}
              />
            ))}
          </div>
        ))}
      </div>
      <span
        aria-hidden="true"
        className="hidden w-6 shrink-0 text-center text-[0.68rem] font-semibold tabular-nums text-muted-foreground sm:block"
      >
        {row.row}
      </span>
    </li>
  );
};

export { SeatRow };
