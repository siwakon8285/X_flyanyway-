import { Bitcoin } from "lucide-react";

import type { BitcoinSimulationOutcome, PaymentAttempt } from "@/components/booking/payment/paymentTypes";
import { Button } from "@/components/ui/Button";
import { useLanguage } from "@/i18n/LanguageProvider";

const BitcoinPaymentPanel = ({ attempt, busy, onCreate, onSimulate, sourceAmount }: { attempt: PaymentAttempt | null; busy: boolean; onCreate: () => Promise<void>; onSimulate: (outcome: BitcoinSimulationOutcome) => Promise<void>; sourceAmount: string }) => {
  const { t } = useLanguage();
  const invoice = attempt?.status === "AWAITING_PAYMENT" ? attempt.demoBitcoinInvoice : undefined;
  return (
    <section className="payment-panel-in mt-6 rounded-surface border border-border bg-surface/80 p-5 sm:p-7" data-payment-panel="BITCOIN">
      <div className="flex items-center gap-3"><Bitcoin aria-hidden="true" className="size-7 text-brand" /><h2 className="text-h3">{t("payment.bitcoin.heading")}</h2></div>
      {!invoice ? <Button className="payment-submit-button mt-7 w-full" disabled={busy} loading={busy} onClick={() => void onCreate()} size="lg">{t("payment.bitcoin.create")}</Button> : <div className="payment-invoice mt-7 space-y-5" data-payment-invoice><p className="rounded-control border border-brand/30 bg-brand/5 p-4 text-sm" data-payment-invoice-item>{t("payment.bitcoin.awaiting")}</p><dl className="grid gap-4 text-sm" data-payment-invoice-item><div><dt className="text-muted-foreground">{t("payment.bitcoin.amount")}</dt><dd className="mt-1 text-xl font-semibold text-brand">{invoice.displayAmount} BTC</dd></div><div><dt className="text-muted-foreground">{t("payment.bitcoin.source")}</dt><dd className="mt-1">{sourceAmount}</dd></div><div><dt className="text-muted-foreground">{t("payment.bitcoin.address")}</dt><dd className="mt-1 select-text break-all font-mono text-xs">{invoice.demoAddress}</dd></div><div><dt className="text-muted-foreground">{t("payment.bitcoin.reference")}</dt><dd className="mt-1 select-text break-all font-mono text-xs">{invoice.invoiceReference}</dd></div></dl><p className="text-xs text-muted-foreground" data-payment-invoice-item>{t("payment.bitcoin.rate")}</p><div className="grid gap-3 sm:grid-cols-2" data-payment-invoice-item><Button disabled={busy} loading={busy} onClick={() => void onSimulate("RECEIVED")}>{t("payment.bitcoin.received")}</Button><Button disabled={busy} onClick={() => void onSimulate("FAILED")} variant="outline">{t("payment.bitcoin.failed")}</Button><Button className="sm:col-span-2" disabled={busy} onClick={() => void onSimulate("CANCELLED")} variant="ghost">{t("payment.bitcoin.cancel")}</Button></div></div>}
    </section>
  );
};

export { BitcoinPaymentPanel };
