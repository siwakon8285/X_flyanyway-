import { getSeatMapFixture } from "@/components/booking/seats/seatMapFixtures";
import { applySeatInventory, getHeldByMeSeats } from "@/components/booking/seats/seatHoldState";

describe("seat hold inventory reconciliation", () => {
  it("preserves seats held by the current browser while disabling another hold", () => {
    const map = getSeatMapFixture("Airbus A350-900", "economy");
    if (!map) throw new Error("Expected seat map fixture");
    const inventory = {
      cabin: "economy" as const,
      departureDate: "2099-05-10",
      flightId: "xf-201",
      seats: [
        {
          columnCode: "A",
          position: "window" as const,
          rowNumber: 20,
          seatNumber: "20A",
          status: "HELD_BY_ME" as const,
        },
        {
          columnCode: "B",
          position: "middle" as const,
          rowNumber: 20,
          seatNumber: "20B",
          status: "UNAVAILABLE" as const,
        },
      ],
      serverTime: "2099-05-10T10:00:00Z",
    };

    const reconciled = applySeatInventory(map, inventory);
    const seats = reconciled.rows.flatMap((row) => row.groups.flat());

    expect(seats.find((seat) => seat.id === "20A")?.availability).toBe("available");
    expect(seats.find((seat) => seat.id === "20B")?.availability).toBe("unavailable");
    expect(getHeldByMeSeats(inventory)).toEqual(["20A"]);
  });
});
