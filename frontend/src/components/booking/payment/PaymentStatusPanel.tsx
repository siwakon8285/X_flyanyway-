import Link from "next/link";
import { CheckCircle2, CircleX } from "lucide-react";

import { buildTicketHandoffHref } from "@/components/booking/passengers/passengerRoute";
import type { PaymentAttempt } from "@/components/booking/payment/paymentTypes";
import { Button, buttonVariants } from "@/components/ui/Button";
import { cn } from "@/lib/utils/cn";
import { useLanguage } from "@/i18n/LanguageProvider";

type PaymentStatusPanelProps = {
  attempt: PaymentAttempt;
  holdId?: string;
  backQuery?: string;
};

const PaymentStatusPanel = ({
  attempt,
  holdId,
  backQuery,
}: PaymentStatusPanelProps) => {
  const { t } = useLanguage();
  if (attempt.status === "SUCCEEDED") {
    const ticketHref = holdId
      ? buildTicketHandoffHref({
          attemptId: attempt.id,
          holdId,
          query: backQuery,
        })
      : null;

    return (
      <section
        aria-label={t("payment.success.title")}
        className="payment-success-in relative overflow-hidden rounded-surface border border-brand/40 bg-brand/5 p-7 text-center"
        data-payment-status={attempt.status}
        data-payment-success="true"
        role="status"
      >
        <CheckCircle2
          aria-hidden="true"
          className="payment-success-icon mx-auto size-12 text-brand"
        />
        <h2 className="mt-4 text-h3">{t("payment.success.title")}</h2>
        <p className="mx-auto mt-3 max-w-xl text-muted-foreground">
          {t("payment.success.body")}
        </p>
        <p className="payment-success-reference mt-4 break-all font-mono text-xs">
          {attempt.providerReference}
        </p>
        {ticketHref ? (
          <Link
            className={cn(buttonVariants({ size: "lg", variant: "primary" }), "mt-7")}
            href={ticketHref}
          >
            {t("payment.success.viewTicket")}
          </Link>
        ) : (
          <Button className="mt-7" disabled size="lg">
            {t("payment.success.ticket")}
          </Button>
        )}
      </section>
    );
  }

  if (!["FAILED", "CANCELLED"].includes(attempt.status))
    return (
      <p
        aria-live="polite"
        className="payment-status-in rounded-control border border-border bg-surface p-4"
        data-payment-status={attempt.status}
        role="status"
      >
        {t(
          attempt.status === "AWAITING_PAYMENT"
            ? "payment.status.awaiting"
            : "payment.status.processing",
        )}
      </p>
    );

  const key =
    attempt.failure?.code === "CARD_DECLINED"
      ? "payment.failure.declined"
      : attempt.failure?.code === "AUTHENTICATION_FAILED"
        ? "payment.failure.authentication"
        : attempt.failure?.code === "PROCESSING_ERROR"
          ? "payment.failure.processing"
          : attempt.failure?.code === "MOCK_BITCOIN_FAILED"
            ? "payment.failure.bitcoin"
            : attempt.status === "CANCELLED"
              ? "payment.failure.cancelled"
              : "payment.failure.generic";

  return (
    <section
      className="payment-failure-in rounded-control border border-destructive/45 bg-destructive/10 p-5"
      data-payment-failure="true"
      data-payment-status={attempt.status}
      role="alert"
    >
      <div className="flex gap-3">
        <CircleX
          aria-hidden="true"
          className="mt-0.5 size-5 shrink-0 text-destructive"
        />
        <div>
          <h2 className="font-semibold">
            {t(
              attempt.status === "CANCELLED"
                ? "payment.status.cancelled"
                : "payment.status.failed",
            )}
          </h2>
          <p className="mt-2 text-sm">{t(key)}</p>
          <p className="mt-2 font-mono text-xs text-muted-foreground">
            {attempt.failure?.code}
          </p>
        </div>
      </div>
    </section>
  );
};

export { PaymentStatusPanel };
