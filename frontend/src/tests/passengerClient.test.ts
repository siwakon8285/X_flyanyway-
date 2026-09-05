import {
  getPassengerContext,
  savePassengerDraft,
} from "@/components/booking/passengers/passengerClient";
import type { PassengerContext } from "@/components/booking/passengers/passengerTypes";

const context: PassengerContext = {
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
  expectedPassengers: [{ ordinal: 1, passengerType: "ADULT" }],
  passengers: [],
  readyToContinue: false,
};

describe("passenger client", () => {
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

  it("loads the hold-bound passenger resource with cookies", async () => {
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

    await expect(getPassengerContext("hold-123")).resolves.toEqual(context);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:8080/api/v1/seat-holds/hold-123/passengers",
      expect.objectContaining({ cache: "no-store", credentials: "include" }),
    );
  });

  it("saves one complete draft atomically without putting PII in the URL", async () => {
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
    const passengers = [
      {
        ordinal: 1,
        passengerType: "ADULT" as const,
        title: "MS" as const,
        givenName: "Nara",
        middleName: null,
        familyName: "Suri",
        dateOfBirth: "1990-01-01",
        gender: "FEMALE" as const,
        nationalityCode: "TH",
        passportNumber: "TH123456",
        passportIssuingCountryCode: "TH",
        email: "nara@example.com",
        phoneCountryCode: "+66",
        phoneNumber: "812345678",
        emergencyContact: null,
      },
    ];

    await savePassengerDraft("hold-123", passengers);
    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe("http://localhost:8080/api/v1/seat-holds/hold-123/passengers");
    expect(String(url)).not.toContain("nara@example.com");
    expect(init).toMatchObject({ cache: "no-store", credentials: "include", method: "PUT" });
    expect(JSON.parse(String(init?.body))).toEqual({ passengers });
  });

  it("submits the explicit booking contact and locale in the private request body", async () => {
    const fetchMock = jest.fn().mockResolvedValue({
      json: async () => context,
      ok: true,
      status: 200,
    });
    Object.defineProperty(global, "fetch", { configurable: true, value: fetchMock, writable: true });
    await savePassengerDraft("hold-123", [], { email: "booking@example.test", preferredLocale: "TH" });
    const [, init] = fetchMock.mock.calls[0];
    expect(JSON.parse(String(init?.body))).toEqual({ passengers: [], bookingContact: { email: "booking@example.test", preferredLocale: "TH" } });
  });
});
