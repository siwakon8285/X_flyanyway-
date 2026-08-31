import type { CabinSeatMap, SeatAvailability } from "@/components/booking/seats/seatMapTypes";
import type { SeatInventory } from "@/components/booking/seats/seatHoldClient";

const availabilityByStatus = {
  AVAILABLE: "available",
  BOOKED: "booked",
  HELD_BY_ME: "available",
  UNAVAILABLE: "unavailable",
} as const satisfies Record<string, SeatAvailability>;

const applySeatInventory = (
  map: CabinSeatMap,
  inventory: SeatInventory,
): CabinSeatMap => {
  const statuses = new Map(
    inventory.seats.map((seat) => [seat.seatNumber, seat.status] as const),
  );

  return {
    ...map,
    rows: map.rows.map((row) => ({
      ...row,
      groups: row.groups.map((group) =>
        group.map((seat) => ({
          ...seat,
          availability: statuses.has(seat.seatNumber)
            ? availabilityByStatus[statuses.get(seat.seatNumber)!]
            : "unavailable",
        })),
      ),
    })),
  };
};

const getHeldByMeSeats = (inventory: SeatInventory) =>
  inventory.seats
    .filter((seat) => seat.status === "HELD_BY_ME")
    .map((seat) => seat.seatNumber);

export { applySeatInventory, getHeldByMeSeats };
