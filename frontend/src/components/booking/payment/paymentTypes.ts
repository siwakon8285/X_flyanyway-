import type { Money } from "@/components/booking/extras/extrasTypes";
import type { ReviewJourney } from "@/components/booking/review/reviewTypes";
import type { SeatHold } from "@/components/booking/seats/seatHoldClient";

type PaymentMethod = "CARD" | "BITCOIN";
type PaymentStatus =
  | "CREATED"
  | "PROCESSING"
  | "AWAITING_PAYMENT"
  | "SUCCEEDED"
  | "FAILED"
  | "CANCELLED";
type BitcoinSimulationOutcome = "RECEIVED" | "FAILED" | "CANCELLED";

type PaymentAttempt = {
  amount: Money;
  createdAt: string;
  demoBitcoinInvoice?: {
    amountSatoshis: number;
    demoAddress: string;
    displayAmount: string;
    invoiceReference: string;
    rateThbPerBtc: number;
  };
  failure?: { code: string; message: string };
  id: string;
  paymentMethod: PaymentMethod;
  clientPaymentSession?: string;
  provider: "STRIPE" | "MOCK_BITCOIN";
  providerReference?: string;
  status: PaymentStatus;
  succeededAt?: string;
  updatedAt: string;
};

type PaymentContext = {
  attempts: PaymentAttempt[];
  hold: SeatHold;
  journey: ReviewJourney;
  methods: PaymentMethod[];
  pricing: {
    currencyCode: string;
    grandTotal: Money;
    pricedAt: string;
  };
  readyForPayment: boolean;
};

type CreatePaymentAttemptInput =
  | { method: "CARD"; preferredLocale: "EN" | "TH"; requestId: string }
  | { method: "BITCOIN"; preferredLocale: "EN" | "TH"; requestId: string };

export type {
  BitcoinSimulationOutcome,
  CreatePaymentAttemptInput,
  PaymentAttempt,
  PaymentContext,
  PaymentMethod,
  PaymentStatus,
};
