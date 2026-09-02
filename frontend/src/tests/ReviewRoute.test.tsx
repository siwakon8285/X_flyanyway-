import { screen } from "@testing-library/react";

import ReviewRoute from "@/app/booking/review/page";
import { render } from "@/tests/renderWithLanguage";

jest.mock("@/components/booking/review/ReviewPage", () => ({
  ReviewPage: ({ backQuery, holdId }: { backQuery: string; holdId: string }) => (
    <div data-back-query={backQuery}>review:{holdId}</div>
  ),
}));

describe("Review route", () => {
  it("passes the public hold separately from allowlisted recovery context", async () => {
    const route = await ReviewRoute({
      searchParams: Promise.resolve({
        departure: "2027-05-10",
        email: "private@example.com",
        flightId: "xf-201",
        grandTotal: "1",
        holdId: "hold-123",
        seats: "20A,20B",
        selectedCabin: "economy",
      }),
    });
    render(route);

    expect(screen.getByText("review:hold-123")).toHaveAttribute(
      "data-back-query",
      "departure=2027-05-10&flightId=xf-201&selectedCabin=economy",
    );
  });
});
