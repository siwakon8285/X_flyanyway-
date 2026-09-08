import { screen } from "@testing-library/react";
import { render } from "@/tests/renderWithLanguage";

import FlightDetailRoute from "@/app/(customer)/flights/[flightId]/page";
import { FLIGHT_RESULT_FIXTURES } from "@/components/booking/results/flightResultFixtures";
import { AIRPORT_FIXTURES } from "@/components/booking/search/airportFixtures";
import { fetchPublicAirports, fetchPublicFlightDetail } from "@/lib/flights/publicFlightBackend";

jest.mock("@/lib/flights/publicFlightBackend");

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

describe("/flights/[flightId] route", () => {
  beforeEach(() => {
    jest.mocked(fetchPublicAirports).mockResolvedValue([...AIRPORT_FIXTURES]);
    jest.mocked(fetchPublicFlightDetail).mockImplementation(async (id) => FLIGHT_RESULT_FIXTURES.find((flight) => flight.id === id) ?? null);
  });
  it("renders a validated local fixture as the flight detail experience", async () => {
    const route = await FlightDetailRoute({
      params: Promise.resolve({ flightId: "xf-201" }),
      searchParams: Promise.resolve(validQuery),
    });

    render(route);

    expect(
      screen.getByRole("heading", { level: 1, name: /BKK.*LHR/ }),
    ).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Cabin experience" })).toBeInTheDocument();
  });

  it.each([
    ["unknown flight", "missing", validQuery],
    ["route mismatch", "xf-621", validQuery],
    ["malformed query", "xf-201", { ...validQuery, adults: "zero" }],
  ])("uses not-found behavior for %s", async (_case, flightId, searchParams) => {
    await expect(
      FlightDetailRoute({
        params: Promise.resolve({ flightId }),
        searchParams: Promise.resolve(searchParams),
      }),
    ).rejects.toThrow(/NEXT_HTTP_ERROR_FALLBACK;404/);
  });
});
