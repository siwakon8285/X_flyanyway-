import { fireEvent, render, screen } from "@testing-library/react";

import { FlightDetailPage } from "@/components/booking/detail/FlightDetailPage";
import {
  resolveFlightDetailRequest,
  resolveSeatSelectionRequest,
} from "@/components/booking/detail/flightDetailUtils";
import { FlightResultsPage } from "@/components/booking/results/FlightResultsPage";
import { SeatMapPage } from "@/components/booking/seats/SeatMapPage";
import { getSeatMapFixture } from "@/components/booking/seats/seatMapFixtures";
import { LanguageToggle } from "@/components/layout/LanguageToggle";
import { LanguageProvider } from "@/i18n/LanguageProvider";

const query = {
  adults: "2",
  cabin: "business",
  children: "1",
  departure: "2099-05-10",
  from: "BKK",
  infants: "0",
  return: "2099-05-18",
  selectedCabin: "business",
  to: "LHR",
  trip: "round-trip",
} as const;

describe("language changes preserve mounted page state", () => {
  it("preserves Results sort state", () => {
    const request = resolveFlightDetailRequest("xf-201", query);
    if (!request) throw new Error("Expected fixture request");

    render(
      <LanguageProvider initialLocale="en">
        <LanguageToggle />
        <FlightResultsPage criteria={request.criteria} query={request.query} />
      </LanguageProvider>,
    );

    const sort = screen.getByLabelText("Sort flights");
    fireEvent.change(sort, { target: { value: "price" } });
    expect(sort).toHaveValue("price");

    fireEvent.click(screen.getByRole("button", { name: /Current language/ }));

    expect(screen.getByLabelText("เรียงเที่ยวบิน")).toHaveValue("price");
    expect(screen.getAllByRole("article")[0]).toHaveTextContent("XF 315");
  });

  it("preserves the active Flight Detail cabin", async () => {
    const request = resolveFlightDetailRequest("xf-201", query);
    if (!request) throw new Error("Expected fixture request");

    render(
      <LanguageProvider initialLocale="en">
        <LanguageToggle />
        <FlightDetailPage {...request} />
      </LanguageProvider>,
    );

    const firstCabin = screen.getByRole("tab", { name: "First" });
    fireEvent.mouseDown(firstCabin, { button: 0, ctrlKey: false });
    fireEvent.click(firstCabin);
    expect(firstCabin).toHaveAttribute("aria-selected", "true");
    await screen.findByRole("img", { name: /First Class private suite/i });

    fireEvent.click(screen.getByRole("button", { name: /Current language/ }));

    expect(screen.getByRole("tab", { name: "ชั้นหนึ่ง" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  it("preserves selected seats", async () => {
    const request = resolveSeatSelectionRequest("xf-201", query);
    if (!request) throw new Error("Expected seat request");
    const seatMap = getSeatMapFixture(request.flight.aircraft, "business");
    if (!seatMap) throw new Error("Expected seat map fixture");

    const inventory = {
      cabin: "business",
      departureDate: query.departure,
      flightId: request.flight.id,
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
      cabin: "business",
      departureDate: query.departure,
      expiresAt: "2099-05-10T10:10:00Z",
      flightId: request.flight.id,
      id: "8d256f1e-4758-4997-861f-3f20a53c5846",
      passengers: request.criteria.passengers,
      seats: ["3A"],
      serverTime: "2099-05-10T10:00:00Z",
    };
    const response = (body: unknown, status = 200) => ({
      json: async () => body,
      ok: status >= 200 && status < 300,
      status,
    });
    const originalFetch = global.fetch;
    const fetchMock = jest
      .fn()
      .mockResolvedValueOnce(response(inventory))
      .mockResolvedValueOnce(response(hold, 201))
      .mockResolvedValue(response(inventory));
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: fetchMock,
      writable: true,
    });

    try {
      render(
        <LanguageProvider initialLocale="en">
          <LanguageToggle />
          <SeatMapPage request={request} seatMap={seatMap} />
        </LanguageProvider>,
      );

      const seat = await screen.findByRole("button", { name: /Seat 3A.*available/i });
      fireEvent.click(seat);
      expect(seat).toHaveAttribute("aria-pressed", "true");

      fireEvent.click(screen.getByRole("button", { name: /Current language/ }));

      expect(await screen.findByText("ยืนยันการสงวนที่นั่งแล้ว")).toBeInTheDocument();
      expect(seat).toHaveAttribute("aria-pressed", "true");
      expect(screen.getByText("เลือกแล้ว 1 จาก 3 ที่")).toBeInTheDocument();
    } finally {
      Object.defineProperty(global, "fetch", {
        configurable: true,
        value: originalFetch,
        writable: true,
      });
    }
  });
});
