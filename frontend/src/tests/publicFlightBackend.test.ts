import { fetchPublicAirports } from "@/lib/flights/publicFlightBackend";

describe("public airport backend", () => {
  const originalFetch = global.fetch;

  afterEach(() => {
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: originalFetch,
      writable: true,
    });
  });

  it("maps the authoritative airport master into customer search options", async () => {
    const fetchMock = jest.fn().mockResolvedValue({
      json: async () => [
        {
          city: "Sydney (Mascot)",
          code: "SYD",
          countryCode: "AU",
          countryName: "Australia",
          name: "Sydney Kingsford Smith International Airport",
          timeZone: "Australia/Sydney",
        },
      ],
      ok: true,
    });
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: fetchMock,
      writable: true,
    });

    await expect(fetchPublicAirports()).resolves.toEqual([
      {
        airport: "Sydney Kingsford Smith International Airport",
        city: "Sydney (Mascot)",
        code: "SYD",
        country: "Australia",
      },
    ]);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:8080/api/v1/airports",
      expect.objectContaining({ cache: "no-store" }),
    );
  });

  it("fails closed when the airport master is unavailable", async () => {
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: jest.fn().mockRejectedValue(new Error("offline")),
      writable: true,
    });

    await expect(fetchPublicAirports()).resolves.toBeNull();
  });
});
