import {
  createPaymentAttempt,
  getPaymentContext,
  simulateBitcoinPayment,
} from "@/components/booking/payment/paymentClient";
import type { PaymentAttempt, PaymentContext } from "@/components/booking/payment/paymentTypes";

const context = { readyForPayment: true } as PaymentContext;
const attempt = { id: "attempt-1", status: "SUCCEEDED" } as PaymentAttempt;

describe("payment client", () => {
  const originalFetch = global.fetch;

  afterEach(() => {
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: originalFetch,
      writable: true,
    });
  });

  it("loads only the authorized hold payment resource", async () => {
    const fetchMock = jest.fn().mockResolvedValue({ json: async () => context, ok: true, status: 200 });
    Object.defineProperty(global, "fetch", { configurable: true, value: fetchMock, writable: true });

    await expect(getPaymentContext("hold-123")).resolves.toEqual(context);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:8080/api/v1/seat-holds/hold-123/payment",
      expect.objectContaining({ cache: "no-store", credentials: "include" }),
    );
  });

  it("submits only a safe Card scenario and request identifier", async () => {
    const fetchMock = jest.fn().mockResolvedValue({ json: async () => attempt, ok: true, status: 201 });
    Object.defineProperty(global, "fetch", { configurable: true, value: fetchMock, writable: true });

    await createPaymentAttempt("hold-123", {
      method: "CARD",
      requestId: "request-1",
      scenario: "DECLINED",
    });

    const init = fetchMock.mock.calls[0][1] as RequestInit;
    expect(JSON.parse(String(init.body))).toEqual({
      method: "CARD",
      requestId: "request-1",
      scenario: "DECLINED",
    });
    expect(String(init.body)).not.toMatch(/card(number|holder)|cvc|expiry|amount|currency/i);
  });

  it("creates and simulates Bitcoin with no wallet or amount input", async () => {
    const fetchMock = jest.fn().mockResolvedValue({ json: async () => attempt, ok: true, status: 200 });
    Object.defineProperty(global, "fetch", { configurable: true, value: fetchMock, writable: true });

    await createPaymentAttempt("hold-123", { method: "BITCOIN", requestId: "request-2" });
    await simulateBitcoinPayment("hold-123", "attempt-1", "RECEIVED");

    expect(JSON.parse(String(fetchMock.mock.calls[0][1].body))).toEqual({
      method: "BITCOIN",
      requestId: "request-2",
    });
    expect(JSON.parse(String(fetchMock.mock.calls[1][1].body))).toEqual({ outcome: "RECEIVED" });
  });
});
