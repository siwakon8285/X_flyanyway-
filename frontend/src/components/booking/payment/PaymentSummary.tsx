import type { PaymentContext, PaymentStatus } from "@/components/booking/payment/paymentTypes";
import { formatPrice } from "@/i18n/formatters";
import { useLanguage } from "@/i18n/LanguageProvider";

const statusKey: Record<PaymentStatus, "payment.status.awaiting" | "payment.status.cancelled" | "payment.status.created" | "payment.status.failed" | "payment.status.processing" | "payment.status.succeeded"> = { CREATED: "payment.status.created", PROCESSING: "payment.status.processing", AWAITING_PAYMENT: "payment.status.awaiting", SUCCEEDED: "payment.status.succeeded", FAILED: "payment.status.failed", CANCELLED: "payment.status.cancelled" };

const PaymentSummary = ({ context, remainingMilliseconds }: { context: PaymentContext; remainingMilliseconds: number }) => {
  const { locale, t } = useLanguage();
  const latest = context.attempts[0];
  const minutes = Math.floor(Math.max(0, remainingMilliseconds) / 60_000);
  const seconds = Math.floor((Math.max(0, remainingMilliseconds) % 60_000) / 1_000);
  const expiry = new Intl.DateTimeFormat(locale === "th" ? "th-TH" : "en-GB", { hour: "2-digit", minute: "2-digit" }).format(new Date(context.hold.expiresAt));
  const succeeded = latest?.status === "SUCCEEDED";
  const totalKey = `${context.pricing.grandTotal.amount}-${succeeded}`;
  return <aside aria-label={t("payment.summary.heading")} className="rounded-surface border border-border bg-surface/90 p-6 lg:sticky lg:top-[calc(var(--header-height)+1.5rem)]" data-payment-reveal="summary"><h2 className="text-h3">{t("payment.summary.heading")}</h2><dl className="mt-6 space-y-5 text-sm"><div><dt className="text-muted-foreground">{t("payment.summary.journey")}</dt><dd className="mt-2"><span className="font-semibold">{context.journey.originCode}</span><span aria-hidden="true" className="mx-2 text-brand">→</span><span className="font-semibold">{context.journey.destinationCode}</span><span className="mt-1 block text-xs text-muted-foreground">{context.journey.flightNumber} · {context.journey.departureTime}</span></dd></div><div className="border-t border-border pt-5"><dt>{t("payment.summary.total")}</dt><dd className="payment-total-emphasis mt-2 text-2xl font-semibold tabular-nums text-brand" data-payment-total key={totalKey}>{formatPrice(context.pricing.grandTotal.amount, locale, context.pricing.currencyCode)}</dd></div>{latest ? <div><dt className="text-muted-foreground">{t("payment.summary.status")}</dt><dd className="payment-summary-status mt-1" data-payment-status={latest.status} key={latest.status}>{t(statusKey[latest.status])}</dd></div> : null}</dl>{!succeeded ? <><p aria-live="off" className="mt-6 font-semibold tabular-nums text-brand" role="timer">{t("payment.summary.hold", { time: `${minutes}:${seconds.toString().padStart(2, "0")}` })}</p><p className="sr-only">{t("payment.summary.expiresAt", { time: expiry })}</p></> : null}</aside>;
};

export { PaymentSummary };
