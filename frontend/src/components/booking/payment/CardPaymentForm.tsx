import { useState } from "react";

import type { CardScenario } from "@/components/booking/payment/paymentTypes";
import { validateDemoCard, type DemoCardErrors, type DemoCardFields } from "@/components/booking/payment/paymentValidation";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Label } from "@/components/ui/Label";
import { useLanguage } from "@/i18n/LanguageProvider";

const emptyFields: DemoCardFields = { cardholderName: "", cardNumber: "", cvc: "", expiry: "" };

const CardPaymentForm = ({ amount, disabled, onSubmit }: { amount: string; disabled: boolean; onSubmit: (scenario: CardScenario) => Promise<void> }) => {
  const { t } = useLanguage();
  const [fields, setFields] = useState(emptyFields);
  const [errors, setErrors] = useState<DemoCardErrors>({});
  const [scenario, setScenario] = useState<CardScenario>("SUCCESS");
  const [submitting, setSubmitting] = useState(false);
  const errorText = (field: keyof DemoCardFields) => {
    const error = errors[field];
    if (!error) return null;
    const key = error === "required" ? "payment.validation.required" : error === "expired" ? "payment.validation.expired" : field === "cardNumber" ? "payment.validation.cardNumber" : field === "cvc" ? "payment.validation.cvc" : "payment.validation.expiry";
    return <p className="mt-1 text-xs text-destructive" id={`${field}-error`} role="alert">{t(key)}</p>;
  };
  const update = (field: keyof DemoCardFields, value: string) => setFields((current) => ({ ...current, [field]: value }));
  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    const nextErrors = validateDemoCard(fields);
    setErrors(nextErrors);
    if (Object.keys(nextErrors).length) return;
    setSubmitting(true);
    try { await onSubmit(scenario); } finally { setFields(emptyFields); setSubmitting(false); }
  };
  const fieldProps = (field: keyof DemoCardFields) => ({ "aria-describedby": errors[field] ? `${field}-error` : undefined, "aria-invalid": Boolean(errors[field]) });
  return (
    <form className="payment-panel-in mt-6 rounded-surface border border-border bg-surface/80 p-5 sm:p-7" data-payment-panel="CARD" onSubmit={submit}>
      <h2 className="text-h3">{t("payment.card.heading")}</h2>
      <p className="mt-3 text-sm text-muted-foreground">{t("payment.card.helper")}</p>
      <div className="mt-6 grid gap-5 sm:grid-cols-2">
        <div className="sm:col-span-2"><Label htmlFor="cardholderName">{t("payment.card.cardholder")}</Label><Input {...fieldProps("cardholderName")} autoComplete="off" className="payment-card-field" disabled={disabled || submitting} id="cardholderName" onChange={(event) => update("cardholderName", event.target.value)} value={fields.cardholderName} />{errorText("cardholderName")}</div>
        <div className="sm:col-span-2"><Label htmlFor="cardNumber">{t("payment.card.cardNumber")}</Label><Input {...fieldProps("cardNumber")} autoComplete="off" className="payment-card-field" disabled={disabled || submitting} id="cardNumber" inputMode="numeric" onChange={(event) => update("cardNumber", event.target.value)} value={fields.cardNumber} />{errorText("cardNumber")}</div>
        <div><Label htmlFor="expiry">{t("payment.card.expiry")}</Label><Input {...fieldProps("expiry")} autoComplete="off" className="payment-card-field" disabled={disabled || submitting} id="expiry" inputMode="numeric" onChange={(event) => update("expiry", event.target.value)} placeholder="MM/YY" value={fields.expiry} />{errorText("expiry")}</div>
        <div><Label htmlFor="cvc">{t("payment.card.cvc")}</Label><Input {...fieldProps("cvc")} autoComplete="off" className="payment-card-field" disabled={disabled || submitting} id="cvc" inputMode="numeric" onChange={(event) => update("cvc", event.target.value)} value={fields.cvc} />{errorText("cvc")}</div>
        <div className="sm:col-span-2"><Label htmlFor="payment-scenario">{t("payment.card.scenario")}</Label><select className="mt-2 min-h-11 w-full rounded-control border border-border bg-surface-elevated px-3 focus-visible:ring-2 focus-visible:ring-focus" disabled={disabled || submitting} id="payment-scenario" onChange={(event) => setScenario(event.target.value as CardScenario)} value={scenario}><option value="SUCCESS">{t("payment.card.scenarioSuccess")}</option><option value="DECLINED">{t("payment.card.scenarioDeclined")}</option><option value="PROCESSING_ERROR">{t("payment.card.scenarioError")}</option></select></div>
      </div>
      <Button className="payment-submit-button mt-7 w-full" disabled={disabled} loading={submitting} size="lg" type="submit">{submitting ? t("payment.card.processing") : t("payment.card.submit", { amount })}</Button>
    </form>
  );
};

export { CardPaymentForm };
