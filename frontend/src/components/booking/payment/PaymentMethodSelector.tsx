import { Bitcoin, CreditCard } from "lucide-react";

import type { PaymentMethod } from "@/components/booking/payment/paymentTypes";
import { useLanguage } from "@/i18n/LanguageProvider";

const PaymentMethodSelector = ({
  disabled,
  onChange,
  value,
}: {
  disabled: boolean;
  onChange: (method: PaymentMethod) => void;
  value: PaymentMethod;
}) => {
  const { t } = useLanguage();
  return (
    <fieldset>
      <legend className="text-sm font-semibold">{t("payment.method.legend")}</legend>
      <div className="mt-4 grid gap-3 sm:grid-cols-2">
        {([
          ["CARD", CreditCard, "payment.method.card", "payment.method.cardHint"],
          ["BITCOIN", Bitcoin, "payment.method.bitcoin", "payment.method.bitcoinHint"],
        ] as const).map(([method, Icon, label, hint]) => (
          <label className="payment-method-card flex min-h-20 cursor-pointer items-center gap-4 rounded-control border border-border bg-surface-elevated p-4 has-[:checked]:border-brand has-[:checked]:bg-brand/5" data-payment-method={method} data-selected={value === method} key={method}>
            <input checked={value === method} className="size-4 accent-brand" disabled={disabled} name="payment-method" onChange={() => onChange(method)} type="radio" value={method} />
            <Icon aria-hidden="true" className="size-6 text-brand" />
            <span><span className="block font-medium">{t(label)}</span><span className="mt-1 block text-xs text-muted-foreground">{t(hint)}</span></span>
          </label>
        ))}
      </div>
    </fieldset>
  );
};

export { PaymentMethodSelector };
