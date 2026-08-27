import { AIRPORT_FIXTURES } from "@/components/booking/search/airportFixtures";
import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import { FLIGHT_RESULT_FIXTURES } from "@/components/booking/results/flightResultFixtures";
import {
  filterFlightResults,
  formatDuration,
  getCabinPrice,
  sortFlightResults,
} from "@/components/booking/results/flightResultUtils";

const criteria: FlightSearchFormValues = {
  cabin: "premium-economy",
  departure: "2099-05-10",
  from: AIRPORT_FIXTURES[0],
  passengers: { adults: 1, children: 1, infants: 0 },
  returnDate: "2099-05-18",
  to: AIRPORT_FIXTURES[2],
  trip: "round-trip",
};

const demoRoutes = [
  ["BKK", "LHR"],
  ["BKK", "HND"],
  ["BKK", "DXB"],
  ["HND", "BKK"],
  ["LHR", "BKK"],
  ["JFK", "LHR"],
] as const;
const cabinClasses = [
  "economy",
  "premium-economy",
  "business",
  "first",
] as const;

const createCriteria = (
  originCode: string,
  destinationCode: string,
  cabin: FlightSearchFormValues["cabin"],
): FlightSearchFormValues => ({
  ...criteria,
  cabin,
  from:
    AIRPORT_FIXTURES.find((airport) => airport.code === originCode) ?? null,
  to:
    AIRPORT_FIXTURES.find((airport) => airport.code === destinationCode) ?? null,
});

describe("flight result utilities", () => {
  it("formats raw duration minutes for customers", () => {
    expect(formatDuration(805)).toBe("13h 25m");
    expect(formatDuration(60)).toBe("1h");
  });

  it("returns only fixtures matching the sanitized route and searched cabin", () => {
    const matches = filterFlightResults(FLIGHT_RESULT_FIXTURES, criteria);

    expect(matches.length).toBeGreaterThan(1);
    expect(matches.every((flight) => flight.originCode === "BKK")).toBe(true);
    expect(matches.every((flight) => flight.destinationCode === "LHR")).toBe(true);
    expect(matches.every((flight) => getCabinPrice(flight, criteria.cabin) !== null)).toBe(
      true,
    );
  });

  it("sorts by recommendation, departure, duration, and searched-cabin price", () => {
    const matches = filterFlightResults(FLIGHT_RESULT_FIXTURES, criteria);

    expect(sortFlightResults(matches, "recommended", criteria.cabin)[0]?.id).toBe(
      "xf-201",
    );
    expect(sortFlightResults(matches, "departure", criteria.cabin)[0]?.id).toBe(
      "xf-315",
    );
    expect(sortFlightResults(matches, "duration", criteria.cabin)[0]?.id).toBe(
      "xf-428",
    );
    expect(sortFlightResults(matches, "price", criteria.cabin)[0]?.id).toBe(
      "xf-315",
    );
  });

  it.each(demoRoutes)(
    "provides 2–4 demo flights from %s to %s with every cabin represented",
    (originCode, destinationCode) => {
      const routeFixtures = FLIGHT_RESULT_FIXTURES.filter(
        (flight) =>
          flight.originCode === originCode &&
          flight.destinationCode === destinationCode,
      );

      expect(routeFixtures.length).toBeGreaterThanOrEqual(2);
      expect(routeFixtures.length).toBeLessThanOrEqual(4);
      cabinClasses.forEach((cabin) => {
        expect(
          filterFlightResults(
            FLIGHT_RESULT_FIXTURES,
            createCriteria(originCode, destinationCode, cabin),
          ).length,
        ).toBeGreaterThan(0);
      });
    },
  );

  it("keeps fixture IDs globally unique and excludes Moon routes", () => {
    const ids = FLIGHT_RESULT_FIXTURES.map((flight) => flight.id);

    expect(new Set(ids).size).toBe(ids.length);
    expect(
      FLIGHT_RESULT_FIXTURES.some(
        (flight) =>
          flight.originCode.toLowerCase().includes("moon") ||
          flight.destinationCode.toLowerCase().includes("moon"),
      ),
    ).toBe(false);
  });

  it("keeps DXB to JFK as a valid route with no fixture results", () => {
    cabinClasses.forEach((cabin) => {
      expect(
        filterFlightResults(
          FLIGHT_RESULT_FIXTURES,
          createCriteria("DXB", "JFK", cabin),
        ),
      ).toHaveLength(0);
    });
  });

  it("matches and sorts fixture data without making a network request", () => {
    const originalFetch = global.fetch;
    const fetchMock = jest.fn();
    Object.defineProperty(global, "fetch", {
      configurable: true,
      value: fetchMock,
      writable: true,
    });

    try {
      const matches = filterFlightResults(FLIGHT_RESULT_FIXTURES, criteria);
      sortFlightResults(matches, "recommended", criteria.cabin);

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
});
