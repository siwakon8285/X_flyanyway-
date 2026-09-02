import {
  BookingApiError,
  requestJson,
} from "@/components/booking/api/bookingApiClient";
import type {
  BitcoinSimulationOutcome,
  CreatePaymentAttemptInput,
  PaymentAttempt,
  PaymentContext,
} from "@/components/booking/payment/paymentTypes";

const paymentPath = (holdId: string) =>
  `/seat-holds/${encodeURIComponent(holdId)}/payment`;
const attemptsPath = (holdId: string) =>
  `/seat-holds/${encodeURIComponent(holdId)}/payment-attempts`;

const getPaymentContext = (holdId: string) =>
  requestJson<PaymentContext>(paymentPath(holdId));

const createPaymentAttempt = (
  holdId: string,
  input: CreatePaymentAttemptInput,
) =>
  requestJson<PaymentAttempt>(attemptsPath(holdId), {
    method: "POST",
    body: JSON.stringify(input),
  });

const simulateBitcoinPayment = (
  holdId: string,
  attemptId: string,
  outcome: BitcoinSimulationOutcome,
) =>
  requestJson<PaymentAttempt>(
    `${attemptsPath(holdId)}/${encodeURIComponent(attemptId)}/simulate`,
    { method: "POST", body: JSON.stringify({ outcome }) },
  );

export {
  BookingApiError,
  createPaymentAttempt,
  getPaymentContext,
  simulateBitcoinPayment,
};
