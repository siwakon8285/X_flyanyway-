import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { render } from "@/tests/renderWithLanguage";

import { FlightDetailPage } from "@/components/booking/detail/FlightDetailPage";
import { resolveFlightDetailRequest } from "@/components/booking/detail/flightDetailUtils";

const businessQuery = {
  adults: "1",
  cabin: "business",
  children: "1",
  departure: "2099-05-10",
  from: "BKK",
  infants: "0",
  return: "2099-05-18",
  to: "LHR",
  trip: "round-trip",
} as const;

const renderDetail = (
  flightId = "xf-201",
  query: Record<string, string> = businessQuery,
) => {
  const request = resolveFlightDetailRequest(flightId, query);
  if (!request) throw new Error("Expected a valid detail fixture");

  return render(<FlightDetailPage {...request} />);
};

describe("FlightDetailPage", () => {
  it("renders the selected route, schedule, itinerary context, and secondary aircraft summary", () => {
    renderDetail();

    expect(
      screen.getByRole("heading", { level: 1, name: /BKK.*LHR/ }),
    ).toBeInTheDocument();
    expect(screen.getByText("XF 201")).toBeInTheDocument();
    expect(screen.getByText("09:15")).toBeInTheDocument();
    expect(screen.getByText("16:40")).toBeInTheDocument();
    expect(screen.getByText("Bangkok")).toBeInTheDocument();
    expect(screen.getByText("London")).toBeInTheDocument();
    expect(screen.getByText("13h 25m")).toBeInTheDocument();
    expect(screen.getByText("Direct")).toBeInTheDocument();
    expect(screen.getByText("Airbus A350-900")).toBeInTheDocument();
    expect(screen.getByText("On schedule")).toBeInTheDocument();
    expect(screen.getByText("10 May 2099")).toBeInTheDocument();
    expect(screen.getByText("Return 18 May 2099")).toBeInTheDocument();
    expect(screen.getByText("2 passengers")).toBeInTheDocument();
    expect(screen.getByText(/all times shown in local airport time/i)).toBeInTheDocument();
  });

  it("selects the searched cabin by default and displays all four cabin choices", () => {
    renderDetail();

    const tabs = screen.getAllByRole("tab");
    expect(tabs).toHaveLength(4);
    expect(screen.getByRole("tab", { name: /^economy$/i })).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: /premium economy/i }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: /business.*your searched cabin/i }),
    ).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tab", { name: /^first$/i })).toBeInTheDocument();
    expect(screen.getByText("THB 68,900")).toBeInTheDocument();
    expect(screen.getByText("Lie-flat seat")).toBeInTheDocument();
    expect(screen.getByRole("img", { name: /business class/i })).toHaveAttribute(
      "src",
      expect.stringContaining("x-fly-cabin-business-v1.png"),
    );
  });

  it("switches cabin imagery, content, pricing, and seat handoff without changing the searched cabin", async () => {
    renderDetail();

    const economyTab = screen.getByRole("tab", { name: /^economy$/i });
    fireEvent.mouseDown(economyTab, { button: 0, ctrlKey: false });
    fireEvent.click(economyTab);

    await waitFor(() =>
      expect(screen.getByRole("tab", { name: /^economy$/i })).toHaveAttribute(
        "aria-selected",
        "true",
      ),
    );
    expect(
      screen.getByRole("tab", { name: /business.*your searched cabin/i }),
    ).toHaveAttribute("aria-selected", "false");
    expect(screen.getByText("THB 21,900")).toBeInTheDocument();
    expect(screen.getByText("Practical comfort")).toBeInTheDocument();
    expect(screen.getByRole("img", { name: /economy cabin seating/i })).toHaveAttribute(
      "src",
      expect.stringContaining("x-fly-cabin-economy-v1.png"),
    );

    const seatLink = screen.getByRole("link", {
      name: "Choose your seat in Economy on flight XF 201",
    });
    expect(seatLink).toHaveAttribute(
      "href",
      "/flights/xf-201/seats?from=BKK&to=LHR&departure=2099-05-10&return=2099-05-18&adults=1&children=1&infants=0&cabin=business&trip=round-trip&selectedCabin=economy",
    );
  });

  it("restores the active preview cabin while retaining the searched-cabin marker", () => {
    renderDetail("xf-201", { ...businessQuery, selectedCabin: "first" });

    expect(screen.getByRole("tab", { name: /^first$/i })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(
      screen.getByRole("tab", { name: /business.*your searched cabin/i }),
    ).toHaveAttribute("aria-selected", "false");
    expect(screen.getByText("THB 114,900")).toBeInTheDocument();
  });

  it("supports keyboard cabin navigation", async () => {
    renderDetail();

    const businessTab = screen.getByRole("tab", {
      name: /business.*your searched cabin/i,
    });
    act(() => businessTab.focus());
    fireEvent.keyDown(businessTab, { key: "ArrowRight" });

    await waitFor(() =>
      expect(screen.getByRole("tab", { name: /^first$/i })).toHaveAttribute(
        "aria-selected",
        "true",
      ),
    );
    expect(screen.getByText("THB 114,900")).toBeInTheDocument();
  });

  it("keeps an unavailable cabin inspectable and prevents continuing with it", async () => {
    renderDetail("xf-315");

    const firstTab = screen.getByRole("tab", { name: /first.*unavailable/i });
    expect(within(firstTab).getByText("Unavailable")).toBeInTheDocument();
    fireEvent.mouseDown(firstTab, { button: 0, ctrlKey: false });
    fireEvent.click(firstTab);

    expect(
      await screen.findByText("Not available on this flight"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", {
        name: "Choose your seat in First on flight XF 315, unavailable",
      }),
    ).toBeDisabled();
    expect(
      screen.queryByRole("link", { name: /choose your seat in first/i }),
    ).not.toBeInTheDocument();
  });

  it("preserves the complete search query when returning to flights", () => {
    renderDetail();

    expect(screen.getByRole("link", { name: "Back to Flights" })).toHaveAttribute(
      "href",
      "/flights?from=BKK&to=LHR&departure=2099-05-10&return=2099-05-18&adults=1&children=1&infants=0&cabin=business&trip=round-trip",
    );
  });

  it("does not render a seat map or make a network request", () => {
    const originalFetch = global.fetch;
    const fetchMock = jest.fn();
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: fetchMock,
      writable: true,
    });

    try {
      const { container } = renderDetail();

      expect(container.querySelector("[data-seat-map]")).not.toBeInTheDocument();
      expect(container.querySelector("[data-seat]")).not.toBeInTheDocument();
      expect(screen.queryByRole("grid", { name: /seat/i })).not.toBeInTheDocument();
      expect(fetchMock).not.toHaveBeenCalled();

      const cabinSection = screen.getByRole("region", {
        name: "Cabin experience",
      });
      expect(within(cabinSection).getAllByRole("tab")).toHaveLength(4);
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
});
