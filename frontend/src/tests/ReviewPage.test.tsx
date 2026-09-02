import { act, fireEvent, screen, within } from "@testing-library/react";

import { ReviewPage } from "@/components/booking/review/ReviewPage";
import type { ReviewContext } from "@/components/booking/review/reviewTypes";
import { render } from "@/tests/renderWithLanguage";

const money = (amount: number) => ({ amount, currencyCode: "THB" });

const context: ReviewContext = {
  fareConditions: {
    changesAllowed: false,
    code: "DEMO_FIXTURE_NONREFUNDABLE_NO_CHANGES",
    fixture: true,
    refundable: false,
  },
  hold: {
    cabin: "economy",
    departureDate: "2030-05-10",
    expiresAt: "2099-05-01T10:10:00Z",
    flightId: "xf-201",
    id: "hold-123",
    passengers: { adults: 1, children: 1, infants: 1 },
    seats: ["20A", "20B"],
    serverTime: "2099-05-01T10:00:00Z",
  },
  includedBenefits: {
    allowances: {
      appliesToPassengerTypes: ["ADULT", "CHILD"],
      cabinBaggageKg: 7,
      checkedBaggageKg: 20,
    },
    benefits: { mealService: "STANDARD", seatSelectionIncluded: true },
  },
  journey: {
    aircraftCode: "Airbus A350-900",
    arrivalDayOffset: 0,
    arrivalTime: "16:40",
    departureTime: "09:15",
    destinationCode: "LHR",
    durationMinutes: 805,
    flightNumber: "XF 201",
    originCode: "BKK",
    stops: "DIRECT",
  },
  passengers: [
    {
      displayName: "MS Nara Suri",
      extras: [
        {
          category: "BAGGAGE",
          lineTotal: money(2800),
          passengerOrdinal: 1,
          productCode: "BAG_20KG",
          quantity: 1,
          unitPrice: money(2800),
        },
        {
          category: "MEAL",
          lineTotal: money(0),
          passengerOrdinal: 1,
          productCode: "MEAL_VEGETARIAN",
          quantity: 1,
          unitPrice: money(0),
        },
      ],
      nationalityCode: "TH",
      ordinal: 1,
      passengerType: "ADULT",
      title: "MS",
      travelDocumentComplete: true,
    },
    {
      displayName: "MS Mali Suri",
      extras: [],
      nationalityCode: "TH",
      ordinal: 2,
      passengerType: "CHILD",
      title: "MS",
      travelDocumentComplete: true,
    },
    {
      displayName: "MS Dara Suri",
      extras: [],
      nationalityCode: "TH",
      ordinal: 3,
      passengerType: "INFANT",
      title: "MS",
      travelDocumentComplete: true,
    },
  ],
  pricing: {
    baseFare: {
      amount: money(43800),
      lines: [
        { amount: money(21900), passengerType: "ADULT", quantity: 1, unitAmount: money(21900) },
        { amount: money(21900), passengerType: "CHILD", quantity: 1, unitAmount: money(21900) },
        { amount: money(0), passengerType: "INFANT", quantity: 1, unitAmount: money(0) },
      ],
    },
    currencyCode: "THB",
    extras: money(2800),
    fees: [
      { amount: money(1000), code: "DEMO_AIRPORT_FEE", quantity: 2, unitAmount: money(500) },
      { amount: money(300), code: "DEMO_BOOKING_FEE", quantity: 1, unitAmount: money(300) },
    ],
    grandTotal: money(49300),
    pricedAt: "2099-05-01T10:00:01Z",
    taxes: [
      { amount: money(1400), code: "DEMO_PASSENGER_TAX", quantity: 2, unitAmount: money(700) },
    ],
  },
  readyForPayment: true,
  seats: [{ seatNumber: "20A" }, { seatNumber: "20B" }],
};

const setFetch = (body: unknown, ok = true, status = 200) => {
  Object.defineProperty(global, "fetch", {
    configurable: true,
    value: jest.fn().mockResolvedValue({ json: async () => body, ok, status }),
    writable: true,
  });
};

describe("ReviewPage", () => {
  const originalFetch = global.fetch;

  afterEach(() => {
    jest.useRealTimers();
    if (originalFetch) {
      Object.defineProperty(global, "fetch", {
        configurable: true,
        value: originalFetch,
        writable: true,
      });
    } else {
      Reflect.deleteProperty(global, "fetch");
    }
  });

  it("exposes a localized loading state while Review is requested", () => {
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: jest.fn().mockReturnValue(new Promise(() => undefined)),
      writable: true,
    });
    render(<ReviewPage backQuery="flightId=xf-201" holdId="hold-123" />);

    expect(screen.getByRole("status", { name: "Loading booking review" })).toBeInTheDocument();
  });

  it("presents the authoritative journey passengers seats extras and pricing", async () => {
    setFetch(context);
    render(
      <ReviewPage
        backQuery="flightId=xf-201&departure=2030-05-10&selectedCabin=economy"
        holdId="hold-123"
      />,
    );

    expect(await screen.findByRole("heading", { name: "Review your journey" })).toBeInTheDocument();
    expect(await screen.findByText("BKK")).toBeInTheDocument();
    expect(screen.getByText("LHR")).toBeInTheDocument();
    expect(screen.getByText("09:15")).toBeInTheDocument();
    expect(screen.getByText("Airbus A350-900")).toBeInTheDocument();
    expect(screen.getByText("MS Nara Suri")).toBeInTheDocument();
    expect(screen.getByText("20A")).toBeInTheDocument();
    expect(screen.getByText("+20 kg")).toBeInTheDocument();
    expect(screen.getByText("Vegetarian")).toBeInTheDocument();
    expect(screen.getByText("Free")).toBeInTheDocument();

    const summary = screen.getByRole("complementary", { name: "Fare summary" });
    expect(within(summary).getByText("THB 43,800")).toBeInTheDocument();
    expect(within(summary).getByText("Infant × 1")).toBeInTheDocument();
    expect(within(summary).getByText("THB 0")).toBeInTheDocument();
    expect(within(summary).getByText("Demo passenger tax")).toBeInTheDocument();
    expect(within(summary).getByText("Demo airport fee")).toBeInTheDocument();
    expect(within(summary).getByText("THB 49,300")).toBeInTheDocument();
    expect(screen.getByText(/demo fare configuration/i)).toBeInTheDocument();
    expect(screen.getByText("Non-refundable demo fare")).toBeInTheDocument();

    const payment = screen.getByRole("button", { name: "Proceed to Payment" });
    expect(payment).toBeDisabled();
    expect(screen.queryByRole("link", { name: "Proceed to Payment" })).not.toBeInTheDocument();
    expect(screen.getByText("Payment will be available in the next step.")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Back to Travel Extras" })).toHaveAttribute(
      "href",
      expect.stringContaining("/booking/extras?"),
    );
    expect(screen.getByTestId("review-layout")).toHaveClass("min-w-0");
  });

  it("exposes stable motion hooks without changing the disabled Payment contract", async () => {
    setFetch(context);
    const { container } = render(
      <ReviewPage backQuery="flightId=xf-201" holdId="hold-123" />,
    );
    await screen.findByText("MS Nara Suri");

    expect(
      Array.from(container.querySelectorAll("[data-review-reveal]")).map(
        (element) => element.getAttribute("data-review-reveal"),
      ),
    ).toEqual(["navigation", "heading", "details", "summary", "confirmation"]);
    expect(container.querySelectorAll("[data-review-card]").length).toBeGreaterThanOrEqual(6);
    expect(container.querySelector('[data-review-value="grand-total"]')).toHaveTextContent(
      "THB 49,300",
    );
    expect(container.querySelector('[data-review-value="extras-total"]')).toHaveTextContent(
      "THB 2,800",
    );
    expect(container.querySelector('[data-review-value="paid-extra"]')).toHaveTextContent(
      "THB 2,800",
    );

    const payment = screen.getByRole("button", { name: "Proceed to Payment" });
    expect(payment).toBeDisabled();
    expect(payment).toHaveAttribute("data-review-payment", "disabled");
    expect(screen.queryByRole("link", { name: "Proceed to Payment" })).not.toBeInTheDocument();
  });

  it("renders Thai customer copy while preserving identifiers and currency", async () => {
    setFetch(context);
    render(<ReviewPage backQuery="flightId=xf-201" holdId="hold-123" />, { locale: "th" });

    expect(await screen.findByRole("heading", { name: "ตรวจสอบการเดินทางของคุณ" })).toBeInTheDocument();
    expect(await screen.findByText("MS Nara Suri")).toBeInTheDocument();
    expect(screen.getByText("20A")).toBeInTheDocument();
    expect(screen.getByText("THB 49,300")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "ไปยังการชำระเงิน" })).toBeDisabled();
  });

  it("uses restrained acknowledgement and countdown accessibility", async () => {
    setFetch(context);
    render(<ReviewPage backQuery="flightId=xf-201" holdId="hold-123" />);
    await screen.findByText("MS Nara Suri");

    const timer = screen.getByRole("timer");
    expect(timer).toHaveAttribute("aria-live", "off");
    expect(screen.getByText(/expires at/i)).toHaveClass("sr-only");
    fireEvent.click(screen.getByRole("checkbox", { name: /names match/i }));
    fireEvent.click(screen.getByRole("checkbox", { name: /demo fare conditions/i }));
    const ready = screen.getByText("Review confirmations complete.");
    expect(ready).toHaveAttribute("role", "status");
    expect(ready).toHaveAttribute("data-review-ready", "true");
    expect(screen.getByRole("button", { name: "Proceed to Payment" })).toBeDisabled();
  });

  it("recovers to the exact prerequisite without fabricating review data", async () => {
    setFetch(
      { error: { code: "EXTRAS_NOT_READY", message: "Extras not ready" } },
      false,
      409,
    );
    render(<ReviewPage backQuery="flightId=xf-201" holdId="hold-123" />);

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent(
      "Save or confirm Travel Extras before reviewing your booking.",
    );
    expect(alert.closest("[data-review-recovery]")).toHaveAttribute(
      "data-review-recovery",
      "extras",
    );
    expect(screen.getByRole("link", { name: "Go to Travel Extras" })).toHaveAttribute(
      "href",
      expect.stringContaining("holdId=hold-123"),
    );
    expect(screen.queryByText("THB 49,300")).not.toBeInTheDocument();
  });

  it("announces expiry once and removes the future-payment action", async () => {
    jest.useFakeTimers();
    setFetch({
      ...context,
      hold: {
        ...context.hold,
        expiresAt: "2099-05-01T10:00:01Z",
        serverTime: "2099-05-01T10:00:00Z",
      },
    });
    render(<ReviewPage backQuery="flightId=xf-201" holdId="hold-123" />);
    await screen.findByText("MS Nara Suri");

    act(() => { jest.advanceTimersByTime(1_000); });

    expect(screen.getByRole("alert")).toHaveTextContent("expired while you were reviewing");
    expect(screen.getByRole("link", { name: "Return to Seat Selection" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Proceed to Payment" })).not.toBeInTheDocument();
  });
});
