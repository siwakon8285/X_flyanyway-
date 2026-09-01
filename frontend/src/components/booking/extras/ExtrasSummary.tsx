import { Check } from "lucide-react";

import { productLabelKey } from "@/components/booking/extras/extrasPresentation";
import type {
  ExtraProduct,
  ExtraSelectionInput,
  ExtrasContext,
} from "@/components/booking/extras/extrasTypes";
import { FLIGHT_RESULT_FIXTURES } from "@/components/booking/results/flightResultFixtures";
import { formatDate, formatPrice } from "@/i18n/formatters";
import { useLanguage } from "@/i18n/LanguageProvider";

const ExtrasSummary = ({
  context,
  remainingMilliseconds,
  selections,
  totalAmount,
}: {
  context: ExtrasContext;
  remainingMilliseconds: number;
  selections: ExtraSelectionInput[];
  totalAmount: number;
}) => {
  const { locale, t } = useLanguage();
  const flight = FLIGHT_RESULT_FIXTURES.find((item) => item.id === context.hold.flightId);
  const minutes = Math.floor(Math.max(0, remainingMilliseconds) / 60_000);
  const seconds = Math.floor((Math.max(0, remainingMilliseconds) % 60_000) / 1_000);
  const productFor = (selection: ExtraSelectionInput): ExtraProduct | undefined =>
    context.catalog.products.find((product) => product.code === selection.productCode);
  const paidSelections = selections.filter(
    (selection) => (productFor(selection)?.unitPrice.amount ?? 0) > 0,
  );
  const paidSelectionKey = paidSelections
    .map((selection) => `${selection.passengerOrdinal}-${selection.productCode}`)
    .join("|");

  return (
    <aside
      className="rounded-surface border border-border bg-surface/80 p-6 lg:sticky lg:top-[calc(var(--header-height)+1.5rem)]"
      data-extras-reveal
    >
      <p className="text-caption text-brand">{t("travelExtras.summary.heading")}</p>
      <p className="mt-4 text-lg font-semibold">
        {flight?.flightNumber ?? context.hold.flightId.toUpperCase()}
      </p>
      {flight ? (
        <p className="mt-1 text-sm text-muted-foreground">
          {flight.originCode} → {flight.destinationCode}
        </p>
      ) : null}
      <p className="mt-4 text-sm text-muted-foreground">
        {formatDate(context.hold.departureDate, locale)}
      </p>
      <div className="my-5 h-px bg-border" />
      <p className="text-caption">{t("travelExtras.summary.seats")}</p>
      <div className="mt-3 flex flex-wrap gap-2">
        {context.hold.seats.map((seat) => (
          <span className="rounded-control border border-brand/45 bg-brand/10 px-3 py-2 text-sm font-semibold text-brand" key={seat}>
            {seat}
          </span>
        ))}
      </div>
      <div className="my-5 h-px bg-border" />
      {paidSelections.length > 0 ? (
        <ul
          className="space-y-3 text-sm motion-safe:animate-[extras-list-change_240ms_ease-out_1] motion-reduce:animate-none"
          key={paidSelectionKey}
        >
          {paidSelections.map((selection) => {
            const product = productFor(selection);
            if (!product) return null;
            return (
              <li className="flex justify-between gap-4" key={`${selection.passengerOrdinal}-${selection.productCode}`}>
                <span>
                  {t("travelExtras.summary.passengerLine", {
                    ordinal: selection.passengerOrdinal,
                    product: t(productLabelKey(product.code)),
                  })}
                </span>
                <span className="shrink-0">{formatPrice(product.unitPrice.amount, locale)}</span>
              </li>
            );
          })}
        </ul>
      ) : (
        <p
          className="text-sm text-muted-foreground motion-safe:animate-[extras-list-change_240ms_ease-out_1] motion-reduce:animate-none"
          key="no-paid-extras"
        >
          {t("travelExtras.summary.noPaidExtras")}
        </p>
      )}
      <div className="my-5 h-px bg-border" />
      <div
        aria-label={t("travelExtras.summary.totalAria", {
          total: formatPrice(totalAmount, locale),
        })}
        className="flex items-end justify-between gap-4"
        aria-live="polite"
        role="status"
      >
        <span className="font-medium">{t("travelExtras.summary.total")}</span>
        <span
          className="text-xl font-semibold tabular-nums text-brand motion-safe:animate-[extras-value-change_240ms_ease-out_1] motion-reduce:animate-none"
          key={totalAmount}
        >
          {formatPrice(totalAmount, locale)}
        </span>
      </div>
      <p className="mt-6 text-sm font-semibold tabular-nums text-brand" aria-live="polite">
        {t("seatMap.holdCountdown", {
          time: `${minutes}:${seconds.toString().padStart(2, "0")}`,
        })}
      </p>
      {context.readyToContinue ? (
        <p className="mt-4 flex items-center gap-2 text-sm text-brand">
          <Check aria-hidden="true" className="size-4" />
          {t("travelExtras.summary.serverSaved")}
        </p>
      ) : null}
    </aside>
  );
};

export { ExtrasSummary };
