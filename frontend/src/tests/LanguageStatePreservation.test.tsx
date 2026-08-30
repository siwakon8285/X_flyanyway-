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

  it("preserves the active Flight Detail cabin", () => {
    const request = resolveFlightDetailRequest("xf-201", query);
    if (!request) throw new Error("Expected fixture request");

    render(
      <LanguageProvider initialLocale="en">
        <LanguageToggle />
        <FlightDetailPage {...request} />
      </LanguageProvider>,
    );

    const economyCabin = screen.getByRole("tab", { name: "Economy" });
    fireEvent.mouseDown(economyCabin, { button: 0, ctrlKey: false });
    fireEvent.click(economyCabin);
    expect(economyCabin).toHaveAttribute("aria-selected", "true");

    fireEvent.click(screen.getByRole("button", { name: /Current language/ }));

    expect(screen.getByRole("tab", { name: "ชั้นประหยัด" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  it("preserves selected seats", () => {
    const request = resolveSeatSelectionRequest("xf-201", query);
    if (!request) throw new Error("Expected seat request");
    const seatMap = getSeatMapFixture(request.flight.aircraft, "business");
    if (!seatMap) throw new Error("Expected seat map fixture");

    render(
      <LanguageProvider initialLocale="en">
        <LanguageToggle />
        <SeatMapPage request={request} seatMap={seatMap} />
      </LanguageProvider>,
    );

    const seat = screen.getAllByRole("button", { name: /, available$/i })[0];
    fireEvent.click(seat);
    expect(seat).toHaveAttribute("aria-pressed", "true");

    fireEvent.click(screen.getByRole("button", { name: /Current language/ }));

    expect(seat).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByText("เลือกแล้ว 1 จาก 3 ที่")).toBeInTheDocument();
  });
});
