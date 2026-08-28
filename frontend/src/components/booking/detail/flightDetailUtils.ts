import { FLIGHT_RESULT_FIXTURES } from "@/components/booking/results/flightResultFixtures";
import { getCabinPrice } from "@/components/booking/results/flightResultUtils";
import {
  isCompleteFlightSearchQuery,
  parseFlightSearch,
  serializeFlightSearch,
} from "@/components/booking/search/searchState";
import type {
  CabinClass,
  FlightSearchFormValues,
} from "@/components/booking/search/searchTypes";
import type { FlightResult } from "@/components/booking/results/flightResultTypes";

type RouteQuery = Record<string, string | string[] | undefined>;

type FlightDetailRequest = {
  criteria: FlightSearchFormValues;
  flight: FlightResult;
  query: string;
};

type SeatSelectionRequest = FlightDetailRequest & {
  selectedCabin: CabinClass;
};

const cabinClasses = [
  "economy",
  "premium-economy",
  "business",
  "first",
] as const satisfies readonly CabinClass[];

const toUrlSearchParams = (query: RouteQuery) => {
  const params = new URLSearchParams();

  Object.entries(query).forEach(([key, value]) => {
    if (typeof value === "string") params.set(key, value);
  });

  return params;
};

const isCabinClass = (value: string | null): value is CabinClass =>
  cabinClasses.some((cabin) => cabin === value);

const resolveFlightDetailRequest = (
  flightId: string,
  routeQuery: RouteQuery,
): FlightDetailRequest | null => {
  const params = toUrlSearchParams(routeQuery);
  if (!isCompleteFlightSearchQuery(params)) return null;

  const criteria = parseFlightSearch(params);
  const flight = FLIGHT_RESULT_FIXTURES.find((item) => item.id === flightId);

  if (
    !flight ||
    flight.originCode !== criteria.from?.code ||
    flight.destinationCode !== criteria.to?.code ||
    getCabinPrice(flight, criteria.cabin) === null
  ) {
    return null;
  }

  return {
    criteria,
    flight,
    query: serializeFlightSearch(criteria),
  };
};

const buildSeatSelectionHref = ({
  flightId,
  query,
  selectedCabin,
}: {
  flightId: string;
  query: string;
  selectedCabin: CabinClass;
}) => {
  const params = new URLSearchParams(query);
  params.set("selectedCabin", selectedCabin);

  return `/flights/${flightId}/seats?${params.toString()}`;
};

const resolveSeatSelectionRequest = (
  flightId: string,
  routeQuery: RouteQuery,
): SeatSelectionRequest | null => {
  const request = resolveFlightDetailRequest(flightId, routeQuery);
  const selectedCabinValue = toUrlSearchParams(routeQuery).get("selectedCabin");

  if (
    !request ||
    !isCabinClass(selectedCabinValue) ||
    getCabinPrice(request.flight, selectedCabinValue) === null
  ) {
    return null;
  }

  return { ...request, selectedCabin: selectedCabinValue };
};

export {
  buildSeatSelectionHref,
  resolveFlightDetailRequest,
  resolveSeatSelectionRequest,
};
export type { FlightDetailRequest, RouteQuery, SeatSelectionRequest };
