import { fireEvent, screen, waitFor, within } from "@testing-library/react";

import { TravelExtrasPage } from "@/components/booking/extras/TravelExtrasPage";
import type {
  ExtraProduct,
  ExtraSelection,
  ExtrasContext,
} from "@/components/booking/extras/extrasTypes";
import { render } from "@/tests/renderWithLanguage";

const mockPush = jest.fn();

jest.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockPush }),
}));

const money = (amount: number) => ({ amount, currencyCode: "THB" });
const product = (
  code: string,
  category: ExtraProduct["category"],
  amount = 0,
  eligiblePassengerTypes: ExtraProduct["eligiblePassengerTypes"] = ["ADULT", "CHILD"],
): ExtraProduct => ({
  category,
  code,
  eligiblePassengerTypes,
  maxQuantity: 1,
  unitPrice: money(amount),
});

const context: ExtrasContext = {
  catalog: {
    allowances: {
      appliesToPassengerTypes: ["ADULT", "CHILD"],
      cabinBaggageKg: 7,
      checkedBaggageKg: 20,
    },
    currencyCode: "THB",
    includedBenefits: {
      mealService: "STANDARD",
      seatSelectionIncluded: true,
    },
    products: [
      product("BAG_10KG", "BAGGAGE", 1500),
      product("BAG_20KG", "BAGGAGE", 2800),
      product("BAG_30KG", "BAGGAGE", 3900),
      product("MEAL_VEGETARIAN", "MEAL"),
      product("MEAL_VEGAN", "MEAL"),
      product("MEAL_HALAL", "MEAL"),
      product("MEAL_KOSHER", "MEAL"),
      product("MEAL_CHILD", "MEAL", 0, ["CHILD"]),
      product("ASSIST_WHEELCHAIR", "ASSISTANCE"),
      product("ASSIST_MOBILITY", "ASSISTANCE"),
      product("ASSIST_VISUAL", "ASSISTANCE"),
      product("ASSIST_HEARING", "ASSISTANCE"),
    ],
  },
  hold: {
    cabin: "economy",
    departureDate: "2027-05-10",
    expiresAt: "2099-05-01T10:10:00Z",
    flightId: "xf-201",
    id: "hold-123",
    passengers: { adults: 1, children: 1, infants: 1 },
    seats: ["20A", "20B"],
    serverTime: "2099-05-01T10:00:00Z",
  },
  passengers: [
    { ordinal: 1, passengerType: "ADULT" },
    { ordinal: 2, passengerType: "CHILD" },
    { ordinal: 3, passengerType: "INFANT" },
  ],
  readyToContinue: false,
  savedAt: null,
  selections: [],
  total: money(0),
};

const savedSelection = (
  passengerOrdinal: number,
  productCode: string,
  category: ExtraSelection["category"],
  amount: number,
): ExtraSelection => ({
  category,
  lineTotal: money(amount),
  passengerOrdinal,
  productCode,
  quantity: 1,
  unitPrice: money(amount),
});

const setFetch = (...responses: Array<{ body: unknown; ok: boolean; status: number }>) => {
  const fetchMock = jest.fn();
  responses.forEach(({ body, ok, status }) => {
    fetchMock.mockResolvedValueOnce({ json: async () => body, ok, status });
  });
  Object.defineProperty(global, "fetch", {
    configurable: true,
    value: fetchMock,
    writable: true,
  });
  return fetchMock;
};

describe("TravelExtrasPage", () => {
  const originalFetch = global.fetch;

  afterEach(() => {
    mockPush.mockReset();
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

  it("renders cabin benefits and stable passenger sections without exposing PII", async () => {
    setFetch({ body: context, ok: true, status: 200 });
    render(
      <TravelExtrasPage
        backQuery="flightId=xf-201&departure=2027-05-10&selectedCabin=economy"
        holdId="hold-123"
      />,
    );

    expect(await screen.findByText("20 kg checked baggage")).toBeInTheDocument();
    expect(screen.getByText("7 kg cabin baggage")).toBeInTheDocument();
    expect(screen.getByText("Seat selection included")).toBeInTheDocument();
    const navigation = screen.getByRole("navigation", { name: "Passengers" });
    expect(within(navigation).getByRole("button", { name: "Passenger 1 — Adult" })).toBeInTheDocument();
    expect(within(navigation).getByRole("button", { name: "Passenger 2 — Child" })).toBeInTheDocument();
    expect(within(navigation).getByRole("button", { name: "Passenger 3 — Infant" })).toBeInTheDocument();
    expect(screen.queryByText("Nara")).not.toBeInTheDocument();
    expect(screen.getByText("20A")).toBeInTheDocument();
  });

  it("updates baggage meal assistance and total then navigates after the server confirms save", async () => {
    const saved: ExtrasContext = {
      ...context,
      readyToContinue: true,
      savedAt: "2099-05-01T10:01:00Z",
      selections: [
        savedSelection(1, "BAG_20KG", "BAGGAGE", 2800),
        savedSelection(1, "MEAL_VEGETARIAN", "MEAL", 0),
        savedSelection(1, "ASSIST_HEARING", "ASSISTANCE", 0),
      ],
      total: money(2800),
    };
    const fetchMock = setFetch(
      { body: context, ok: true, status: 200 },
      { body: saved, ok: true, status: 200 },
    );
    render(<TravelExtrasPage backQuery="flightId=xf-201" holdId="hold-123" />);
    await screen.findByText("20 kg checked baggage");

    const paidBaggage = screen.getByRole("radio", { name: "+20 kg THB 2,800" });
    const noBaggage = screen.getByRole("radio", { name: "No extra checked baggage" });
    fireEvent.click(paidBaggage);
    expect(paidBaggage.closest("label")).toHaveAttribute("data-selected", "true");
    expect(noBaggage.closest("label")).toHaveAttribute("data-selected", "false");
    expect(screen.getByLabelText("Extras total THB 2,800")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("radio", { name: "Vegetarian" }));
    fireEvent.click(screen.getByRole("checkbox", { name: "Hearing assistance" }));
    const saveButton = screen.getByRole("button", { name: "Save & Continue" });
    expect(saveButton).toHaveAttribute("data-save-state", "dirty");
    fireEvent.click(saveButton);

    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(2));
    expect(await screen.findByText("Extras saved. Ready for review.")).toBeInTheDocument();
    expect(saveButton).toHaveAttribute("data-save-state", "saved");
    expect(mockPush).toHaveBeenCalledWith(
      "/booking/review?flightId=xf-201&holdId=hold-123",
    );

    fireEvent.click(noBaggage);
    expect(saveButton).toHaveAttribute("data-save-state", "dirty");
    expect(screen.queryByText("Extras saved. Ready for review.")).not.toBeInTheDocument();
  });

  it("restores saved choices after refresh and supports deselection", async () => {
    const restored: ExtrasContext = {
      ...context,
      readyToContinue: true,
      savedAt: "2099-05-01T10:01:00Z",
      selections: [savedSelection(1, "BAG_10KG", "BAGGAGE", 1500)],
      total: money(1500),
    };
    setFetch({ body: restored, ok: true, status: 200 });
    render(<TravelExtrasPage backQuery="flightId=xf-201" holdId="hold-123" />);

    const selected = await screen.findByRole("radio", { name: "+10 kg THB 1,500" });
    expect(selected).toBeChecked();
    fireEvent.click(screen.getByRole("radio", { name: "No extra checked baggage" }));
    expect(screen.getByLabelText("Extras total THB 0")).toBeInTheDocument();
    expect(screen.queryByText("Extras saved. Ready for review.")).not.toBeInTheDocument();
  });

  it("keeps infants informational only and translates the workflow into Thai", async () => {
    setFetch({ body: context, ok: true, status: 200 });
    render(<TravelExtrasPage backQuery="flightId=xf-201" holdId="hold-123" />, {
      locale: "th",
    });
    const navigation = await screen.findByRole("navigation", { name: "ผู้โดยสาร" });
    fireEvent.click(within(navigation).getByRole("button", { name: "ผู้โดยสาร 3 — ทารก" }));

    expect(screen.getByText(/ทารกไม่มีบริการเสริมแยกต่างหาก/)).toBeInTheDocument();
    expect(screen.queryByRole("group", { name: /สัมภาระเพิ่มเติม/ })).not.toBeInTheDocument();
    expect(screen.getByText("THB 0")).toBeInTheDocument();
    expect(screen.getByText("20A")).toBeInTheDocument();
  });

  it("blocks saving after hold expiry and preserves exact seat-selection recovery", async () => {
    setFetch({
      body: { error: { code: "HOLD_EXPIRED", message: "The seat hold has expired." } },
      ok: false,
      status: 410,
    });
    render(
      <TravelExtrasPage
        backQuery="flightId=xf-201&departure=2027-05-10&adults=1&children=1&infants=1&selectedCabin=economy"
        holdId="hold-123"
      />,
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Your seat hold expired. Return to seat selection to choose seats again.",
    );
    expect(screen.queryByRole("button", { name: "Save & Continue" })).not.toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Return to seat selection" })).toHaveAttribute(
      "href",
      expect.stringContaining("/flights/xf-201/seats?"),
    );
  });

  it("retries a recoverable load failure without sending the user back to seats", async () => {
    const fetchMock = setFetch(
      {
        body: { error: { code: "INTERNAL_ERROR", message: "Temporary failure" } },
        ok: false,
        status: 503,
      },
      { body: context, ok: true, status: 200 },
    );
    render(<TravelExtrasPage backQuery="flightId=xf-201" holdId="hold-123" />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Travel Extras are temporarily unavailable",
    );
    expect(screen.queryByRole("link", { name: "Return to seat selection" })).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Try again" }));

    expect(await screen.findByText("20 kg checked baggage")).toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it("keeps unsaved selections after a recoverable save failure and retries", async () => {
    const saved: ExtrasContext = {
      ...context,
      readyToContinue: true,
      savedAt: "2099-05-01T10:01:00Z",
      selections: [savedSelection(1, "BAG_20KG", "BAGGAGE", 2800)],
      total: money(2800),
    };
    const fetchMock = setFetch(
      { body: context, ok: true, status: 200 },
      {
        body: { error: { code: "INTERNAL_ERROR", message: "Temporary failure" } },
        ok: false,
        status: 503,
      },
      { body: saved, ok: true, status: 200 },
    );
    render(<TravelExtrasPage backQuery="flightId=xf-201" holdId="hold-123" />);
    await screen.findByText("20 kg checked baggage");

    const baggage = screen.getByRole("radio", { name: "+20 kg THB 2,800" });
    fireEvent.click(baggage);
    fireEvent.click(screen.getByRole("button", { name: "Save & Continue" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "We couldn't save your extras. Your choices are still here. Please try again.",
    );
    expect(mockPush).not.toHaveBeenCalled();
    expect(baggage).toBeChecked();
    expect(screen.getByLabelText("Extras total THB 2,800")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Save & Continue" }));

    expect(await screen.findByText("Extras saved. Ready for review.")).toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledTimes(3);
    expect(mockPush).toHaveBeenCalledTimes(1);
  });
});
