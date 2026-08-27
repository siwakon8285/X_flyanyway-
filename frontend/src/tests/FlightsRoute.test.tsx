import { render, screen } from "@testing-library/react";

import FlightsRoute from "@/app/flights/page";

describe("/flights route", () => {
  it("renders results from supported query parameters", async () => {
    const route = await FlightsRoute({
      searchParams: Promise.resolve({
        adults: "1",
        cabin: "business",
        children: "0",
        departure: "2099-05-10",
        from: "BKK",
        infants: "0",
        return: "2099-05-18",
        to: "LHR",
        trip: "round-trip",
      }),
    });

    render(route);

    expect(screen.getByRole("heading", { level: 1, name: /BKK.*LHR/ })).toBeInTheDocument();
    expect(screen.getAllByRole("article").length).toBeGreaterThan(0);
  });

  it("does not display parser fallbacks as valid results", async () => {
    const route = await FlightsRoute({
      searchParams: Promise.resolve({
        adults: "1",
        cabin: "spaceship",
        children: "0",
        departure: "2099-05-10",
        from: "BKK",
        infants: "0",
        to: "MOON",
        trip: "one-way",
      }),
    });

    render(route);

    expect(screen.getByRole("heading", { name: "Search details required" })).toBeInTheDocument();
    expect(screen.queryByRole("article")).not.toBeInTheDocument();
  });
});
