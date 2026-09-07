import TravelExtrasRoute from "@/app/(customer)/booking/extras/page";

const mockRedirect = jest.fn((href: string): never => {
  throw new Error(`NEXT_REDIRECT:${href}`);
});

jest.mock("next/navigation", () => ({
  redirect: (href: string) => mockRedirect(href),
}));

describe("legacy Travel Extras route", () => {
  beforeEach(() => mockRedirect.mockClear());

  it("redirects a valid legacy URL to Review while preserving hold recovery state", async () => {
    await expect(
      TravelExtrasRoute({
        searchParams: Promise.resolve({
          departure: "2027-05-10",
          email: "private@example.com",
          flightId: "xf-201",
          holdId: "hold-123",
          seats: "3A,3D",
          selectedCabin: "business",
        }),
      }),
    ).rejects.toThrow("NEXT_REDIRECT:/booking/review?");

    expect(mockRedirect).toHaveBeenCalledWith(
      "/booking/review?departure=2027-05-10&flightId=xf-201&selectedCabin=business&holdId=hold-123",
    );
  });

  it("redirects missing-hold legacy URLs without inventing authority", async () => {
    await expect(
      TravelExtrasRoute({
        searchParams: Promise.resolve({ flightId: "xf-201" }),
      }),
    ).rejects.toThrow("NEXT_REDIRECT:/booking/review?flightId=xf-201");
    expect(mockRedirect).toHaveBeenCalledWith(
      "/booking/review?flightId=xf-201",
    );
  });
});
