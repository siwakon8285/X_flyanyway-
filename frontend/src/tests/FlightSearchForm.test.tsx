import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { render } from "@/tests/renderWithLanguage";

import { AIRPORT_FIXTURES } from "@/components/booking/search/airportFixtures";
import { FlightSearchForm } from "@/components/booking/search/FlightSearchForm";
import { FlightSearchSection } from "@/components/booking/search/FlightSearchSection";
import { createDefaultFlightSearchValues } from "@/components/booking/search/searchState";
import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";

const mockPush = jest.fn();

jest.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockPush }),
}));

const createValidValues = (): FlightSearchFormValues => ({
  cabin: "economy",
  departure: "2099-05-10",
  from: AIRPORT_FIXTURES[0],
  passengers: { adults: 1, children: 0, infants: 0 },
  returnDate: "2099-05-18",
  to: AIRPORT_FIXTURES[2],
  trip: "round-trip",
});

describe("FlightSearchForm", () => {
  beforeEach(() => {
    mockPush.mockClear();
    window.history.replaceState(null, "", "/");
  });

  it("renders the complete search form with Round Trip selected by default", () => {
    render(
      <FlightSearchForm
        initialValues={createDefaultFlightSearchValues()}
        onValidSubmit={jest.fn()}
      />,
    );

    expect(
      screen.getByRole("form", { name: "Search Earth flights" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "Round Trip" })).toBeChecked();
    expect(screen.getByRole("radio", { name: "One Way" })).not.toBeChecked();
    expect(screen.getByRole("button", { name: "Choose origin" })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Choose destination" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Departure date")).toBeInTheDocument();
    expect(screen.getByLabelText("Return date")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Passengers, 1 total" })).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: "Cabin class" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Search Flights" })).toBeInTheDocument();
  });

  it("removes the return-date control for One Way and restores it for Round Trip", () => {
    render(
      <FlightSearchForm
        initialValues={createDefaultFlightSearchValues()}
        onValidSubmit={jest.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("radio", { name: "One Way" }));
    expect(screen.queryByLabelText("Return date")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("radio", { name: "Round Trip" }));
    expect(screen.getByLabelText("Return date")).toBeInTheDocument();
  });

  it("filters and selects local origin and destination airports", () => {
    render(
      <FlightSearchForm
        initialValues={createDefaultFlightSearchValues()}
        onValidSubmit={jest.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Choose origin" }));
    const originDialog = screen.getByRole("dialog", { name: "Search airport or city" });
    fireEvent.change(within(originDialog).getByRole("textbox"), {
      target: { value: "Tokyo" },
    });
    expect(within(originDialog).getByRole("button", { name: /HND Tokyo/ })).toBeInTheDocument();
    expect(within(originDialog).queryByRole("button", { name: /BKK Bangkok/ })).not.toBeInTheDocument();
    fireEvent.click(within(originDialog).getByRole("button", { name: /HND Tokyo/ }));

    expect(
      screen.getByRole("button", { name: "From HND, Tokyo, Japan" }),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Choose destination" }));
    const destinationDialog = screen.getByRole("dialog", {
      name: "Search airport or city",
    });
    fireEvent.click(
      within(destinationDialog).getByRole("button", { name: /LHR London/ }),
    );
    expect(
      screen.getByRole("button", {
        name: "To LHR, London, United Kingdom",
      }),
    ).toBeInTheDocument();
  });

  it("swaps origin and destination without changing the route layout", () => {
    render(
      <FlightSearchForm initialValues={createValidValues()} onValidSubmit={jest.fn()} />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Swap origin and destination" }));

    expect(
      screen.getByRole("button", { name: "From LHR, London, United Kingdom" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "To BKK, Bangkok, Thailand" }),
    ).toBeInTheDocument();
  });

  it("keeps one adult minimum and updates the total as passenger counts change", () => {
    render(
      <FlightSearchForm
        initialValues={createDefaultFlightSearchValues()}
        onValidSubmit={jest.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Passengers, 1 total" }));
    expect(screen.getByRole("button", { name: /Decrease adults/i })).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: /Increase children/i }));
    expect(screen.getByLabelText("Children 1")).toHaveTextContent("1");

    fireEvent.click(screen.getByRole("button", { name: "Close dialog" }));
    expect(screen.getByRole("button", { name: "Passengers, 2 total" })).toBeInTheDocument();
  });

  it("selects a cabin class through the accessible select", async () => {
    render(
      <FlightSearchForm
        initialValues={createDefaultFlightSearchValues()}
        onValidSubmit={jest.fn()}
      />,
    );

    const cabin = screen.getByRole("combobox", { name: "Cabin class" });
    fireEvent.keyDown(cabin, { key: "ArrowDown" });
    fireEvent.click(await screen.findByRole("option", { name: "Business" }));

    await waitFor(() => expect(cabin).toHaveTextContent("Business"));
  });

  it("blocks invalid submission and connects route and date errors to their controls", () => {
    const onValidSubmit = jest.fn();
    render(
      <FlightSearchForm
        initialValues={createDefaultFlightSearchValues()}
        onValidSubmit={onValidSubmit}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Search Flights" }));

    expect(onValidSubmit).not.toHaveBeenCalled();
    expect(screen.getByText("Choose an origin.")).toBeInTheDocument();
    expect(screen.getByText("Choose a destination.")).toBeInTheDocument();
    expect(screen.getByText("Choose a departure date.")).toBeInTheDocument();
    expect(screen.getByText("Choose a return date.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Choose origin" })).toHaveAttribute(
      "aria-describedby",
      "from-error",
    );
  });

  it("reports an identical origin and destination instead of continuing", () => {
    const onValidSubmit = jest.fn();
    render(
      <FlightSearchForm
        initialValues={{ ...createValidValues(), to: AIRPORT_FIXTURES[0] }}
        onValidSubmit={onValidSubmit}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Search Flights" }));

    expect(onValidSubmit).not.toHaveBeenCalled();
    expect(
      screen.getByText("Origin and destination must be different."),
    ).toBeInTheDocument();
  });

  it("rejects past departure and return-before-departure dates", () => {
    render(
      <FlightSearchForm
        initialValues={{
          ...createValidValues(),
          departure: "2000-01-02",
          returnDate: "2000-01-01",
        }}
        onValidSubmit={jest.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Search Flights" }));

    expect(screen.getByText("Departure date cannot be in the past.")).toBeInTheDocument();
    expect(
      screen.getByText("Return date must be on or after departure."),
    ).toBeInTheDocument();
  });

  it("submits valid local state, announces readiness, and makes no network request", () => {
    const onValidSubmit = jest.fn();
    const originalFetch = global.fetch;
    const fetchMock = jest.fn();
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: fetchMock,
      writable: true,
    });

    try {
      render(
        <FlightSearchForm initialValues={createValidValues()} onValidSubmit={onValidSubmit} />,
      );

      fireEvent.click(screen.getByRole("button", { name: "Search Flights" }));

      expect(onValidSubmit).toHaveBeenCalledWith(createValidValues());
      expect(screen.getByRole("status")).toHaveTextContent(
        "Search criteria ready. Flight results will be available in the next step.",
      );
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

  it("contains no Moon destination option", () => {
    render(
      <FlightSearchForm
        initialValues={createDefaultFlightSearchValues()}
        onValidSubmit={jest.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Choose destination" }));
    expect(screen.queryByText(/moon/i)).not.toBeInTheDocument();
  });

  it("hydrates supported URL criteria and navigates to the results route after submit", async () => {
    window.history.replaceState(
      null,
      "",
      "/?from=BKK&to=LHR&departure=2099-05-10&adults=1&children=2&infants=0&cabin=first&trip=one-way#flight-search",
    );
    render(<FlightSearchSection />);

    expect(
      await screen.findByRole("button", { name: "From BKK, Bangkok, Thailand" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "One Way" })).toBeChecked();
    expect(screen.getByRole("combobox", { name: "Cabin class" })).toHaveTextContent(
      "First",
    );
    expect(screen.getByRole("button", { name: "Passengers, 3 total" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Swap origin and destination" }));
    fireEvent.click(screen.getByRole("button", { name: "Search Flights" }));

    expect(mockPush).toHaveBeenCalledWith(
      "/flights?from=LHR&to=BKK&departure=2099-05-10&adults=1&children=2&infants=0&cabin=first&trip=one-way",
    );
  });
});
