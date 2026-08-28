import { render, screen } from "@testing-library/react";

import FlightDetailLoading from "@/app/flights/[flightId]/loading";

describe("flight detail loading state", () => {
  it("uses a detail-shaped accessible skeleton without a spinner", () => {
    const { container } = render(<FlightDetailLoading />);

    expect(screen.getByRole("status", { name: "Loading flight detail" })).toBeInTheDocument();
    expect(container.querySelectorAll("[data-detail-skeleton]")).toHaveLength(3);
    expect(container.querySelector(".animate-spin")).not.toBeInTheDocument();
  });
});
