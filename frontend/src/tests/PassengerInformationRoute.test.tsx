import { screen } from "@testing-library/react";

import PassengerInformationRoute from "@/app/booking/passengers/page";
import { render } from "@/tests/renderWithLanguage";

jest.mock("@/components/booking/passengers/PassengerInformationPage", () => ({
  PassengerInformationPage: ({
    backQuery,
    holdId,
  }: {
    backQuery: string;
    holdId: string;
  }) => <div data-back-query={backQuery}>hold:{holdId}</div>,
}));

describe("passenger information route", () => {
  it("passes the public hold id separately from non-sensitive recovery context", async () => {
    const route = await PassengerInformationRoute({
      searchParams: Promise.resolve({
        adults: "2",
        departure: "2027-05-10",
        flightId: "xf-201",
        holdId: "hold-123",
        seats: "20A,20B",
      }),
    });

    render(route);

    const boundary = screen.getByText("hold:hold-123");
    expect(boundary).toHaveAttribute(
      "data-back-query",
      "adults=2&departure=2027-05-10&flightId=xf-201",
    );
  });

  it("renders the valid Passenger route without inventing a hold id", async () => {
    const route = await PassengerInformationRoute({
      searchParams: Promise.resolve({ flightId: "xf-201" }),
    });

    render(route);

    expect(screen.getByText("hold:")).toHaveAttribute(
      "data-back-query",
      "flightId=xf-201",
    );
  });
});
