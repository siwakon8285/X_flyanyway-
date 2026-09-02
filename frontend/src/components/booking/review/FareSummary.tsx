import { Check } from "lucide-react";
import { useState } from "react";

import type { ReviewContext, ReviewPricingLine } from "@/components/booking/review/reviewTypes";
import { Button } from "@/components/ui/Button";
import { formatPrice } from "@/i18n/formatters";
import { useLanguage } from "@/i18n/LanguageProvider";

const priceLineKey = {
  DEMO_AIRPORT_FEE: "review.fare.airportFee",
  DEMO_BOOKING_FEE: "review.fare.bookingFee",
  DEMO_PASSENGER_TAX: "review.fare.passengerTax",
} as const;

const passengerTypeKey = {
  ADULT: "passengerInformation.passenger.adult",
  CHILD: "passengerInformation.passenger.child",
  INFANT: "passengerInformation.passenger.infant",
} as const;

const FareSummary = ({ context, remainingMilliseconds }: { context: ReviewContext; remainingMilliseconds: number }) => {
  const { locale, t } = useLanguage();
  const [namesConfirmed, setNamesConfirmed] = useState(false);
  const [conditionsConfirmed, setConditionsConfirmed] = useState(false);
  const currency = context.pricing.currencyCode;
  const display = (amount: number) => formatPrice(amount, locale, currency);
  const minutes = Math.floor(Math.max(0, remainingMilliseconds) / 60_000);
  const seconds = Math.floor((Math.max(0, remainingMilliseconds) % 60_000) / 1_000);
  const complete = namesConfirmed && conditionsConfirmed;
  const renderLine = (line: ReviewPricingLine) => <div className="flex justify-between gap-4" key={line.code}><dt>{t(priceLineKey[line.code])}</dt><dd className="shrink-0 tabular-nums">{display(line.amount.amount)}</dd></div>;
  const expiryTime = new Intl.DateTimeFormat(locale === "th" ? "th-TH" : "en-GB", { hour: "2-digit", minute: "2-digit" }).format(new Date(context.hold.expiresAt));

  return (
    <aside aria-label={t("review.fare.heading")} className="review-interactive-card rounded-surface border border-border bg-surface/90 p-6 lg:sticky lg:top-[calc(var(--header-height)+1.5rem)]" data-review-card="summary" data-review-reveal="summary">
      <h2 className="text-h3">{t("review.fare.heading")}</h2>
      <dl className="mt-6 space-y-4 text-sm">
        <div className="flex justify-between gap-4"><dt>{t("review.fare.base")}</dt><dd className="tabular-nums">{display(context.pricing.baseFare.amount.amount)}</dd></div>
        {context.pricing.baseFare.lines.map((line) => <div className="flex justify-between gap-4 pl-3 text-xs text-muted-foreground" key={line.passengerType}><dt>{t("review.fare.passengerLine", { type: t(passengerTypeKey[line.passengerType]), quantity: line.quantity })}</dt><dd className="tabular-nums">{display(line.amount.amount)}</dd></div>)}
        <div className="flex justify-between gap-4"><dt>{t("review.fare.extras")}</dt><dd className="review-value-emphasis tabular-nums" data-review-value="extras-total" key={context.pricing.extras.amount}>{display(context.pricing.extras.amount)}</dd></div>
        {context.pricing.taxes.map(renderLine)}
        {context.pricing.fees.map(renderLine)}
        <div className="flex items-end justify-between gap-4 border-t border-border pt-5"><dt className="font-semibold">{t("review.fare.grandTotal")}</dt><dd className="review-value-emphasis text-2xl font-semibold tabular-nums text-brand" data-review-value="grand-total" key={context.pricing.grandTotal.amount}>{display(context.pricing.grandTotal.amount)}</dd></div>
      </dl>
      <p className="mt-4 text-xs leading-relaxed text-muted-foreground">{t("review.fare.demoCharges")}</p>
      <div className="review-interactive-card mt-6 rounded-control border border-brand/35 bg-brand/5 p-4" data-review-card="policy"><p className="text-xs font-semibold tracking-wide text-brand">{t("review.conditions.badge")}</p><h3 className="mt-2 font-semibold">{t("review.conditions.heading")}</h3><p className="mt-2 text-sm">{t("review.conditions.nonRefundable")}</p><p className="mt-1 text-sm">{t("review.conditions.noChanges")}</p><p className="mt-3 text-xs leading-relaxed text-muted-foreground">{t("review.conditions.note")}</p></div>
      <div className="review-interactive-card mt-6 rounded-control border border-transparent p-1" data-review-card="confirmation" data-review-reveal="confirmation">
        <fieldset className="space-y-4"><legend className="sr-only">{t("review.conditions.heading")}</legend><label className="flex cursor-pointer gap-3 text-sm"><input checked={namesConfirmed} className="review-confirmation-checkbox mt-0.5 size-4 accent-brand" onChange={(event) => setNamesConfirmed(event.target.checked)} type="checkbox" />{t("review.acknowledgement.names")}</label><label className="flex cursor-pointer gap-3 text-sm"><input checked={conditionsConfirmed} className="review-confirmation-checkbox mt-0.5 size-4 accent-brand" onChange={(event) => setConditionsConfirmed(event.target.checked)} type="checkbox" />{t("review.acknowledgement.conditions")}</label></fieldset>
        {complete ? <p className="review-ready-emphasis mt-4 flex items-center gap-2 text-sm text-brand" data-review-ready="true" role="status"><Check aria-hidden="true" className="review-check-emphasis size-4" />{t("review.acknowledgement.complete")}</p> : null}
        <p aria-live="off" className="mt-6 font-semibold tabular-nums text-brand" role="timer">{t("review.hold.countdown", { time: `${minutes}:${seconds.toString().padStart(2, "0")}` })}</p>
        <p className="sr-only">{t("review.hold.expiresAt", { time: expiryTime })}</p>
        <Button aria-describedby="review-payment-note" className="review-payment-disabled mt-5 w-full" data-review-payment="disabled" disabled size="lg">{t("review.payment.action")}</Button>
        <p className="review-payment-note mt-3 text-center text-xs text-muted-foreground" id="review-payment-note">{t("review.payment.nextStep")}</p>
      </div>
    </aside>
  );
};

export { FareSummary };
