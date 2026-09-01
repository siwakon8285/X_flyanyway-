import {
  getExtrasContext,
  saveExtras,
} from "@/components/booking/extras/extrasClient";
import type { ExtrasContext } from "@/components/booking/extras/extrasTypes";

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
      {
        category: "BAGGAGE",
        code: "BAG_10KG",
        eligiblePassengerTypes: ["ADULT", "CHILD"],
        maxQuantity: 1,
        unitPrice: { amount: 1500, currencyCode: "THB" },
      },
    ],
  },
  hold: {
    cabin: "economy",
    departureDate: "2030-05-10",
    expiresAt: "2030-05-01T10:10:00Z",
    flightId: "xf-201",
    id: "hold-123",
    passengers: { adults: 1, children: 0, infants: 0 },
    seats: ["20A"],
    serverTime: "2030-05-01T10:00:00Z",
  },
  passengers: [{ ordinal: 1, passengerType: "ADULT" }],
  readyToContinue: false,
  savedAt: null,
  selections: [],
  total: { amount: 0, currencyCode: "THB" },
};

describe("extras client", () => {
  const originalFetch = global.fetch;

  afterEach(() => {
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

  it("loads the hold-bound Extras resource with HttpOnly cookie credentials", async () => {
    const fetchMock = jest.fn().mockResolvedValue({
      json: async () => context,
      ok: true,
      status: 200,
    });
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: fetchMock,
      writable: true,
    });

    await expect(getExtrasContext("hold-123")).resolves.toEqual(context);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:8080/api/v1/seat-holds/hold-123/extras",
      expect.objectContaining({ cache: "no-store", credentials: "include" }),
    );
  });

  it("saves only stable passenger ordinals product codes and quantities", async () => {
    const saved = { ...context, readyToContinue: true };
    const fetchMock = jest.fn().mockResolvedValue({
      json: async () => saved,
      ok: true,
      status: 200,
    });
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: fetchMock,
      writable: true,
    });
    const selections = [
      { passengerOrdinal: 1, productCode: "BAG_10KG", quantity: 1 },
    ];

    await saveExtras("hold-123", selections);
    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe("http://localhost:8080/api/v1/seat-holds/hold-123/extras");
    expect(String(url)).not.toContain("20A");
    expect(init).toMatchObject({
      cache: "no-store",
      credentials: "include",
      method: "PUT",
    });
    expect(JSON.parse(String(init.body))).toEqual({ selections });
    expect(String(init.body)).not.toContain("unitPrice");
  });
});

