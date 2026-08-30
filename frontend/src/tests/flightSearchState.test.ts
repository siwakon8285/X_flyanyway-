import { AIRPORT_FIXTURES } from "@/components/booking/search/airportFixtures";
import {
  createDefaultFlightSearchValues,
  isCompleteFlightSearchQuery,
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

  it("accepts only complete supported result-page criteria", () => {
    expect(
      isCompleteFlightSearchQuery(
        new URLSearchParams(
          "from=BKK&to=LHR&departure=2099-05-10&return=2099-05-18&adults=1&children=0&infants=0&cabin=business&trip=round-trip",
        ),
      ),
    ).toBe(true);

    expect(
      isCompleteFlightSearchQuery(
        new URLSearchParams(
          "from=BKK&to=MOON&departure=2099-05-10&adults=1&children=0&infants=0&cabin=spaceship&trip=one-way",
        ),
      ),
    ).toBe(false);
    expect(
      isCompleteFlightSearchQuery(
        new URLSearchParams(
          "from=BKK&to=LHR&departure=2099-05-10&adults=1&children=0&infants=0&cabin=economy&trip=round-trip",
        ),
      ),
    ).toBe(false);
  });

  it("rejects an identical origin and destination", () => {
    const errors = validateFlightSearch(
      { ...createValidValues(), to: bkk },
      "2026-08-27",
    );

    expect(errors.to).toBe("flightSearch.validation.destinationDifferent");
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
      departure: "flightSearch.validation.departurePast",
      returnDate: "flightSearch.validation.returnOrder",
    });

    expect(
      validateFlightSearch(
        { ...createValidValues(), departure: "", returnDate: "" },
        "2026-08-27",
      ),
    ).toMatchObject({
      departure: "flightSearch.validation.departureRequired",
      returnDate: "flightSearch.validation.returnRequired",
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
    ).toBe("flightSearch.validation.passengerRange");
  });

  it("creates independent default passenger state", () => {
    const first = createDefaultFlightSearchValues();
    const second = createDefaultFlightSearchValues();

    first.passengers.adults = 2;

    expect(second.passengers.adults).toBe(1);
  });
});
