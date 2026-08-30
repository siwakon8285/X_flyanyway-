import { screen } from "@testing-library/react";
import { render } from "@/tests/renderWithLanguage";

import SeatMapLoading from "@/app/flights/[flightId]/seats/loading";

describe("seat map loading state", () => {
  it("uses an aircraft-map-shaped accessible skeleton without a spinner", () => {
    const { container } = render(<SeatMapLoading />);

    expect(screen.getByRole("status", { name: "Loading seat map" })).toBeInTheDocument();
    expect(container.querySelectorAll("[data-seat-map-skeleton]")).toHaveLength(2);
    expect(container.querySelector(".animate-spin")).not.toBeInTheDocument();
  });
});
