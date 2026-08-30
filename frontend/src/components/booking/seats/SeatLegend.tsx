"use client";

import { Check, Slash, UserRound } from "lucide-react";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

const legendItems = [
  { icon: null, labelKey: "seatMap.available", swatch: "border-white/35 bg-white/10" },
  { icon: Check, labelKey: "seatMap.selected", swatch: "border-brand bg-brand/18 text-brand" },
  { icon: UserRound, labelKey: "seatMap.booked", swatch: "border-white/12 bg-[#24272c] text-[#828993]" },
  { icon: Slash, labelKey: "seatMap.unavailable", swatch: "border-white/8 bg-[#111318] text-[#626873]" },
] as const satisfies readonly { icon: typeof Check | null; labelKey: TranslationKey; swatch: string }[];

const SeatLegend = () => {
  const { t } = useLanguage();

  return (
  <section aria-labelledby="seat-legend-heading">
    <h2 className="text-caption" id="seat-legend-heading">
      {t("seatMap.legend")}
    </h2>
    <ul className="mt-4 grid grid-cols-2 gap-3 text-xs text-muted-foreground">
      {legendItems.map(({ icon: Icon, labelKey, swatch }) => (
        <li className="flex items-center gap-2" key={labelKey}>
          <span className={`flex size-7 items-center justify-center rounded-lg border ${swatch}`}>
            {Icon ? <Icon aria-hidden="true" className="size-3.5" /> : null}
          </span>
          {t(labelKey)}
        </li>
      ))}
    </ul>
  </section>
  );
};

export { SeatLegend };
