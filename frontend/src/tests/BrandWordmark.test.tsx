import { render, screen, within } from "@testing-library/react";

import { BrandWordmark } from "@/components/brand/BrandWordmark";

describe("BrandWordmark", () => {
  it("uses the winged-X app icon once in the accessible X-Fly Anyway lockup", () => {
    render(<BrandWordmark />);

    const lockup = screen.getByRole("img", { name: "X-Fly Anyway" });
    const logo = lockup.querySelector("img");

    expect(logo).toHaveAttribute(
      "src",
      expect.stringContaining("%2Fimg.jpg"),
    );
    expect(logo).toHaveAttribute("alt", "");
    expect(logo).toHaveClass("object-contain");
    expect(within(lockup).getByText("-FLY ANYWAY")).toBeInTheDocument();
    expect(within(lockup).queryByText("X-FLY ANYWAY")).not.toBeInTheDocument();
  });
});
