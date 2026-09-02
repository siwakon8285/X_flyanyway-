import { screen } from "@testing-library/react";

import PaymentRoute from "@/app/booking/payment/page";
import { render } from "@/tests/renderWithLanguage";

jest.mock("@/components/booking/payment/PaymentPage", () => ({
  PaymentPage: ({ backQuery, holdId }: { backQuery: string; holdId: string }) => (
    <div data-back-query={backQuery}>payment:{holdId}</div>
  ),
}));

it("passes the public hold separately from allowlisted recovery context", async () => {
  const route = await PaymentRoute({
    searchParams: Promise.resolve({
      departure: "2027-05-10",
      email: "private@example.com",
      flightId: "xf-201",
      grandTotal: "1",
      holdId: "hold-123",
      seats: "20A",
    }),
  });
  render(route);

  expect(screen.getByText("payment:hold-123")).toHaveAttribute(
    "data-back-query",
    "departure=2027-05-10&flightId=xf-201",
  );
});
