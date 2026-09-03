"use client";

import { Elements, PaymentElement, useElements, useStripe } from "@stripe/react-stripe-js";
import { loadStripe } from "@stripe/stripe-js";
import { useState } from "react";

import { Button } from "@/components/ui/Button";
import { useLanguage } from "@/i18n/LanguageProvider";

const key = process.env.NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY;
const stripePromise = key?.startsWith("pk_test_") ? loadStripe(key) : null;

function Confirm({ amount, disabled, done, confirming }: { amount: string; disabled: boolean; done: () => void; confirming: boolean }) {
  const { t } = useLanguage();
  const stripe = useStripe();
  const elements = useElements();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState(false);

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!stripe || !elements) return;

    setBusy(true);
    setError(false);
    try {
      const result = await stripe.confirmPayment({
        elements,
        redirect: "if_required",
      });
      if (result.error) setError(true);
      else done();
    } catch {
      setError(true);
    } finally {
      setBusy(false);
    }
  };

  return (
    <form className="payment-panel-in mt-6 rounded-surface border border-border bg-surface/80 p-5 sm:p-7" data-payment-panel="CARD" onSubmit={submit}>
      <h2 className="text-h3">{t("payment.method.card")}</h2>
      <div className="mt-6">
        <PaymentElement options={{ layout: "tabs" }} />
      </div>
      {error ? <p className="mt-3 text-sm text-destructive" role="alert">{t("payment.failure.generic")}</p> : null}
      {confirming ? <p aria-label={t("payment.card.confirming")} aria-live="polite" className="mt-4 text-sm text-muted-foreground" role="status">{t("payment.card.confirming")}</p> : null}
      <Button className="payment-submit-button mt-7 w-full" disabled={disabled || busy || confirming || !stripe || !elements} loading={busy} size="lg" type="submit">
        {t("payment.card.submit", { amount })}
      </Button>
    </form>
  );
}

export function CardPaymentForm({ amount, clientSecret, disabled, confirming, onStart, onConfirmed }: { amount: string; clientSecret?: string; disabled: boolean; confirming: boolean; onStart: () => Promise<void>; onConfirmed: () => void }) {
  const { t } = useLanguage();
  const panelClassName = "payment-panel-in mt-6 rounded-surface border border-border bg-surface/80 p-5 sm:p-7";

  if (!stripePromise) {
    return (
      <section className={panelClassName} data-payment-panel="CARD">
        <h2 className="text-h3">{t("payment.method.card")}</h2>
        <p className="mt-3 text-sm text-muted-foreground" role="alert">{t("payment.recovery.unavailable")}</p>
      </section>
    );
  }

  if (!clientSecret) {
    return (
      <section className={panelClassName} data-payment-panel="CARD">
        <h2 className="text-h3">{t("payment.method.card")}</h2>
        <Button className="payment-submit-button mt-7 w-full" disabled={disabled} onClick={() => void onStart()} size="lg">
          {t("payment.card.submit", { amount })}
        </Button>
      </section>
    );
  }

  return (
    <Elements options={{ clientSecret, appearance: { theme: "flat" } }} stripe={stripePromise}>
      <Confirm amount={amount} confirming={confirming} disabled={disabled} done={onConfirmed} />
    </Elements>
  );
}
