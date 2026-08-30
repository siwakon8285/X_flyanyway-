"use client";

import { cabinLabelKeys } from "@/components/booking/cabin/cabinPresentation";
import { SEAT_MAP_PRESENTATION } from "@/components/booking/seats/seatMapPresentation";
import type { CabinSeatMap } from "@/components/booking/seats/seatMapTypes";
import { cn } from "@/lib/utils/cn";
import { useLanguage } from "@/i18n/LanguageProvider";

const AircraftPlanOverview = ({ map }: { map: CabinSeatMap }) => {
  const { t } = useLanguage();
  const presentation = SEAT_MAP_PRESENTATION[map.cabin];
  const fuselageStart = 26;
  const fuselageLength = 710;
  const highlightStart =
    fuselageStart + (map.section.startPercent / 100) * fuselageLength;
  const highlightWidth =
    ((map.section.endPercent - map.section.startPercent) / 100) *
    fuselageLength;
  const fuselageClipId = `x-fly-widebody-fuselage-${map.cabin}`;

  const fuselagePath =
    "M26 105 C30 90 47 78 72 69 C110 56 165 52 235 51 C330 50 425 50 520 52 C585 53 636 60 675 72 C700 80 720 91 736 100 Q744 105 736 110 C720 119 700 130 675 138 C636 150 585 157 520 158 C425 160 330 160 235 159 C165 158 110 154 72 141 C47 132 30 120 26 105 Z";

  return (
    <div className="border-b border-white/8 px-4 py-5 sm:px-6">
      <div className="flex items-end justify-between gap-4">
        <div>
          <p className="text-[0.58rem] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
            {t("seatMap.aircraftPlan")}
          </p>
          <p className="mt-1 text-sm font-medium text-foreground">
            {map.aircraftFamily.label} · {t(map.section.labelKey)}
          </p>
        </div>
        <p className={cn("text-[0.58rem] font-semibold uppercase tracking-[0.16em]", presentation.accentClass)}>
          {t(cabinLabelKeys[map.cabin])}
        </p>
      </div>

      <div
        aria-label={t("seatMap.aircraftOverview", { aircraft: map.aircraftFamily.label, section: t(map.section.labelKey), cabin: t(cabinLabelKeys[map.cabin]) })}
        className="mt-5"
        data-aircraft-family={map.aircraftFamily.id}
        role="img"
      >
        <div className="relative mx-auto max-w-2xl pt-4" aria-hidden="true">
          <span className="absolute left-0 top-0 text-[0.5rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
            {t("seatMap.front")}
          </span>
          <span className="absolute right-0 top-0 text-[0.5rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
            {t("seatMap.rear")}
          </span>
          <svg
            className="mx-auto h-auto max-h-32 w-full overflow-visible"
            data-aircraft-profile="tapered-widebody"
            data-aircraft-silhouette="widebody"
            preserveAspectRatio="xMidYMid meet"
            viewBox="0 -85 760 380"
          >
            <defs>
              <clipPath id={fuselageClipId}>
                <path d={fuselagePath} />
              </clipPath>
            </defs>

            <g fill="#101820" stroke="rgb(255 255 255 / 0.14)" strokeWidth="1.5">
              <path
                d="M 270 51 Q 288 51 305 40 L 440 -45 Q 456 -55 475 -65 C 479 -67 484 -62 480 -58 Q 472 -50 465 -30 L 458 -10 Q 456 20 455 51 Z"
                data-aircraft-part="main-wing-left"
                data-flight-surface-scale="primary"
                data-wing-planform="asymmetric-swept"
              />
              <path
                d="M 270 159 Q 288 159 305 170 L 440 255 Q 456 265 475 275 C 479 277 484 272 480 268 Q 472 260 465 240 L 458 220 Q 456 190 455 159 Z"
                data-aircraft-part="main-wing-right"
                data-flight-surface-scale="primary"
                data-wing-planform="asymmetric-swept"
              />
              <path
                d="M600 66 L658 34 Q665 30 672 35 L718 68 L695 75 L617 78 Z"
                data-aircraft-part="tailplane-left"
                data-flight-surface-scale="secondary"
              />
              <path
                d="M600 144 L658 176 Q665 180 672 175 L718 142 L695 135 L617 132 Z"
                data-aircraft-part="tailplane-right"
                data-flight-surface-scale="secondary"
              />
            </g>

            <path
              d={fuselagePath}
              data-aft-profile="rounded-taper"
              data-aircraft-part="fuselage"
              fill="#111820"
              stroke="rgb(255 255 255 / 0.2)"
              strokeWidth="1.75"
            />

            <g
              className={presentation.accentClass}
              clipPath={`url(#${fuselageClipId})`}
              data-current-section={map.section.location}
              data-highlight-clip="fuselage"
              data-section-end={map.section.endPercent}
              data-section-start={map.section.startPercent}
            >
              <rect
                fill="currentColor"
                height="110"
                opacity="0.28"
                width={highlightWidth}
                x={highlightStart}
                y="50"
              />
              <rect
                fill="currentColor"
                height="70"
                opacity="0.1"
                width={highlightWidth}
                x={highlightStart}
                y="70"
              />
              <line
                stroke="currentColor"
                strokeOpacity="0.72"
                x1={highlightStart}
                x2={highlightStart}
                y1="48"
                y2="162"
              />
              <line
                stroke="currentColor"
                strokeOpacity="0.72"
                x1={highlightStart + highlightWidth}
                x2={highlightStart + highlightWidth}
                y1="48"
                y2="162"
              />
            </g>

            <path
              d="M58 105 H716"
              fill="none"
              stroke="rgb(255 255 255 / 0.1)"
              strokeDasharray="5 7"
            />
            <path
              d={fuselagePath}
              fill="none"
              stroke="rgb(255 255 255 / 0.18)"
              strokeWidth="1.25"
            />
          </svg>
        </div>
      </div>
    </div>
  );
};

export { AircraftPlanOverview };
