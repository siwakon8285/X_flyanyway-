type TripType = "round-trip" | "one-way";

type CabinClass = "economy" | "premium-economy" | "business" | "first";

type PassengerCounts = {
  adults: number;
  children: number;
  infants: number;
};

type AirportOption = {
  airport: string;
  city: string;
  code: string;
  country: string;
};

type FlightSearchFormValues = {
  cabin: CabinClass;
  departure: string;
  from: AirportOption | null;
  passengers: PassengerCounts;
  returnDate: string;
  to: AirportOption | null;
  trip: TripType;
};

type FlightSearchErrors = {
  departure?: TranslationKey;
  from?: TranslationKey;
  passengers?: TranslationKey;
  returnDate?: TranslationKey;
  to?: TranslationKey;
};

export type {
  AirportOption,
  CabinClass,
  FlightSearchErrors,
  FlightSearchFormValues,
  PassengerCounts,
  TripType,
};
import type { TranslationKey } from "@/i18n/types";
