import type { CabinClass } from "@/components/booking/search/searchTypes";

type PassengerCounts = {
  adults: number;
  children: number;
  infants: number;
};

type SelectionResult = {
  limitReached: boolean;
  selectedSeatIds: Set<string>;
};

const getRequiredSeatCount = ({ adults, children }: PassengerCounts) =>
  adults + children;

const parseSeatNumber = (seatNumber: string) => {
  const match = /^(\d+)([A-Z]+)$/.exec(seatNumber);
  return match
    ? { column: match[2] ?? "", row: Number(match[1]) }
    : { column: seatNumber, row: Number.POSITIVE_INFINITY };
};

const sortSeatNumbers = (seatNumbers: readonly string[]) =>
  [...seatNumbers].sort((left, right) => {
    const leftSeat = parseSeatNumber(left);
    const rightSeat = parseSeatNumber(right);
    return leftSeat.row - rightSeat.row || leftSeat.column.localeCompare(rightSeat.column);
  });

const toggleSeatSelection = ({
  requiredSeatCount,
  seatId,
  selectedSeatIds,
}: {
  requiredSeatCount: number;
  seatId: string;
  selectedSeatIds: ReadonlySet<string>;
}): SelectionResult => {
  const nextSelection = new Set(selectedSeatIds);

  if (nextSelection.has(seatId)) {
    nextSelection.delete(seatId);
    return { limitReached: false, selectedSeatIds: nextSelection };
  }

  if (nextSelection.size >= requiredSeatCount) {
    return { limitReached: true, selectedSeatIds: nextSelection };
  }

  nextSelection.add(seatId);
  return { limitReached: false, selectedSeatIds: nextSelection };
};

const buildPassengerDetailsHref = ({
  flightId,
  query,
  seats,
  selectedCabin,
}: {
  flightId: string;
  query: string;
  seats: readonly string[];
  selectedCabin: CabinClass;
}) => {
  const params = new URLSearchParams(query);
  params.set("flightId", flightId);
  params.set("selectedCabin", selectedCabin);
  params.set("seats", sortSeatNumbers(seats).join(","));
  return `/booking/passengers?${params.toString()}`;
};

export {
  buildPassengerDetailsHref,
  getRequiredSeatCount,
  sortSeatNumbers,
  toggleSeatSelection,
};
