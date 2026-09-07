import { Check } from "lucide-react";

import { cabinLabelKeys } from "@/components/booking/cabin/cabinPresentation";
import { mealServiceKeys } from "@/components/booking/extras/extrasPresentation";
import type { ReviewContext } from "@/components/booking/review/reviewTypes";
import { formatDate, formatDuration } from "@/i18n/formatters";
import { useLanguage } from "@/i18n/LanguageProvider";

const passengerTypeKey = {
  ADULT: "passengerInformation.passenger.adult",
  CHILD: "passengerInformation.passenger.child",
  INFANT: "passengerInformation.passenger.infant",
} as const;

const ReviewDetails = ({ context }: { context: ReviewContext }) => {
  const { locale, t } = useLanguage();
  return (
    <div className="min-w-0 space-y-6">
      <section className="review-interactive-card rounded-surface border border-border bg-surface/75 p-6 sm:p-8" aria-labelledby="review-journey" data-review-card="journey">
        <h2 className="text-h3" id="review-journey">{t("review.section.journey")}</h2>
        <div className="mt-6 flex items-end justify-between gap-5 border-b border-border pb-6">
          <div><p className="text-3xl font-semibold">{context.journey.originCode}</p><p className="mt-2 text-xl tabular-nums">{context.journey.departureTime}</p></div>
          <div className="text-center text-sm text-muted-foreground"><p>{formatDuration(context.journey.durationMinutes, locale)}</p><p className="mt-1">{t(context.journey.stops === "DIRECT" ? "review.label.direct" : "review.label.oneStop")}</p></div>
          <div className="text-right"><p className="text-3xl font-semibold">{context.journey.destinationCode}</p><p className="mt-2 text-xl tabular-nums">{context.journey.arrivalTime}{context.journey.arrivalDayOffset ? <span className="ml-1 text-xs text-brand">{t("review.label.nextDay")}</span> : null}</p></div>
        </div>
        <p className="mt-3 text-xs text-muted-foreground">{t("review.label.localTimes")}</p>
        <dl className="mt-6 grid gap-5 sm:grid-cols-2">
          <div><dt className="text-caption">{t("review.label.date")}</dt><dd className="mt-2">{formatDate(context.hold.departureDate, locale)}</dd></div>
          <div><dt className="text-caption">{t("review.label.aircraft")}</dt><dd className="mt-2">{context.journey.aircraftCode}</dd></div>
          <div><dt className="text-caption">{t("review.label.cabin")}</dt><dd className="mt-2">{t(cabinLabelKeys[context.hold.cabin])}</dd></div>
          <div><dt className="text-caption">{t("review.label.duration")}</dt><dd className="mt-2">{formatDuration(context.journey.durationMinutes, locale)}</dd></div>
        </dl>
      </section>

      <section className="review-interactive-card rounded-surface border border-border bg-surface/75 p-6 sm:p-8" aria-labelledby="review-passengers" data-review-card="passengers">
        <h2 className="text-h3" id="review-passengers">{t("review.section.passengers")}</h2>
        <ul className="mt-6 divide-y divide-border">
          {context.passengers.map((passenger) => (
            <li className="py-5 first:pt-0 last:pb-0" key={passenger.ordinal}>
              <div className="flex flex-wrap items-start justify-between gap-4">
                <div><p className="font-semibold">{passenger.displayName}</p><p className="mt-1 text-sm text-muted-foreground">{t("passengerInformation.passenger.label", { ordinal: passenger.ordinal, type: t(passengerTypeKey[passenger.passengerType]) })}</p></div>
                <div className="text-sm text-muted-foreground"><p>{t("review.label.nationality")}: {passenger.nationalityCode}</p>{passenger.travelDocumentComplete ? <p className="mt-2 flex items-center gap-2 text-brand"><Check aria-hidden="true" className="size-4" />{t("review.label.documentComplete")}</p> : null}</div>
              </div>
              {passenger.passengerType === "INFANT" ? <p className="mt-3 text-sm text-muted-foreground">{t("review.label.infantSeat")}</p> : null}
            </li>
          ))}
        </ul>
      </section>

      <section className="review-interactive-card rounded-surface border border-border bg-surface/75 p-6 sm:p-8" aria-labelledby="review-seats" data-review-card="seats">
        <h2 className="text-h3" id="review-seats">{t("review.section.seats")}</h2>
        <div className="mt-5 flex flex-wrap gap-3">{context.seats.map(({ seatNumber }) => <span className="rounded-control border border-brand/45 bg-brand/10 px-4 py-2 font-semibold text-brand" key={seatNumber}>{seatNumber}</span>)}</div>
      </section>

      <section className="review-interactive-card rounded-surface border border-border bg-surface/75 p-6 sm:p-8" aria-labelledby="review-included" data-review-card="included">
        <h2 className="text-h3" id="review-included">{t("review.section.included")}</h2>
        <ul className="mt-5 grid gap-3 text-sm sm:grid-cols-2"><li>{t("review.label.cabinBaggage", { kg: context.includedBenefits.allowances.cabinBaggageKg })}</li><li>{t("review.label.checkedBaggage", { kg: context.includedBenefits.allowances.checkedBaggageKg })}</li><li>{t("review.label.seatIncluded")}</li><li>{t(mealServiceKeys[context.includedBenefits.benefits.mealService])}</li></ul>
      </section>
    </div>
  );
};

export { ReviewDetails };
