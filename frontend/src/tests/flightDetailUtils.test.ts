import {
  buildSeatSelectionHref,
  resolveFlightDetailRequest,
  resolveSeatSelectionRequest,
} from "@/components/booking/detail/flightDetailUtils";

const validQuery = {
  adults: "1",
  cabin: "business",
  children: "0",
  departure: "2099-05-10",
  from: "BKK",
  infants: "0",
  return: "2099-05-18",
  to: "LHR",
  trip: "round-trip",
} as const;

describe("flight detail request utilities", () => {
  it("resolves a matching fixture from complete canonical search criteria", () => {
    const result = resolveFlightDetailRequest("xf-201", validQuery);

    expect(result?.flight.id).toBe("xf-201");
    expect(result?.criteria.cabin).toBe("business");
    expect(result?.query).toBe(
      "from=BKK&to=LHR&departure=2099-05-10&return=2099-05-18&adults=1&children=0&infants=0&cabin=business&trip=round-trip",
    );
  });

  it("rejects unknown flights, malformed criteria, route mismatches, and unavailable searched cabins", () => {
    expect(resolveFlightDetailRequest("unknown", validQuery)).toBeNull();
    expect(
      resolveFlightDetailRequest("xf-201", { ...validQuery, adults: "zero" }),
    ).toBeNull();
    expect(
      resolveFlightDetailRequest("xf-621", validQuery),
    ).toBeNull();
    expect(
      resolveFlightDetailRequest("xf-315", { ...validQuery, cabin: "first" }),
    ).toBeNull();
  });

  it("builds a seat handoff that preserves the searched cabin and adds the active preview cabin", () => {
    expect(
      buildSeatSelectionHref({
        flightId: "xf-201",
        query:
          "from=BKK&to=LHR&departure=2099-05-10&return=2099-05-18&adults=1&children=0&infants=0&cabin=business&trip=round-trip",
        selectedCabin: "first",
      }),
    ).toBe(
      "/flights/xf-201/seats?from=BKK&to=LHR&departure=2099-05-10&return=2099-05-18&adults=1&children=0&infants=0&cabin=business&trip=round-trip&selectedCabin=first",
    );
  });

  it("accepts only an available selected cabin at the seat boundary", () => {
    expect(
      resolveSeatSelectionRequest("xf-201", {
        ...validQuery,
        selectedCabin: "first",
      })?.selectedCabin,
    ).toBe("first");
    expect(
      resolveSeatSelectionRequest("xf-315", {
        ...validQuery,
        selectedCabin: "first",
      }),
    ).toBeNull();
    expect(
      resolveSeatSelectionRequest("xf-201", {
        ...validQuery,
        selectedCabin: "spaceship",
      }),
    ).toBeNull();
  });
});
