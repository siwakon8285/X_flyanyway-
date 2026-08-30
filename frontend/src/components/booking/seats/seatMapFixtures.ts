import type { CabinClass } from "@/components/booking/search/searchTypes";
import type {
  AircraftCabinSection,
  AircraftSeat,
  CabinSeatMap,
  SeatAvailability,
  SeatPosition,
} from "@/components/booking/seats/seatMapTypes";

type ColumnDefinition = {
  column: string;
  position: SeatPosition;
};

type CabinLayoutDefinition = {
  groups: readonly (readonly ColumnDefinition[])[];
};

type AircraftCabinDefinition = {
  rowCount: number;
  rowStart: number;
};

type AircraftDefinition = Record<CabinClass, AircraftCabinDefinition>;

const AIRCRAFT_FAMILY = {
  id: "x-fly-widebody-master",
  label: "X-Fly Widebody",
} as const;

const CABIN_SECTIONS = {
  first: {
    endPercent: 19,
    labelKey: "seatMap.marker.foremostCabin",
    location: "foremost",
    markers: [
      { kind: "door", labelKey: "seatMap.marker.door1", positionPercent: 8, side: "both" },
      { kind: "galley", labelKey: "seatMap.marker.forwardGalley", positionPercent: 18, side: "left" },
      { kind: "lavatory", labelKey: "seatMap.marker.lavatory", positionPercent: 18, side: "right" },
      { kind: "bulkhead", labelKey: "seatMap.marker.bulkhead", positionPercent: 96, side: "both" },
    ],
    startPercent: 3,
  },
  business: {
    endPercent: 44,
    labelKey: "seatMap.marker.forwardCabin",
    location: "forward",
    markers: [
      { kind: "bulkhead", labelKey: "seatMap.marker.bulkhead", positionPercent: 4, side: "both" },
      { kind: "door", labelKey: "seatMap.marker.door2", positionPercent: 54, side: "both" },
      { kind: "exit", labelKey: "seatMap.marker.exit", positionPercent: 94, side: "both" },
    ],
    startPercent: 19,
  },
  "premium-economy": {
    endPercent: 61,
    labelKey: "seatMap.marker.centralCabin",
    location: "central",
    markers: [
      { kind: "bulkhead", labelKey: "seatMap.marker.bulkhead", positionPercent: 4, side: "both" },
      { kind: "wing", labelKey: "seatMap.marker.wingLeading", positionPercent: 58, side: "both" },
      { kind: "exit", labelKey: "seatMap.marker.overwingExit", positionPercent: 88, side: "both" },
    ],
    startPercent: 44,
  },
  economy: {
    endPercent: 94,
    labelKey: "seatMap.marker.midRearCabin",
    location: "mid-rear",
    markers: [
      { kind: "wing", labelKey: "seatMap.marker.wingArea", positionPercent: 12, side: "both" },
      { kind: "exit", labelKey: "seatMap.marker.rearExit", positionPercent: 62, side: "both" },
      { kind: "galley", labelKey: "seatMap.marker.rearGalley", positionPercent: 94, side: "left" },
      { kind: "lavatory", labelKey: "seatMap.marker.lavatory", positionPercent: 94, side: "right" },
    ],
    startPercent: 61,
  },
} as const satisfies Record<CabinClass, AircraftCabinSection>;

const CABIN_LAYOUTS = {
  economy: {
    groups: [
      [
        { column: "A", position: "window" },
        { column: "B", position: "middle" },
        { column: "C", position: "aisle" },
      ],
      [
        { column: "D", position: "aisle" },
        { column: "E", position: "middle" },
        { column: "F", position: "window" },
      ],
    ],
  },
  "premium-economy": {
    groups: [
      [
        { column: "A", position: "window" },
        { column: "C", position: "aisle" },
      ],
      [
        { column: "D", position: "aisle" },
        { column: "F", position: "window" },
      ],
    ],
  },
  business: {
    groups: [
      [{ column: "A", position: "window" }],
      [
        { column: "D", position: "aisle" },
        { column: "G", position: "aisle" },
      ],
      [{ column: "K", position: "window" }],
    ],
  },
  first: {
    groups: [
      [{ column: "A", position: "window" }],
      [{ column: "K", position: "window" }],
    ],
  },
} as const satisfies Record<CabinClass, CabinLayoutDefinition>;

const AIRCRAFT_CABINS: Record<string, AircraftDefinition> = {
  "Airbus A330-900": {
    economy: { rowCount: 6, rowStart: 20 },
    "premium-economy": { rowCount: 4, rowStart: 12 },
    business: { rowCount: 4, rowStart: 3 },
    first: { rowCount: 2, rowStart: 1 },
  },
  "Airbus A350-900": {
    economy: { rowCount: 7, rowStart: 20 },
    "premium-economy": { rowCount: 5, rowStart: 11 },
    business: { rowCount: 5, rowStart: 3 },
    first: { rowCount: 3, rowStart: 1 },
  },
  "Airbus A350-1000": {
    economy: { rowCount: 8, rowStart: 22 },
    "premium-economy": { rowCount: 5, rowStart: 12 },
    business: { rowCount: 6, rowStart: 3 },
    first: { rowCount: 3, rowStart: 1 },
  },
  "Boeing 787-9": {
    economy: { rowCount: 6, rowStart: 18 },
    "premium-economy": { rowCount: 4, rowStart: 10 },
    business: { rowCount: 4, rowStart: 3 },
    first: { rowCount: 2, rowStart: 1 },
  },
};

const BOOKED_SEATS = new Set(["0:1", "1:3", "3:0", "5:4"]);
const UNAVAILABLE_SEATS = new Set(["1:0", "2:2", "4:5", "6:1"]);

const getAvailability = (
  rowIndex: number,
  columnIndex: number,
): SeatAvailability => {
  const key = `${rowIndex}:${columnIndex}`;
  if (BOOKED_SEATS.has(key)) return "booked";
  if (UNAVAILABLE_SEATS.has(key)) return "unavailable";
  return "available";
};

const getSeatMapFixture = (
  aircraft: string,
  cabin: CabinClass,
): CabinSeatMap | null => {
  const aircraftDefinition = AIRCRAFT_CABINS[aircraft];
  if (!aircraftDefinition) return null;

  const cabinDefinition = aircraftDefinition[cabin];
  const layout = CABIN_LAYOUTS[cabin];
  const rows = Array.from({ length: cabinDefinition.rowCount }, (_, rowIndex) => {
    const row = cabinDefinition.rowStart + rowIndex;
    let columnIndex = 0;
    const groups = layout.groups.map((group) =>
      group.map(({ column, position }): AircraftSeat => {
        const seatNumber = `${row}${column}`;
        const seat = {
          availability: getAvailability(rowIndex, columnIndex),
          cabin,
          column,
          id: seatNumber,
          position,
          row,
          seatNumber,
        };
        columnIndex += 1;
        return seat;
      }),
    );

    return { groups, row };
  });

  return {
    aircraft,
    aircraftFamily: AIRCRAFT_FAMILY,
    cabin,
    rows,
    section: CABIN_SECTIONS[cabin],
  };
};

export { getSeatMapFixture };
