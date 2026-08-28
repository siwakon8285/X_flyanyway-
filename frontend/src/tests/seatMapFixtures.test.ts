import { getSeatMapFixture } from "@/components/booking/seats/seatMapFixtures";

describe("seat map fixtures", () => {
  it.each([
    ["economy", [3, 3], "Airbus A350-900"],
    ["premium-economy", [2, 2], "Airbus A350-900"],
    ["business", [1, 2, 1], "Airbus A350-900"],
    ["first", [1, 1], "Airbus A350-900"],
  ] as const)(
    "builds a structurally distinct %s layout",
    (cabin, expectedGroupSizes, aircraft) => {
      const map = getSeatMapFixture(aircraft, cabin);

      expect(map).not.toBeNull();
      expect(map?.rows[0]?.groups.map((group) => group.length)).toEqual(
        expectedGroupSizes,
      );
      expect(map?.rows.length).toBeGreaterThan(1);
      expect(map?.rows[0]?.groups.flat().every((seat) => seat.cabin === cabin)).toBe(
        true,
      );
    },
  );

  it("provides stable seat codes, row numbers, positions, and varied static availability", () => {
    const map = getSeatMapFixture("Airbus A350-900", "economy");
    const seats = map?.rows.flatMap((row) => row.groups.flat()) ?? [];

    expect(map?.rows[0]?.row).toBeGreaterThan(0);
    expect(seats[0]).toMatchObject({
      cabin: "economy",
      column: "A",
      position: "window",
    });
    expect(seats[0]?.seatNumber).toBe(`${seats[0]?.row}A`);
    expect(new Set(seats.map((seat) => seat.seatNumber)).size).toBe(seats.length);
    expect(new Set(seats.map((seat) => seat.availability))).toEqual(
      new Set(["available", "booked", "unavailable"]),
    );
  });

  it("returns no fallback map for an unknown aircraft", () => {
    expect(getSeatMapFixture("Unknown aircraft", "business")).toBeNull();
  });

  it("maps every cabin into one shared X-Fly aircraft family in longitudinal order", () => {
    const cabins = [
      getSeatMapFixture("Airbus A350-900", "first"),
      getSeatMapFixture("Airbus A350-900", "business"),
      getSeatMapFixture("Airbus A350-900", "premium-economy"),
      getSeatMapFixture("Airbus A350-900", "economy"),
    ];

    expect(cabins.map((map) => map?.aircraftFamily.id)).toEqual([
      "x-fly-widebody-master",
      "x-fly-widebody-master",
      "x-fly-widebody-master",
      "x-fly-widebody-master",
    ]);
    expect(cabins.map((map) => map?.section.location)).toEqual([
      "foremost",
      "forward",
      "central",
      "mid-rear",
    ]);
    expect(cabins.map((map) => map?.section.startPercent)).toEqual([
      3, 19, 44, 61,
    ]);
  });

  it("adds restrained aircraft-plan markers appropriate to each section", () => {
    const first = getSeatMapFixture("Airbus A350-900", "first");
    const premium = getSeatMapFixture("Airbus A350-900", "premium-economy");
    const economy = getSeatMapFixture("Airbus A350-900", "economy");

    expect(first?.section.markers.map((marker) => marker.kind)).toEqual([
      "door",
      "galley",
      "lavatory",
      "bulkhead",
    ]);
    expect(premium?.section.markers.map((marker) => marker.kind)).toEqual([
      "bulkhead",
      "wing",
      "exit",
    ]);
    expect(economy?.section.markers.map((marker) => marker.kind)).toEqual([
      "wing",
      "exit",
      "galley",
      "lavatory",
    ]);
  });
});
