"use client";

import { CircleGauge, Plane, Sparkles, Wind } from "lucide-react";

import type { FlightResult } from "@/components/booking/results/flightResultTypes";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

const aircraftFeatures = [
  { icon: Plane, labelKey: "flightDetail.aircraftFeatures.widebody" },
  { icon: Sparkles, labelKey: "flightDetail.aircraftFeatures.quiet" },
  { icon: Wind, labelKey: "flightDetail.aircraftFeatures.filtration" },
  { icon: CircleGauge, labelKey: "flightDetail.aircraftFeatures.airframe" },
] as const satisfies readonly { icon: typeof Plane; labelKey: TranslationKey }[];

const AircraftSummary = ({ aircraft }: Pick<FlightResult, "aircraft">) => {
  const { t } = useLanguage();

  return (
  <section
    aria-labelledby="aircraft-summary-heading"
    className="grid gap-6 border-b border-border py-9 lg:grid-cols-[minmax(12rem,0.35fr)_1fr] lg:items-center lg:gap-12"
  >
    <div>
      <p className="text-caption text-muted-foreground">{t("flightDetail.aircraft")}</p>
      <h2 className="mt-2 text-xl font-semibold tracking-[-0.025em]" id="aircraft-summary-heading">
        {aircraft}
      </h2>
    </div>
    <ul className="grid gap-3 text-body-sm text-muted-foreground sm:grid-cols-2 lg:grid-cols-4">
      {aircraftFeatures.map(({ icon: Icon, labelKey }) => (
        <li
          className="group/feature flex items-center gap-2 transition-[color,background-color,transform] duration-200 hover:text-foreground motion-safe:hover:translate-x-1 motion-reduce:transition-none"
          key={labelKey}
        >
          <Icon
            aria-hidden="true"
            className="size-4 shrink-0 text-brand/65 transition-[color,filter] duration-200 group-hover/feature:text-brand group-hover/feature:drop-shadow-[0_0_6px_rgb(255_212_0/0.25)] motion-reduce:transition-none"
          />
          <span>{t(labelKey)}</span>
        </li>
      ))}
    </ul>
  </section>
  );
};

export { AircraftSummary };
