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

  it("preserves list semantics for a staggered reduced-motion reveal", () => {
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
      <Reveal aria-label="Available flights" as="ol" stagger={0.07}>
        <li>XF 201</li>
        <li>XF 207</li>
      </Reveal>,
    );

    const list = screen.getByRole("list", { name: "Available flights" });
    expect(list).toHaveTextContent("XF 201");
    expect(list).toHaveTextContent("XF 207");
    expect(list).not.toHaveAttribute("aria-hidden");
    expect(list).not.toHaveStyle({ opacity: "0" });
  });
});
