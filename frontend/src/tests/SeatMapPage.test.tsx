import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { render } from "@/tests/renderWithLanguage";

import { resolveSeatSelectionRequest } from "@/components/booking/detail/flightDetailUtils";
import { SeatMapPage } from "@/components/booking/seats/SeatMapPage";
import { getSeatMapFixture } from "@/components/booking/seats/seatMapFixtures";
import type { CabinClass } from "@/components/booking/search/searchTypes";

const baseQuery = {
  adults: "2",
  cabin: "business",
  children: "1",
  departure: "2099-05-10",
  from: "BKK",
  infants: "1",
  return: "2099-05-18",
  to: "LHR",
  trip: "round-trip",
} as const;

const renderSeatMap = (selectedCabin: CabinClass = "business") => {
  const request = resolveSeatSelectionRequest("xf-201", {
    ...baseQuery,
    selectedCabin,
  });
  if (!request) throw new Error("Expected a valid seat-selection request");

  const seatMap = getSeatMapFixture(request.flight.aircraft, selectedCabin);
  if (!seatMap) throw new Error("Expected a matching local seat map");

  return render(<SeatMapPage request={request} seatMap={seatMap} />);
};

const response = (payload: unknown, status = 200) => ({
  json: async () => payload,
  ok: status >= 200 && status < 300,
  status,
});

const installSuccessfulSeatHoldFetch = () => {
  let currentHold: Record<string, unknown> | null = null;
  const fetchMock = jest.fn(async (input: string | URL | Request, init?: RequestInit) => {
    const url = String(input);
    if (url.includes("/flights/")) {
      const cabin = new URL(url).searchParams.get("cabin") as CabinClass;
      const fixture = getSeatMapFixture("Airbus A350-900", cabin);
      return response({
        cabin,
        departureDate: "2099-05-10",
        flightId: "xf-201",
        seats: fixture?.rows.flatMap((row) =>
          row.groups.flat().map((seat) => ({
            columnCode: seat.column,
            position: seat.position,
            rowNumber: seat.row,
            seatNumber: seat.seatNumber,
            status:
              seat.availability === "booked"
                ? "BOOKED"
                : seat.availability === "unavailable"
                  ? "UNAVAILABLE"
                  : "AVAILABLE",
          })),
        ) ?? [],
        serverTime: "2099-05-10T10:00:00Z",
      });
    }
    if (url.endsWith("/validation")) return response(currentHold);
    if (init?.method === "POST" || init?.method === "PUT") {
      const body = JSON.parse(String(init.body)) as {
        cabin?: CabinClass;
        departureDate?: string;
        flightId?: string;
        passengers?: { adults: number; children: number; infants: number };
        seats: string[];
      };
      currentHold = {
        cabin: body.cabin ?? currentHold?.cabin ?? "business",
        departureDate: body.departureDate ?? currentHold?.departureDate ?? "2099-05-10",
        expiresAt: "2099-05-10T10:10:00Z",
        flightId: body.flightId ?? currentHold?.flightId ?? "xf-201",
        id: "8d256f1e-4758-4997-861f-3f20a53c5846",
        passengers:
          body.passengers ?? currentHold?.passengers ?? { adults: 2, children: 1, infants: 1 },
        seats: body.seats,
        serverTime: "2099-05-10T10:00:00Z",
      };
      return response(currentHold, init.method === "POST" ? 201 : 200);
    }
    if (init?.method === "DELETE") return response(null, 204);
    return response(currentHold);
  });
  Object.defineProperty(global, "fetch", {
    configurable: true,
    value: fetchMock,
    writable: true,
  });
  return fetchMock;
};

describe("SeatMapPage", () => {
  const originalFetch = global.fetch;

  beforeEach(() => window.sessionStorage.clear());

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
  it("renders validated flight, route, selected cabin, and seat requirement context", () => {
    renderSeatMap();

    expect(
      screen.getByRole("heading", { level: 1, name: "Select your seat" }),
    ).toBeInTheDocument();
    expect(screen.getByText("XF 201 · BKK → LHR")).toBeInTheDocument();
    expect(screen.getAllByText("Business").length).toBeGreaterThan(0);
    expect(screen.getByText("4 passengers · 1 lap infant")).toBeInTheDocument();
    expect(screen.getByText("Select 3 seats")).toBeInTheDocument();
    expect(screen.getByText("Aircraft front")).toBeInTheDocument();
  });

  it.each([
    ["economy", "economy"],
    ["premium-economy", "premium-economy"],
    ["business", "business"],
    ["first", "first"],
  ] as const)("renders the distinct %s seat model", (cabin, model) => {
    const { container } = renderSeatMap(cabin);

    expect(container.querySelector(`[data-seat-model="${model}"]`)).toBeInTheDocument();
  });

  it.each([
    ["economy", "conventional", ["headrest", "backrest", "seat-pan", "armrests"]],
    ["premium-economy", "wide-recliner", ["headrest", "padded-backrest", "seat-pan", "armrests"]],
    ["business", "lie-flat-pod", ["privacy-shell", "seat", "console"]],
    ["first", "private-suite", ["suite-shell", "armchair", "console"]],
  ] as const)(
    "renders %s with the %s top-view aircraft-seat form",
    (cabin, form, expectedParts) => {
      const { container } = renderSeatMap(cabin);
      const visual = container.querySelector(
        `[data-seat-form="${form}"]`,
      );

      expect(visual).toBeInTheDocument();
      expect(
        Array.from(visual?.querySelectorAll("[data-seat-part]") ?? []).map(
          (part) => part.getAttribute("data-seat-part"),
        ),
      ).toEqual(expectedParts);
    },
  );

  it.each([
    ["first", "foremost", "Foremost cabin", "3", "19"],
    ["business", "forward", "Forward cabin", "19", "44"],
    ["premium-economy", "central", "Central cabin", "44", "61"],
    ["economy", "mid-rear", "Mid-rear cabin", "61", "94"],
  ] as const)(
    "places %s in the shared aircraft %s section",
    (cabin, location, sectionLabel, sectionStart, sectionEnd) => {
      const { container } = renderSeatMap(cabin);

      expect(
        container.querySelector('[data-aircraft-family="x-fly-widebody-master"]'),
      ).toBeInTheDocument();
      expect(
        screen.getByRole("img", {
          name: new RegExp(`X-Fly Widebody aircraft overview.*${sectionLabel}`, "i"),
        }),
      ).toBeInTheDocument();
      const highlight = container.querySelector(
        `[data-current-section="${location}"]`,
      );
      expect(highlight).toHaveAttribute("data-section-start", sectionStart);
      expect(highlight).toHaveAttribute("data-section-end", sectionEnd);
      expect(screen.getByText(sectionLabel)).toBeInTheDocument();
    },
  );

  it("renders a recognizable widebody overview with distinct flight surfaces", () => {
    const { container } = renderSeatMap("premium-economy");
    const overview = screen.getByRole("img", {
      name: /X-Fly Widebody aircraft overview.*Central cabin/i,
    });

    expect(
      overview.querySelector('[data-aircraft-silhouette="widebody"]'),
    ).toHaveAttribute("data-aircraft-profile", "tapered-widebody");
    expect(overview.querySelector('[data-aircraft-part="fuselage"]')).toHaveAttribute(
      "data-aft-profile",
      "rounded-taper",
    );
    expect(overview.querySelector('[data-aircraft-part="main-wing-left"]')).toHaveAttribute(
      "data-wing-planform",
      "asymmetric-swept",
    );
    expect(overview.querySelector('[data-aircraft-part="main-wing-right"]')).toHaveAttribute(
      "data-wing-planform",
      "asymmetric-swept",
    );
    expect(overview.querySelector('[data-aircraft-part="tailplane-left"]')).toHaveAttribute(
      "data-flight-surface-scale",
      "secondary",
    );
    expect(overview.querySelector('[data-aircraft-part="tailplane-right"]')).toHaveAttribute(
      "data-flight-surface-scale",
      "secondary",
    );
    expect(
      container.querySelector('[data-current-section="central"]'),
    ).toHaveAttribute("data-highlight-clip", "fuselage");
  });

  it("renders restrained, section-specific aircraft plan context", () => {
    renderSeatMap("premium-economy");

    expect(screen.getByText("Cabin bulkhead")).toBeInTheDocument();
    expect(screen.getByText("Wing leading area")).toBeInTheDocument();
    expect(screen.getByText("Overwing exit")).toBeInTheDocument();
    expect(screen.queryByText("Rear galley")).not.toBeInTheDocument();
  });

  it.each([
    ["business", "true"],
    ["first", "false"],
    ["premium-economy", "false"],
    ["economy", "false"],
  ] as const)(
    "marks contained horizontal scrolling for %s as %s",
    (cabin, expected) => {
      const { container } = renderSeatMap(cabin);

      expect(container.querySelector("[data-seat-map]")).toHaveAttribute(
        "data-contained-scroll",
        expected,
      );
    },
  );

  it("renders row markers and descriptive native seat buttons", () => {
    renderSeatMap("economy");

    expect(screen.getByText("20", { selector: "[data-row-number]" })).toBeInTheDocument();
    const availableSeat = screen.getAllByRole("button", {
      name: /Seat \d+[A-Z], Economy, (window|middle|aisle), available/i,
    })[0];
    expect(availableSeat?.tagName).toBe("BUTTON");
    expect(availableSeat).toHaveAttribute("type", "button");
  });

  it("selects and deselects an available seat and exposes the state semantically", async () => {
    installSuccessfulSeatHoldFetch();
    renderSeatMap();
    const seat = screen.getAllByRole("button", { name: /, available$/i })[0];
    if (!seat) throw new Error("Expected an available seat");

    fireEvent.click(seat);
    expect(seat).toHaveAttribute("aria-pressed", "true");
    expect(seat).toHaveAccessibleName(/selected/i);
    expect(screen.getByText("1 of 3 seats selected")).toBeInTheDocument();

    await waitFor(() =>
      expect(screen.getByRole("complementary")).toHaveAttribute(
        "data-hold-state",
        "confirmed",
      ),
    );

    fireEvent.click(seat);
    await waitFor(() => expect(seat).toHaveAttribute("aria-pressed", "false"));
    expect(screen.getByText("0 of 3 seats selected")).toBeInTheDocument();
  });

  it("prevents booked and unavailable seats from being selected", () => {
    renderSeatMap();
    const booked = screen.getAllByRole("button", { name: /booked/i })[0];
    const unavailable = screen.getAllByRole("button", { name: /unavailable/i })[0];

    expect(booked).toBeDisabled();
    expect(unavailable).toBeDisabled();
    fireEvent.click(booked as HTMLButtonElement);
    fireEvent.click(unavailable as HTMLButtonElement);
    expect(screen.getByText("0 of 3 seats selected")).toBeInTheDocument();
  });

  it("does not replace a selection after reaching the passenger limit", async () => {
    installSuccessfulSeatHoldFetch();
    renderSeatMap();
    const available = screen.getAllByRole("button", { name: /, available$/i });

    for (const seat of available.slice(0, 3)) {
      fireEvent.click(seat);
      await waitFor(() =>
        expect(screen.getByRole("complementary")).toHaveAttribute(
          "data-hold-state",
          "confirmed",
        ),
      );
    }
    fireEvent.click(available[3] as HTMLButtonElement);

    expect(screen.getByText("3 of 3 seats selected")).toBeInTheDocument();
    expect(
      screen.getByText("Deselect a seat before choosing another."),
    ).toBeInTheDocument();
    expect(available[3]).toHaveAttribute("aria-pressed", "false");
  });

  it("enables continuation only after a server hold and revalidates before handoff", async () => {
    const fetchMock = installSuccessfulSeatHoldFetch();
    renderSeatMap();
    expect(screen.getByRole("button", { name: /continue.*select 3 seats/i })).toBeDisabled();

    const available = screen.getAllByRole("button", { name: /, available$/i });
    for (const seat of available.slice(0, 3)) {
      fireEvent.click(seat);
      await waitFor(() =>
        expect(screen.getByRole("complementary")).toHaveAttribute(
          "data-hold-state",
          "confirmed",
        ),
      );
    }

    const continueButton = screen.getByRole("button", { name: "Continue with 3 seats" });
    expect(continueButton).toBeEnabled();
    fireEvent.click(continueButton);
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        expect.stringMatching(/\/seat-holds\/8d256f1e/),
        expect.objectContaining({ credentials: "include" }),
      ),
    );
  });

  it("preserves the active cabin when returning to Flight Detail", () => {
    renderSeatMap("first");

    expect(screen.getByRole("link", { name: "Back to Flight" })).toHaveAttribute(
      "href",
      "/flights/xf-201?from=BKK&to=LHR&departure=2099-05-10&return=2099-05-18&adults=2&children=1&infants=1&cabin=business&trip=round-trip&selectedCabin=first#cabin-experience",
    );
  });

  it("uses native focusable controls and requests authoritative availability", () => {
    const originalFetch = global.fetch;
    const fetchMock = jest.fn(() => new Promise(() => {}));
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: fetchMock,
      writable: true,
    });

    try {
      const { container } = renderSeatMap();
      const map = within(container.querySelector("[data-seat-map]") as HTMLElement);
      const seat = map.getAllByRole("button", { name: /, available$/i })[0];
      seat?.focus();

      expect(seat).toHaveFocus();
      expect(container.querySelector('[role="grid"]')).not.toBeInTheDocument();
      expect(fetchMock).toHaveBeenCalledWith(
        expect.stringContaining("/api/v1/flights/xf-201/seats"),
        expect.objectContaining({ credentials: "include" }),
      );
      expect(screen.queryByRole("form")).not.toBeInTheDocument();
      expect(screen.queryByText(/held for/i)).not.toBeInTheDocument();
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

  it("shows an explicit safe state when a valid request has no local map", () => {
    const request = resolveSeatSelectionRequest("xf-201", {
      ...baseQuery,
      selectedCabin: "business",
    });
    if (!request) throw new Error("Expected a valid seat-selection request");

    render(<SeatMapPage request={request} seatMap={null} />);

    expect(screen.getByText("Seat map unavailable")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /continue/i })).not.toBeInTheDocument();
  });

  it.each(["5", "99"])(
    "shows an explicit safe state when %s passengers exceed local fixture availability",
    (adults) => {
      const request = resolveSeatSelectionRequest("xf-201", {
        ...baseQuery,
        adults,
        children: adults === "99" ? "99" : "0",
        infants: "0",
        selectedCabin: "first",
      });
      if (!request) throw new Error("Expected a valid seat-selection request");
      const seatMap = getSeatMapFixture(request.flight.aircraft, "first");

      render(<SeatMapPage request={request} seatMap={seatMap} />);

      expect(screen.getByText("Not enough fixture seats")).toBeInTheDocument();
      expect(screen.queryByRole("button", { name: /^Seat /i })).not.toBeInTheDocument();
      expect(screen.queryByRole("button", { name: /continue/i })).not.toBeInTheDocument();
    },
  );
});
