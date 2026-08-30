import { screen } from "@testing-library/react";
import { render } from "@/tests/renderWithLanguage";

import SeatSelectionBoundary from "@/app/flights/[flightId]/seats/page";

const validQuery = {
  adults: "1",
  cabin: "business",
  children: "0",
  departure: "2099-05-10",
  from: "BKK",
  infants: "0",
  return: "2099-05-18",
  selectedCabin: "first",
  to: "LHR",
  trip: "round-trip",
} as const;

describe("seat selection route boundary", () => {
  it("renders the local map for the validated flight and selected cabin", async () => {
    const route = await SeatSelectionBoundary({
      params: Promise.resolve({ flightId: "xf-201" }),
      searchParams: Promise.resolve(validQuery),
    });

    const { container } = render(route);

    expect(screen.getByRole("heading", { level: 1, name: "Select your seat" })).toBeInTheDocument();
    expect(screen.getByText("XF 201 · BKK → LHR")).toBeInTheDocument();
    expect(container.querySelector("[data-seat-map]")).toBeInTheDocument();
    expect(container.querySelector("[data-seat-model=\"first\"]")).toBeInTheDocument();
    expect(screen.queryByRole("grid", { name: /seat/i })).not.toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Back to Flight" })).toHaveAttribute(
      "href",
      "/flights/xf-201?from=BKK&to=LHR&departure=2099-05-10&return=2099-05-18&adults=1&children=0&infants=0&cabin=business&trip=round-trip&selectedCabin=first#cabin-experience",
    );
  });

  it("rejects malformed or unavailable selected cabins", async () => {
    await expect(
      SeatSelectionBoundary({
        params: Promise.resolve({ flightId: "xf-315" }),
        searchParams: Promise.resolve(validQuery),
      }),
    ).rejects.toThrow(/NEXT_HTTP_ERROR_FALLBACK;404/);
    await expect(
      SeatSelectionBoundary({
        params: Promise.resolve({ flightId: "xf-201" }),
        searchParams: Promise.resolve({
          ...validQuery,
          selectedCabin: "spaceship",
        }),
      }),
    ).rejects.toThrow(/NEXT_HTTP_ERROR_FALLBACK;404/);
  });

  it("rejects invalid flight IDs and route mismatches", async () => {
    await expect(
      SeatSelectionBoundary({
        params: Promise.resolve({ flightId: "unknown" }),
        searchParams: Promise.resolve(validQuery),
      }),
    ).rejects.toThrow(/NEXT_HTTP_ERROR_FALLBACK;404/);
    await expect(
      SeatSelectionBoundary({
        params: Promise.resolve({ flightId: "xf-621" }),
        searchParams: Promise.resolve(validQuery),
      }),
    ).rejects.toThrow(/NEXT_HTTP_ERROR_FALLBACK;404/);
  });
});
