import { fireEvent, render, screen, within } from "@testing-library/react";

import { AIRPORT_FIXTURES } from "@/components/booking/search/airportFixtures";
import { serializeFlightSearch } from "@/components/booking/search/searchState";
import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import { FlightResultsPage } from "@/components/booking/results/FlightResultsPage";

const validCriteria: FlightSearchFormValues = {
  cabin: "premium-economy",
  departure: "2099-05-10",
  from: AIRPORT_FIXTURES[0],
  passengers: { adults: 1, children: 1, infants: 0 },
  returnDate: "2099-05-18",
  to: AIRPORT_FIXTURES[2],
  trip: "round-trip",
};

const renderResults = (criteria: FlightSearchFormValues | null = validCriteria) =>
  render(
    <FlightResultsPage
      criteria={criteria}
      query={criteria ? serializeFlightSearch(criteria) : ""}
    />,
  );

describe("FlightResultsPage", () => {
  it("renders a sanitized itinerary summary and outbound context", () => {
    renderResults();

    expect(screen.getByRole("heading", { level: 1, name: /BKK.*LHR/ })).toBeInTheDocument();
    expect(screen.getByText("Bangkok to London")).toBeInTheDocument();
    expect(screen.getByText("10 MAY 2099")).toBeInTheDocument();
    expect(screen.getByText(/2 passengers/i)).toBeInTheDocument();
    expect(screen.getAllByText(/premium economy/i).length).toBeGreaterThan(0);
    expect(screen.getByText("Select your outbound flight")).toBeInTheDocument();
  });

  it("renders schedule-first flight results with duration, aircraft, and searched-cabin fare", () => {
    renderResults();

    const firstFlight = screen.getAllByRole("article")[0];
    expect(within(firstFlight).getByText("XF 201")).toBeInTheDocument();
    expect(within(firstFlight).getByText("09:15")).toBeInTheDocument();
    expect(within(firstFlight).getByText("16:40")).toBeInTheDocument();
    expect(within(firstFlight).getByText("13h 25m")).toBeInTheDocument();
    expect(within(firstFlight).getByText("Direct")).toBeInTheDocument();
    expect(within(firstFlight).getByText("Airbus A350-900")).toBeInTheDocument();
    expect(within(firstFlight).getByText("THB 32,900")).toBeInTheDocument();
    expect(within(firstFlight).getByText("Sample fare · per passenger")).toBeInTheDocument();
  });

  it("sorts the local result list with an accessible control", () => {
    renderResults();

    expect(screen.getByLabelText("Sort flights")).toHaveValue("recommended");
    expect(screen.getAllByRole("article")[0]).toHaveTextContent("XF 201");

    fireEvent.change(screen.getByLabelText("Sort flights"), {
      target: { value: "departure" },
    });
    expect(screen.getAllByRole("article")[0]).toHaveTextContent("XF 315");

    fireEvent.change(screen.getByLabelText("Sort flights"), {
      target: { value: "duration" },
    });
    expect(screen.getAllByRole("article")[0]).toHaveTextContent("XF 428");

    fireEvent.change(screen.getByLabelText("Sort flights"), {
      target: { value: "price" },
    });
    expect(screen.getAllByRole("article")[0]).toHaveTextContent("XF 315");
  });

  it("provides future-ready selection and modify-search links without making a request", () => {
    const originalFetch = global.fetch;
    const fetchMock = jest.fn();
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: fetchMock,
      writable: true,
    });

    try {
      renderResults();

      expect(screen.getByRole("link", { name: "Modify Search" })).toHaveAttribute(
        "href",
        expect.stringMatching(/^\/\?.*#flight-search$/),
      );
      expect(
        screen.getByRole("link", { name: "Select flight XF 201" }),
      ).toHaveAttribute("href", expect.stringMatching(/^\/flights\/xf-201\?/));
      expect(fetchMock).not.toHaveBeenCalled();
    } finally {
      if (originalFetch) {
        Object.defineProperty(global, "fetch", {
          configurable: true,
          value: originalFetch,
          writable: true,
        });
      } else {
        Reflect.deleteProperty(global, "fetch");
      }
    }
  });

  it("shows a safe invalid-query state", () => {
    renderResults(null);

    expect(screen.getByRole("heading", { name: "Search details required" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Modify Search" })).toHaveAttribute(
      "href",
      "/#flight-search",
    );
    expect(screen.queryByRole("article")).not.toBeInTheDocument();
  });

  it("shows a useful empty state for a valid route without fixtures", () => {
    renderResults({
      ...validCriteria,
      from: AIRPORT_FIXTURES[3],
      to: AIRPORT_FIXTURES[4],
    });

    expect(screen.getByRole("heading", { name: "No flights found" })).toBeInTheDocument();
    expect(screen.getByText("Try adjusting your date, route, or cabin.")).toBeInTheDocument();
    expect(screen.queryByText(/moon/i)).not.toBeInTheDocument();
  });
});
