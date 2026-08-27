import { AIRPORT_FIXTURES } from "@/components/booking/search/airportFixtures";
import {
  createDefaultFlightSearchValues,
  parseFlightSearch,
  serializeFlightSearch,
  validateFlightSearch,
} from "@/components/booking/search/searchState";
import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";

const bkk = AIRPORT_FIXTURES[0];
const hnd = AIRPORT_FIXTURES[1];

const createValidValues = (): FlightSearchFormValues => ({
  cabin: "business",
  departure: "2026-09-10",
  from: bkk,
  passengers: { adults: 2, children: 1, infants: 0 },
  returnDate: "2026-09-18",
  to: hnd,
  trip: "round-trip",
});

describe("flight search state", () => {
  it("serializes only the supported round-trip criteria", () => {
    expect(serializeFlightSearch(createValidValues())).toBe(
      "from=BKK&to=HND&departure=2026-09-10&return=2026-09-18&adults=2&children=1&infants=0&cabin=business&trip=round-trip",
    );
  });

  it("omits return criteria for a one-way trip", () => {
    expect(
      serializeFlightSearch({
        ...createValidValues(),
        trip: "one-way",
      }),
    ).not.toContain("return=");
  });

  it("parses supported criteria and safely falls back for invalid values", () => {
    const parsed = parseFlightSearch(
      new URLSearchParams(
        "from=BKK&to=MOON&departure=not-a-date&return=2026-02-30&adults=0&children=-1&infants=100&cabin=spaceship&trip=one-way&ignored=value",
      ),
    );

    expect(parsed).toEqual({
      cabin: "economy",
      departure: "",
      from: bkk,
      passengers: { adults: 1, children: 0, infants: 0 },
      returnDate: "",
      to: null,
      trip: "one-way",
    });
  });

  it("rejects an identical origin and destination", () => {
    const errors = validateFlightSearch(
      { ...createValidValues(), to: bkk },
      "2026-08-27",
    );

    expect(errors.to).toBe("Origin and destination must be different.");
  });

  it("rejects missing or chronologically invalid dates", () => {
    expect(
      validateFlightSearch(
        {
          ...createValidValues(),
          departure: "2026-08-26",
          returnDate: "2026-08-25",
        },
        "2026-08-27",
      ),
    ).toMatchObject({
      departure: "Departure date cannot be in the past.",
      returnDate: "Return date must be on or after departure.",
    });

    expect(
      validateFlightSearch(
        { ...createValidValues(), departure: "", returnDate: "" },
        "2026-08-27",
      ),
    ).toMatchObject({
      departure: "Choose a departure date.",
      returnDate: "Choose a return date.",
    });
  });

  it("requires at least one adult and keeps passenger values within the UI safety guard", () => {
    expect(
      validateFlightSearch(
        {
          ...createValidValues(),
          passengers: { adults: 0, children: 100, infants: -1 },
        },
        "2026-08-27",
      ).passengers,
    ).toBe("Passenger counts are outside the supported UI range.");
  });

  it("creates independent default passenger state", () => {
    const first = createDefaultFlightSearchValues();
    const second = createDefaultFlightSearchValues();

    first.passengers.adults = 2;

    expect(second.passengers.adults).toBe(1);
  });
});
