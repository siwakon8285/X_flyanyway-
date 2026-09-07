import { getSeatMapFixture } from "@/components/booking/seats/seatMapFixtures";
import { applySeatInventory, getHeldByMeSeats } from "@/components/booking/seats/seatHoldState";

describe("seat hold inventory reconciliation", () => {
  it("preserves seats held by the current browser while disabling another hold", () => {
    const map = getSeatMapFixture("Airbus A350-900", "business");
    if (!map) throw new Error("Expected seat map fixture");
    const inventory = {
      cabin: "business" as const,
      departureDate: "2099-05-10",
      flightId: "xf-201",
      seats: [
        {
          columnCode: "A",
          position: "window" as const,
          rowNumber: 3,
          seatNumber: "3A",
          status: "HELD_BY_ME" as const,
        },
        {
          columnCode: "D",
          position: "aisle" as const,
          rowNumber: 3,
          seatNumber: "3D",
          status: "UNAVAILABLE" as const,
        },
      ],
      serverTime: "2099-05-10T10:00:00Z",
    };

    const reconciled = applySeatInventory(map, inventory);
    const seats = reconciled.rows.flatMap((row) => row.groups.flat());

    expect(seats.find((seat) => seat.id === "3A")?.availability).toBe("available");
    expect(seats.find((seat) => seat.id === "3D")?.availability).toBe("unavailable");
    expect(getHeldByMeSeats(inventory)).toEqual(["3A"]);
  });
});
