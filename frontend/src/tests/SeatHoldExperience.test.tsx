import { fireEvent, screen, waitFor } from "@testing-library/react";
import { render } from "@/tests/renderWithLanguage";

import { resolveSeatSelectionRequest } from "@/components/booking/detail/flightDetailUtils";
import { SeatMapPage } from "@/components/booking/seats/SeatMapPage";
import { getSeatMapFixture } from "@/components/booking/seats/seatMapFixtures";

const request = resolveSeatSelectionRequest("xf-201", {
  adults: "1",
  cabin: "economy",
  children: "0",
  departure: "2099-05-10",
  from: "BKK",
  infants: "0",
  selectedCabin: "economy",
  to: "LHR",
  trip: "one-way",
});
if (!request) throw new Error("Expected valid seat selection request");
const seatMap = getSeatMapFixture(request.flight.aircraft, "economy");
if (!seatMap) throw new Error("Expected seat map fixture");

const inventory = {
  cabin: "economy",
  departureDate: "2099-05-10",
  flightId: "xf-201",
  seats: seatMap.rows.flatMap((row) =>
    row.groups.flat().map((seat) => ({
      columnCode: seat.column,
      position: seat.position,
      rowNumber: seat.row,
      seatNumber: seat.seatNumber,
      status: seat.availability === "booked" ? "BOOKED" : "AVAILABLE",
    })),
  ),
  serverTime: "2099-05-10T10:00:00Z",
};

const hold = {
  cabin: "economy",
  departureDate: "2099-05-10",
  expiresAt: "2099-05-10T10:10:00Z",
  flightId: "xf-201",
  id: "8d256f1e-4758-4997-861f-3f20a53c5846",
  passengers: { adults: 1, children: 0, infants: 0 },
  seats: ["20A"],
  serverTime: "2099-05-10T10:00:00Z",
};

const response = (payload: unknown, status = 200) => ({
  json: async () => payload,
  ok: status >= 200 && status < 300,
  status,
});

describe("server-authoritative seat hold experience", () => {
  const originalFetch = global.fetch;

  beforeEach(() => {
    window.sessionStorage.clear();
  });

  afterEach(() => {
    jest.useRealTimers();
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

  it("shows optimistic pending state then confirms the server hold and countdown", async () => {
    let resolveHold!: (value: ReturnType<typeof response>) => void;
    const pendingHold = new Promise<ReturnType<typeof response>>((resolve) => {
      resolveHold = resolve;
    });
    const fetchMock = jest
      .fn()
      .mockResolvedValueOnce(response(inventory))
      .mockReturnValueOnce(pendingHold)
      .mockResolvedValue(response(inventory));
    Object.defineProperty(global, "fetch", { configurable: true, value: fetchMock });

    const { container } = render(<SeatMapPage request={request} seatMap={seatMap} />);
    const seat = await screen.findByRole("button", { name: /Seat 20A.*available/i });
    fireEvent.click(seat);

    expect(container.querySelector('[data-hold-state="pending"]')).toBeInTheDocument();
    expect(screen.getByText("1 of 1 seat selected")).toBeInTheDocument();

    resolveHold(response(hold, 201));
    expect(await screen.findByText(/Seats held for 10:00/)).toBeInTheDocument();
    expect(seat).toHaveAccessibleName(/selected/i);
  });

  it("rolls back an optimistic seat after a 409 and refreshes availability", async () => {
    const fetchMock = jest
      .fn()
      .mockResolvedValueOnce(response(inventory))
      .mockResolvedValueOnce(
        response(
          {
            error: {
              code: "SEAT_UNAVAILABLE",
              conflictingSeats: ["20A"],
              message: "One or more seats were just taken.",
            },
          },
          409,
        ),
      )
      .mockResolvedValue(response({
        ...inventory,
        seats: inventory.seats.map((seat) =>
          seat.seatNumber === "20A" ? { ...seat, status: "UNAVAILABLE" } : seat,
        ),
      }));
    Object.defineProperty(global, "fetch", { configurable: true, value: fetchMock });

    render(<SeatMapPage request={request} seatMap={seatMap} />);
    fireEvent.click(await screen.findByRole("button", { name: /Seat 20A.*available/i }));

    expect(await screen.findByText("That seat was just taken. Choose another seat.")).toBeInTheDocument();
    expect(screen.getByText("0 of 1 seat selected")).toBeInTheDocument();
    await waitFor(() => expect(screen.getByRole("button", { name: /Seat 20A.*unavailable/i })).toBeDisabled());
  });

  it("blocks Continue when authoritative revalidation reports an expired hold", async () => {
    const fetchMock = jest
      .fn()
      .mockResolvedValueOnce(response(inventory))
      .mockResolvedValueOnce(response(hold, 201))
      .mockResolvedValueOnce(response(inventory))
      .mockResolvedValueOnce(
        response({ error: { code: "HOLD_EXPIRED", message: "Expired" } }, 410),
      )
      .mockResolvedValue(response(inventory));
    Object.defineProperty(global, "fetch", { configurable: true, value: fetchMock });

    render(<SeatMapPage request={request} seatMap={seatMap} />);
    fireEvent.click(await screen.findByRole("button", { name: /Seat 20A.*available/i }));
    const continueButton = await screen.findByRole("button", { name: "Continue with 1 seat" });
    fireEvent.click(continueButton);

    expect(await screen.findByText("Your seat hold expired. Select your seat again.")).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: /Continue with 1 seat/ })).not.toBeInTheDocument();
  });
});
