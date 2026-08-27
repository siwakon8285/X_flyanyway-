import { AIRPORT_FIXTURES } from "@/components/booking/search/airportFixtures";
import type {
  AirportOption,
  CabinClass,
  FlightSearchErrors,
  FlightSearchFormValues,
  TripType,
} from "@/components/booking/search/searchTypes";

const PASSENGER_UI_SAFETY_LIMIT = 99;
const cabinClasses = [
  "economy",
  "premium-economy",
  "business",
  "first",
] as const satisfies readonly CabinClass[];
const tripTypes = ["round-trip", "one-way"] as const satisfies readonly TripType[];

const createDefaultFlightSearchValues = (): FlightSearchFormValues => ({
  cabin: "economy",
  departure: "",
  from: null,
  passengers: { adults: 1, children: 0, infants: 0 },
  returnDate: "",
  to: null,
  trip: "round-trip",
});

const isDateInputValue = (value: string) => {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) return false;

  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const date = new Date(year, month - 1, day);

  return (
    date.getFullYear() === year &&
    date.getMonth() === month - 1 &&
    date.getDate() === day
  );
};

const getTodayDateInputValue = (date = new Date()) => {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");

  return `${year}-${month}-${day}`;
};

const findAirport = (code: string | null): AirportOption | null =>
  AIRPORT_FIXTURES.find((airport) => airport.code === code?.toUpperCase()) ?? null;

const parseCount = (value: string | null, fallback: number) => {
  if (value === null || !/^\d+$/.test(value)) return fallback;

  const parsed = Number(value);
  return Number.isSafeInteger(parsed) && parsed <= PASSENGER_UI_SAFETY_LIMIT
    ? parsed
    : fallback;
};

const parseFlightSearch = (params: URLSearchParams): FlightSearchFormValues => {
  const defaults = createDefaultFlightSearchValues();
  const tripValue = params.get("trip");
  const cabinValue = params.get("cabin");
  const departureValue = params.get("departure") ?? "";
  const returnValue = params.get("return") ?? "";
  const trip = tripTypes.find((tripType) => tripType === tripValue) ?? defaults.trip;
  const cabin =
    cabinClasses.find((cabinClass) => cabinClass === cabinValue) ?? defaults.cabin;

  return {
    cabin,
    departure: isDateInputValue(departureValue) ? departureValue : "",
    from: findAirport(params.get("from")),
    passengers: {
      adults: Math.max(1, parseCount(params.get("adults"), defaults.passengers.adults)),
      children: parseCount(params.get("children"), defaults.passengers.children),
      infants: parseCount(params.get("infants"), defaults.passengers.infants),
    },
    returnDate:
      trip === "round-trip" && isDateInputValue(returnValue) ? returnValue : "",
    to: findAirport(params.get("to")),
    trip,
  };
};

const isCompleteFlightSearchQuery = (params: URLSearchParams) => {
  const values = parseFlightSearch(params);
  const hasCanonicalPassengers = (
    ["adults", "children", "infants"] as const
  ).every(
    (key) => params.get(key) === String(values.passengers[key]),
  );
  const hasCanonicalCriteria =
    params.get("from") === values.from?.code &&
    params.get("to") === values.to?.code &&
    params.get("departure") === values.departure &&
    params.get("cabin") === values.cabin &&
    params.get("trip") === values.trip &&
    hasCanonicalPassengers;
  const hasRequiredReturn =
    values.trip === "one-way" || params.get("return") === values.returnDate;

  return (
    hasCanonicalCriteria &&
    hasRequiredReturn &&
    Object.keys(validateFlightSearch(values)).length === 0
  );
};

const serializeFlightSearch = (values: FlightSearchFormValues) => {
  const params = new URLSearchParams();

  if (values.from) params.set("from", values.from.code);
  if (values.to) params.set("to", values.to.code);
  if (values.departure) params.set("departure", values.departure);
  if (values.trip === "round-trip" && values.returnDate) {
    params.set("return", values.returnDate);
  }
  params.set("adults", String(values.passengers.adults));
  params.set("children", String(values.passengers.children));
  params.set("infants", String(values.passengers.infants));
  params.set("cabin", values.cabin);
  params.set("trip", values.trip);

  return params.toString();
};

const validateFlightSearch = (
  values: FlightSearchFormValues,
  today = getTodayDateInputValue(),
): FlightSearchErrors => {
  const errors: FlightSearchErrors = {};

  if (!values.from) errors.from = "Choose an origin.";
  if (!values.to) {
    errors.to = "Choose a destination.";
  } else if (values.from?.code === values.to.code) {
    errors.to = "Origin and destination must be different.";
  }

  if (!values.departure) {
    errors.departure = "Choose a departure date.";
  } else if (!isDateInputValue(values.departure)) {
    errors.departure = "Choose a valid departure date.";
  } else if (values.departure < today) {
    errors.departure = "Departure date cannot be in the past.";
  }

  if (values.trip === "round-trip") {
    if (!values.returnDate) {
      errors.returnDate = "Choose a return date.";
    } else if (!isDateInputValue(values.returnDate)) {
      errors.returnDate = "Choose a valid return date.";
    } else if (values.departure && values.returnDate < values.departure) {
      errors.returnDate = "Return date must be on or after departure.";
    }
  }

  const counts = Object.values(values.passengers);
  const outsideSafetyRange = counts.some(
    (count) =>
      !Number.isSafeInteger(count) || count < 0 || count > PASSENGER_UI_SAFETY_LIMIT,
  );
  if (outsideSafetyRange) {
    errors.passengers = "Passenger counts are outside the supported UI range.";
  } else if (values.passengers.adults < 1) {
    errors.passengers = "At least one adult is required.";
  }

  return errors;
};

export {
  PASSENGER_UI_SAFETY_LIMIT,
  createDefaultFlightSearchValues,
  getTodayDateInputValue,
  isCompleteFlightSearchQuery,
  parseFlightSearch,
  serializeFlightSearch,
  validateFlightSearch,
};
