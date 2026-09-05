import {
  getCurrentManageBooking,
  getCurrentManageBookingTicket,
  lookupManageBooking,
} from "@/components/manage-booking/manageBookingClient";

const response = { bookingReference: "XFABCDEFGH" };

describe("manageBookingClient", () => {
  beforeEach(() => {
    global.fetch = jest.fn().mockResolvedValue({
      ok: true,
      json: async () => response,
    }) as jest.Mock;
  });

  it("keeps lookup identity fields in a POST body and out of the URL", async () => {
    await lookupManageBooking({
      bookingReference: "XFABCDEFGH",
      lastName: "Van der Meer",
    });

    const [url, init] = (global.fetch as jest.Mock).mock.calls[0] as [string, RequestInit];
    expect(url).toBe("http://localhost:8080/api/v1/manage-booking/lookup");
    expect(url).not.toContain("Van");
    expect(url).not.toContain("example.com");
    expect(init.method).toBe("POST");
    expect(JSON.parse(init.body as string)).toEqual({
      bookingReference: "XFABCDEFGH",
      lastName: "Van der Meer",
    });
    expect(init.credentials).toBe("include");
    expect(init.cache).toBe("no-store");
  });

  it("uses cookie-scoped current resources without booking identifiers", async () => {
    await getCurrentManageBooking();
    await getCurrentManageBookingTicket();

    expect(global.fetch).toHaveBeenNthCalledWith(
      1,
      "http://localhost:8080/api/v1/manage-booking/current",
      expect.objectContaining({ credentials: "include" }),
    );
    expect(global.fetch).toHaveBeenNthCalledWith(
      2,
      "http://localhost:8080/api/v1/manage-booking/current/ticket",
      expect.objectContaining({ credentials: "include" }),
    );
  });
});
