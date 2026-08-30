import type { CabinClass } from "@/components/booking/search/searchTypes";
import type { TranslationKey } from "@/i18n/types";

type SeatAvailability = "available" | "booked" | "unavailable";
type SeatPosition = "aisle" | "middle" | "window";
type AircraftSectionLocation = "central" | "foremost" | "forward" | "mid-rear";
type AircraftPlanMarkerKind =
  | "bulkhead"
  | "door"
  | "exit"
  | "galley"
  | "lavatory"
  | "wing";
type AircraftPlanMarkerSide = "both" | "left" | "right";

type AircraftFamily = {
  id: "x-fly-widebody-master";
  label: "X-Fly Widebody";
};

type AircraftPlanMarker = {
  kind: AircraftPlanMarkerKind;
  labelKey: TranslationKey;
  positionPercent: number;
  side: AircraftPlanMarkerSide;
};

type AircraftCabinSection = {
  endPercent: number;
  labelKey: TranslationKey;
  location: AircraftSectionLocation;
  markers: readonly AircraftPlanMarker[];
  startPercent: number;
};

type AircraftSeat = {
  availability: SeatAvailability;
  cabin: CabinClass;
  column: string;
  id: string;
  position: SeatPosition;
  row: number;
  seatNumber: string;
};

type SeatRow = {
  groups: readonly (readonly AircraftSeat[])[];
  row: number;
};

type CabinSeatMap = {
  aircraft: string;
  aircraftFamily: AircraftFamily;
  cabin: CabinClass;
  rows: readonly SeatRow[];
  section: AircraftCabinSection;
};

export type {
  AircraftSeat,
  AircraftCabinSection,
  AircraftFamily,
  AircraftPlanMarker,
  AircraftPlanMarkerKind,
  AircraftPlanMarkerSide,
  AircraftSectionLocation,
  CabinSeatMap,
  SeatAvailability,
  SeatPosition,
  SeatRow,
};
