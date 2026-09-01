import { screen } from "@testing-library/react";

import TravelExtrasRoute from "@/app/booking/extras/page";
import { render } from "@/tests/renderWithLanguage";

jest.mock("@/components/booking/extras/TravelExtrasPage", () => ({
  TravelExtrasPage: ({
    backQuery,
    holdId,
  }: {
    backQuery: string;
    holdId: string;
  }) => <div data-back-query={backQuery}>extras:{holdId}</div>,
}));

describe("Travel Extras route", () => {
  it("passes public hold authority separately from allowlisted recovery context", async () => {
    const route = await TravelExtrasRoute({
      searchParams: Promise.resolve({
        departure: "2027-05-10",
        email: "private@example.com",
        flightId: "xf-201",
        holdId: "hold-123",
        seats: "20A,20B",
        selectedCabin: "economy",
      }),
    });
    render(route);

    expect(screen.getByText("extras:hold-123")).toHaveAttribute(
      "data-back-query",
      "departure=2027-05-10&flightId=xf-201&selectedCabin=economy",
    );
  });

  it("does not invent a missing hold id", async () => {
    const route = await TravelExtrasRoute({
      searchParams: Promise.resolve({ flightId: "xf-201" }),
    });
    render(route);
    expect(screen.getByText("extras:")).toHaveAttribute(
      "data-back-query",
      "flightId=xf-201",
    );
  });
});

