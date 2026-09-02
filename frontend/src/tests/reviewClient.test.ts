import { getReviewContext } from "@/components/booking/review/reviewClient";
import type { ReviewContext } from "@/components/booking/review/reviewTypes";

const context = {
  readyForPayment: true,
} as ReviewContext;

describe("review client", () => {
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

  it("loads only the hold-bound Review resource with cookie credentials", async () => {
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

    await expect(getReviewContext("hold-123")).resolves.toEqual(context);
    expect(fetchMock).toHaveBeenCalledWith(
      "http://localhost:8080/api/v1/seat-holds/hold-123/review",
      expect.objectContaining({ cache: "no-store", credentials: "include" }),
    );
    expect(fetchMock.mock.calls[0][1]).not.toHaveProperty("body");
  });
});
