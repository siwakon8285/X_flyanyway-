import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import type {
  FlightResult,
  FlightSortOption,
} from "@/components/booking/results/flightResultTypes";

const getCabinPrice = (
  flight: FlightResult,
  cabin: FlightSearchFormValues["cabin"],
) => flight.cabinPrices.find((price) => price.cabin === cabin)?.amountThb ?? null;

const filterFlightResults = (
  flights: readonly FlightResult[],
  criteria: FlightSearchFormValues,
) =>
  flights.filter(
    (flight) =>
      flight.originCode === criteria.from?.code &&
      flight.destinationCode === criteria.to?.code &&
      getCabinPrice(flight, criteria.cabin) !== null,
  );

const sortFlightResults = (
  flights: readonly FlightResult[],
  sort: FlightSortOption,
  cabin: FlightSearchFormValues["cabin"],
) =>
  [...flights].sort((first, second) => {
    if (sort === "departure") {
      return first.departureTime.localeCompare(second.departureTime);
    }
    if (sort === "duration") {
      return first.durationMinutes - second.durationMinutes;
    }
    if (sort === "price") {
      return (getCabinPrice(first, cabin) ?? Infinity) -
        (getCabinPrice(second, cabin) ?? Infinity);
    }
    return first.recommendedRank - second.recommendedRank;
  });

const formatDuration = (durationMinutes: number) => {
  const hours = Math.floor(durationMinutes / 60);
  const minutes = durationMinutes % 60;

  return minutes === 0 ? `${hours}h` : `${hours}h ${minutes}m`;
};

export { filterFlightResults, formatDuration, getCabinPrice, sortFlightResults };
