import type { CabinClass } from "@/components/booking/search/searchTypes";

type FlightStatus = "scheduled";
type StopType = "direct" | "one-stop";
type FlightSortOption = "recommended" | "departure" | "duration" | "price";

type CabinPrice = {
  amountThb: number;
  cabin: CabinClass;
};

type FlightResult = {
  aircraft: string;
  arrivalDayOffset: 0 | 1;
  arrivalTime: string;
  cabinPrices: readonly CabinPrice[];
  departureTime: string;
  destinationCode: string;
  durationMinutes: number;
  flightNumber: string;
  id: string;
  originCode: string;
  recommendedRank: number;
  status: FlightStatus;
  stops: StopType;
};

export type {
  CabinPrice,
  FlightResult,
  FlightSortOption,
  FlightStatus,
  StopType,
};
