import { fireEvent, render, screen } from "@testing-library/react";

import { SeatButton } from "@/components/booking/seats/SeatButton";
import type { AircraftSeat } from "@/components/booking/seats/seatMapTypes";

const seat: AircraftSeat = {
  availability: "available",
  cabin: "economy",
  column: "A",
  id: "20A",
  position: "window",
  row: 20,
  seatNumber: "20A",
};

describe("SeatButton", () => {
  it("keeps selection usable and spatial motion disabled under reduced motion", () => {
    const onToggle = jest.fn();
    render(<SeatButton onToggle={onToggle} seat={seat} selected={false} />);
    const button = screen.getByRole("button", { name: /seat 20a.*available/i });

    fireEvent.mouseEnter(button);
    expect(button).not.toHaveStyle({ transform: expect.stringContaining("scale") });
    fireEvent.click(button);
    expect(onToggle).toHaveBeenCalledWith("20A");
  });

  it("uses a native button so browser Enter and Space activation remain available", () => {
    render(<SeatButton onToggle={jest.fn()} seat={seat} selected={false} />);
    const button = screen.getByRole("button", { name: /seat 20a.*available/i });

    button.focus();
    expect(button).toHaveFocus();
    expect(button.tagName).toBe("BUTTON");
    expect(button).toHaveAttribute("type", "button");
  });
});
