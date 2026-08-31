import {
  createSeatHold,
  getRemainingHoldMilliseconds,
  SeatHoldApiError,
} from "@/components/booking/seats/seatHoldClient";

const request = {
  cabin: "business" as const,
  departureDate: "2099-05-10",
  flightId: "xf-201",
  passengers: { adults: 2, children: 1, infants: 1 },
  seats: ["3A"],
};

describe("seat hold API client", () => {
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

  it("uses credentialed server-authoritative hold requests without exposing a token", async () => {
    const fetchMock = jest.fn().mockResolvedValue({
      json: async () => ({
          cabin: "business",
          departureDate: "2099-05-10",
          expiresAt: "2099-05-10T10:10:00Z",
          flightId: "xf-201",
          id: "8d256f1e-4758-4997-861f-3f20a53c5846",
          passengers: request.passengers,
          seats: ["3A"],
          serverTime: "2099-05-10T10:00:00Z",
      }),
      ok: true,
      status: 201,
    });
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: fetchMock,
      writable: true,
    });

    const hold = await createSeatHold(request);

    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:8080/api/v1/seat-holds",
      expect.objectContaining({ credentials: "include", method: "POST" }),
    );
    expect(hold.id).toBe("8d256f1e-4758-4997-861f-3f20a53c5846");
    expect(hold).not.toHaveProperty("accessToken");
  });

  it("surfaces machine-readable conflicts for optimistic rollback", async () => {
    const fetchMock = jest.fn().mockResolvedValue({
      json: async () => ({
          error: {
            code: "SEAT_UNAVAILABLE",
            conflictingSeats: ["3A"],
            message: "One or more seats were just taken.",
          },
      }),
      ok: false,
      status: 409,
    });
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: fetchMock,
      writable: true,
    });

    await expect(createSeatHold(request)).rejects.toEqual(
      expect.objectContaining<Partial<SeatHoldApiError>>({
        code: "SEAT_UNAVAILABLE",
        conflictingSeats: ["3A"],
        status: 409,
      }),
    );
  });

  it("derives countdown time from server time instead of local request time", () => {
    expect(
      getRemainingHoldMilliseconds({
        clientNow: Date.parse("2099-05-10T09:58:30Z"),
        expiresAt: "2099-05-10T10:10:00Z",
        serverTime: "2099-05-10T10:00:00Z",
      }),
    ).toBe(10 * 60 * 1000);
  });
});
