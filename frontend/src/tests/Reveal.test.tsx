import { render, screen } from "@testing-library/react";

import { Reveal } from "@/components/motion/Reveal";

describe("Reveal", () => {
  it("keeps semantic content visible when reduced motion is requested", () => {
    window.matchMedia = jest.fn().mockReturnValue({
      matches: true,
      media: "(prefers-reduced-motion: reduce)",
      onchange: null,
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
      addListener: jest.fn(),
      removeListener: jest.fn(),
      dispatchEvent: jest.fn(),
    });

    render(
      <Reveal aria-label="Motion-safe content" as="section">
        <p>Every journey stays readable.</p>
      </Reveal>,
    );

    const section = screen.getByRole("region", { name: "Motion-safe content" });
    expect(section).toHaveTextContent("Every journey stays readable.");
    expect(section).not.toHaveAttribute("aria-hidden");
    expect(section).not.toHaveStyle({ opacity: "0" });
  });
});
