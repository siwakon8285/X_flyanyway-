import {
  getRequiredSeatCount,
  sortSeatNumbers,
  toggleSeatSelection,
} from "@/components/booking/seats/seatMapUtils";

describe("seat map utilities", () => {
  it("requires seats for adults and children while treating infants as lap infants", () => {
    expect(
      getRequiredSeatCount({ adults: 2, children: 1, infants: 4 }),
    ).toBe(3);
  });

  it("sorts seat codes by numeric row and then column", () => {
    expect(sortSeatNumbers(["12F", "3K", "12A", "3A"])).toEqual([
      "3A",
      "3K",
      "12A",
      "12F",
    ]);
  });

  it("selects, deselects, and refuses to silently replace a seat at the limit", () => {
    const firstSelection = toggleSeatSelection({
      requiredSeatCount: 1,
      seatId: "12A",
      selectedSeatIds: new Set<string>(),
    });
    expect([...firstSelection.selectedSeatIds]).toEqual(["12A"]);
    expect(firstSelection.limitReached).toBe(false);

    const atLimit = toggleSeatSelection({
      requiredSeatCount: 1,
      seatId: "12B",
      selectedSeatIds: firstSelection.selectedSeatIds,
    });
    expect([...atLimit.selectedSeatIds]).toEqual(["12A"]);
    expect(atLimit.limitReached).toBe(true);

    const deselected = toggleSeatSelection({
      requiredSeatCount: 1,
      seatId: "12A",
      selectedSeatIds: atLimit.selectedSeatIds,
    });
    expect([...deselected.selectedSeatIds]).toEqual([]);
    expect(deselected.limitReached).toBe(false);
  });

});
