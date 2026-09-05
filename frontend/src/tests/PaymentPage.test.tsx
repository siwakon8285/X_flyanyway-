import { act, fireEvent, screen, within } from "@testing-library/react";

import { PaymentPage } from "@/components/booking/payment/PaymentPage";
import type { PaymentAttempt, PaymentContext } from "@/components/booking/payment/paymentTypes";
import { render } from "@/tests/renderWithLanguage";

const mockConfirmPayment = jest.fn();

jest.mock("@stripe/stripe-js", () => {
  process.env.NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY = "pk_test_payment_page";
  return { loadStripe: jest.fn(() => Promise.resolve({})) };
});

jest.mock("@stripe/react-stripe-js", () => ({
  Elements: ({ children }: { children: React.ReactNode }) => children,
  PaymentElement: () => <div aria-label="Stripe Payment Element" />,
  useElements: () => ({}),
  useStripe: () => ({ confirmPayment: mockConfirmPayment }),
}));

const baseAttempt: PaymentAttempt = {
  amount: { amount: 49_300, currencyCode: "THB" },
  createdAt: "2099-05-01T09:55:00Z",
  id: "attempt-1",
  paymentMethod: "CARD",
  provider: "STRIPE",
  providerReference: "XFCARD-DEMO",
  status: "SUCCEEDED",
  succeededAt: "2099-05-01T09:56:00Z",
  updatedAt: "2099-05-01T09:56:00Z",
};

const context: PaymentContext = {
  attempts: [],
  hold: {
    cabin: "economy",
    departureDate: "2099-05-10",
    expiresAt: "2099-05-01T10:10:00Z",
    flightId: "xf-201",
    id: "hold-123",
    passengers: { adults: 1, children: 0, infants: 0 },
    seats: ["20A"],
    serverTime: "2099-05-01T10:00:00Z",
  },
  journey: {
    aircraftCode: "Airbus A350-900",
    arrivalDayOffset: 0,
    arrivalTime: "16:40",
    departureTime: "09:15",
    destinationCode: "LHR",
    durationMinutes: 685,
    flightNumber: "XF 201",
    originCode: "BKK",
    stops: "DIRECT",
  },
  methods: ["CARD", "BITCOIN"],
  pricing: {
    currencyCode: "THB",
    grandTotal: { amount: 49_300, currencyCode: "THB" },
    pricedAt: "2099-05-01T09:54:00Z",
  },
  readyForPayment: true,
};

const setFetchResponses = (...responses: Array<{ body: unknown; ok?: boolean; status?: number }>) => {
  const fetchMock = jest.fn();
  responses.forEach(({ body, ok = true, status = 200 }) => {
    fetchMock.mockResolvedValueOnce({ json: async () => body, ok, status });
  });
  Object.defineProperty(global, "fetch", { configurable: true, value: fetchMock, writable: true });
  return fetchMock;
};

describe("Payment page", () => {
  const originalFetch = global.fetch;
  const originalMatchMedia = window.matchMedia;

  afterEach(() => {
    mockConfirmPayment.mockReset();
    jest.useRealTimers();
    Object.defineProperty(global, "fetch", { configurable: true, value: originalFetch, writable: true });
    window.matchMedia = originalMatchMedia;
  });

  it("loads and presents the server-authoritative amount in a responsive summary", async () => {
    setFetchResponses({ body: context });
    const { container } = render(<PaymentPage backQuery="flightId=xf-201" holdId="hold-123" />);

    expect(screen.getByRole("status", { name: "Loading secure demo payment" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Secure demo payment" })).toBeInTheDocument();
    expect(await screen.findByRole("radio", { name: /Credit or Debit Card/i })).toBeChecked();
    expect(screen.getByRole("radio", { name: /Bitcoin/i })).not.toBeChecked();
    const summary = screen.getByRole("complementary", { name: "Payment summary" });
    expect(within(summary).getByText("THB 49,300")).toBeInTheDocument();
    expect(within(summary).getByText("BKK")).toBeInTheDocument();
    expect(container.querySelector("[data-payment-layout]")).toHaveClass("min-w-0");
    expect(
      Array.from(container.querySelectorAll("[data-payment-reveal]")).map(
        (element) => element.getAttribute("data-payment-reveal"),
      ),
    ).toEqual(["navigation", "heading", "notice", "checkout", "summary"]);
    expect(screen.getByRole("radio", { name: /Credit or Debit Card/i }).closest("[data-payment-method]"))
      .toHaveAttribute("data-selected", "true");
    expect(screen.getByRole("radio", { name: /Bitcoin/i }).closest("[data-payment-method]"))
      .toHaveAttribute("data-selected", "false");
    expect(container.querySelector('[data-payment-panel="CARD"]')).toBeInTheDocument();
  });

  it("starts Stripe Card payment without rendering raw card fields or a mock scenario", async () => {
    const stripeAttempt: PaymentAttempt = {
      ...baseAttempt,
      clientPaymentSession: "pi_test_secret_123",
      providerReference: "pi_test_123",
      status: "CREATED",
      succeededAt: undefined,
    };
    const fetchMock = setFetchResponses(
      { body: context },
      { body: stripeAttempt },
    );
    render(<PaymentPage backQuery="flightId=xf-201" holdId="hold-123" />);
    await screen.findByRole("button", { name: "Pay THB 49,300 in demo" });

    expect(screen.queryByLabelText("Cardholder name")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Card number")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Expiry")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("CVC")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Test payment scenario")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Pay THB 49,300 in demo" }));

    expect(await screen.findByLabelText("Stripe Payment Element")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Pay THB 49,300 in demo" })).toBeEnabled();
    expect(fetchMock).toHaveBeenLastCalledWith(
      "http://localhost:8080/api/v1/seat-holds/hold-123/payment-attempts",
      expect.objectContaining({
        body: expect.stringContaining('"preferredLocale":"EN"'),
      }),
    );
    expect(String((fetchMock.mock.calls[1]?.[1] as RequestInit).body)).not.toContain("scenario");
  });

  it("snapshots the active Thai locale through the provider-neutral payment request", async () => {
    const stripeAttempt: PaymentAttempt = {
      ...baseAttempt,
      clientPaymentSession: "pi_test_secret_123",
      status: "CREATED",
      succeededAt: undefined,
    };
    const fetchMock = setFetchResponses({ body: context }, { body: stripeAttempt });
    render(<PaymentPage backQuery="flightId=xf-201" holdId="hold-123" />, { locale: "th" });

    fireEvent.click(await screen.findByRole("button", { name: "ชำระ THB 49,300 ในโหมดตัวอย่าง" }));

    const request = fetchMock.mock.calls[1]?.[1] as RequestInit;
    expect(JSON.parse(String(request.body))).toEqual(expect.objectContaining({
      method: "CARD",
      preferredLocale: "TH",
    }));
  });

  it("waits for authoritative payment context after Stripe confirmation", async () => {
    jest.useFakeTimers();
    const stripeAttempt: PaymentAttempt = {
      ...baseAttempt,
      clientPaymentSession: "pi_test_secret_123",
      providerReference: "pi_test_123",
      status: "AWAITING_PAYMENT",
      succeededAt: undefined,
    };
    const authoritativeSuccess = { ...context, attempts: [{ ...stripeAttempt, status: "SUCCEEDED", succeededAt: "2099-05-01T10:01:00Z" }], readyForPayment: false };
    const fetchMock = setFetchResponses({ body: context }, { body: stripeAttempt }, { body: authoritativeSuccess });
    mockConfirmPayment.mockResolvedValue({});
    render(<PaymentPage backQuery="flightId=xf-201" holdId="hold-123" />);
    fireEvent.click(await screen.findByRole("button", { name: "Pay THB 49,300 in demo" }));
    await screen.findByLabelText("Stripe Payment Element");

    await act(async () => {
      fireEvent.submit(screen.getByRole("button", { name: "Pay THB 49,300 in demo" }).closest("form") as HTMLFormElement);
    });
    expect(await screen.findByRole("status", { name: "Confirming payment" })).toBeInTheDocument();
    expect(screen.queryByRole("status", { name: "Demo payment successful" })).not.toBeInTheDocument();

    await act(async () => { jest.advanceTimersByTime(2_000); });
    expect(await screen.findByRole("status", { name: "Demo payment successful" })).toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:8080/api/v1/seat-holds/hold-123/payment",
      expect.objectContaining({ cache: "no-store", credentials: "include" }),
    );
  });

  it("creates a Bitcoin invoice and simulates payment received", async () => {
    const bitcoin: PaymentAttempt = {
      ...baseAttempt,
      demoBitcoinInvoice: {
        amountSatoshis: 2_465_000,
        demoAddress: "DEMO-ONLY-NOT-A-BITCOIN-ADDRESS-ABC123",
        displayAmount: "0.02465000",
        invoiceReference: "XFBTC-ABC123",
        rateThbPerBtc: 2_000_000,
      },
      paymentMethod: "BITCOIN",
      provider: "MOCK_BITCOIN",
      status: "AWAITING_PAYMENT",
      succeededAt: undefined,
    };
    setFetchResponses({ body: context }, { body: bitcoin }, { body: { ...bitcoin, status: "SUCCEEDED" } });
    render(<PaymentPage backQuery="flightId=xf-201" holdId="hold-123" />);
    await screen.findByRole("radio", { name: /Bitcoin/i });

    fireEvent.click(screen.getByRole("radio", { name: /Bitcoin/i }));
    expect(screen.getByRole("radio", { name: /Bitcoin/i }).closest("[data-payment-method]"))
      .toHaveAttribute("data-selected", "true");
    expect(document.querySelector('[data-payment-panel="BITCOIN"]')).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Create demo Bitcoin invoice" }));
    expect(await screen.findByText("0.02465000 BTC")).toBeInTheDocument();
    expect(screen.getByText("DEMO-ONLY-NOT-A-BITCOIN-ADDRESS-ABC123")).toBeInTheDocument();
    expect(document.querySelector("[data-payment-invoice]")).toBeInTheDocument();
    expect(document.querySelector('[data-payment-status="AWAITING_PAYMENT"]')).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Simulate payment received" }));

    expect(await screen.findByRole("status", { name: "Demo payment successful" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "View Ticket" })).toHaveAttribute(
      "href",
      expect.stringContaining("/booking/ticket"),
    );
  });

  it("shows recoverable Bitcoin failure and cancellation states", async () => {
    const bitcoin: PaymentAttempt = {
      ...baseAttempt,
      demoBitcoinInvoice: {
        amountSatoshis: 2_465_000,
        demoAddress: "DEMO-ONLY-NOT-A-BITCOIN-ADDRESS-ABC123",
        displayAmount: "0.02465000",
        invoiceReference: "XFBTC-ABC123",
        rateThbPerBtc: 2_000_000,
      },
      paymentMethod: "BITCOIN",
      provider: "MOCK_BITCOIN",
      status: "AWAITING_PAYMENT",
      succeededAt: undefined,
    };
    const failed = { ...bitcoin, failure: { code: "MOCK_BITCOIN_FAILED", message: "failed" }, status: "FAILED" as const };
    setFetchResponses({ body: context }, { body: bitcoin }, { body: failed });
    render(<PaymentPage backQuery="flightId=xf-201" holdId="hold-123" />);
    await screen.findByRole("radio", { name: /Bitcoin/i });

    fireEvent.click(screen.getByRole("radio", { name: /Bitcoin/i }));
    fireEvent.click(screen.getByRole("button", { name: "Create demo Bitcoin invoice" }));
    await screen.findByText("0.02465000 BTC");
    fireEvent.click(screen.getByRole("button", { name: "Simulate payment failed" }));

    const failure = await screen.findByRole("alert");
    expect(failure).toHaveTextContent("demo Bitcoin payment failed");
    expect(failure).toHaveAttribute("data-payment-failure", "true");
    expect(screen.getByRole("button", { name: "Create demo Bitcoin invoice" })).toBeEnabled();
  });

  it("renders an existing success without offering another payment", async () => {
    setFetchResponses({ body: { ...context, attempts: [baseAttempt], readyForPayment: false } });
    render(<PaymentPage backQuery="flightId=xf-201" holdId="hold-123" />);

    expect(await screen.findByRole("status", { name: "Demo payment successful" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Pay THB/i })).not.toBeInTheDocument();
  });

  it("blocks payment when the hold expires and provides Seat Selection recovery", async () => {
    jest.useFakeTimers();
    setFetchResponses({
      body: {
        ...context,
        hold: { ...context.hold, expiresAt: "2099-05-01T10:00:01Z" },
      },
    });
    render(<PaymentPage backQuery="flightId=xf-201" holdId="hold-123" />);
    await screen.findByRole("button", { name: "Pay THB 49,300 in demo" });

    act(() => { jest.advanceTimersByTime(1_000); });

    expect(screen.getByRole("alert")).toHaveTextContent("expired before payment completed");
    expect(screen.getByRole("link", { name: "Return to Seat Selection" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Pay THB/i })).not.toBeInTheDocument();
  });

  it("translates customer-facing content to Thai while preserving identifiers", async () => {
    setFetchResponses({ body: context });
    const { container } = render(<PaymentPage backQuery="flightId=xf-201" holdId="hold-123" />, { locale: "th" });

    expect(await screen.findByRole("heading", { name: "การชำระเงินตัวอย่างที่ปลอดภัย" })).toBeInTheDocument();
    expect(await screen.findByText("THB 49,300")).toBeInTheDocument();
    expect(screen.getByText(/XF 201/)).toBeInTheDocument();
    expect(within(container.querySelector('[data-payment-panel="CARD"]') as HTMLElement)
      .getByRole("heading", { name: "บัตรเครดิตหรือเดบิต" })).toBeInTheDocument();
  });

  it("exposes the reduced-motion state without changing payment content", async () => {
    window.matchMedia = jest.fn().mockReturnValue({
      addEventListener: jest.fn(),
      addListener: jest.fn(),
      dispatchEvent: jest.fn(),
      matches: true,
      media: "(prefers-reduced-motion: reduce)",
      onchange: null,
      removeEventListener: jest.fn(),
      removeListener: jest.fn(),
    });
    setFetchResponses({ body: context });
    const { container } = render(<PaymentPage backQuery="flightId=xf-201" holdId="hold-123" />);

    expect(await screen.findByText("THB 49,300")).toBeInTheDocument();
    expect(container.querySelector("main")).toHaveAttribute("data-reduced-motion", "true");
  });
});
