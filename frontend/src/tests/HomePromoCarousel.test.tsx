import { fireEvent, screen } from "@testing-library/react";

import Home from "@/app/page";
import { render } from "@/tests/renderWithLanguage";

describe("homepage promotion carousel", () => {
  it("replaces the former featured journey card with an accessible carousel", () => {
    render(<Home />);

    expect(
      screen.getByRole("region", { name: "Featured promotions" }),
    ).toBeInTheDocument();
    expect(screen.queryByText("Featured journey")).not.toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Travel, elevated." })).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Featured promotions" }),
    ).toHaveAttribute("id", "offers");

    const signature = screen.getByText("X-Fly Signature");
    expect(signature).toHaveAttribute("data-x-fly-signature");
    expect(signature).toHaveClass(
      "motion-safe:hover:scale-[1.025]",
      "motion-reduce:transform-none",
    );
  });

  it("moves between promotions and updates the active pagination dot", () => {
    render(<Home />);

    const dots = screen.getAllByRole("button", { name: /Go to promotion/ });
    expect(dots).toHaveLength(3);
    expect(dots[0]).toHaveAttribute("aria-current", "true");

    fireEvent.click(screen.getByRole("button", { name: "Next promotion" }));

    expect(screen.getByRole("heading", { name: "Book early. Go further." })).toBeInTheDocument();
    expect(dots[1]).toHaveAttribute("aria-current", "true");
  });
});
