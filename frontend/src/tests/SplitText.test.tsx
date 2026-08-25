import { render, screen } from "@testing-library/react";

import { SplitText } from "@/components/motion/SplitText";

describe("SplitText", () => {
  it("hides generated units from assistive technology and reverts on unmount", () => {
    window.matchMedia = jest.fn().mockReturnValue({
      matches: false,
      media: "(prefers-reduced-motion: reduce)",
      onchange: null,
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
      addListener: jest.fn(),
      removeListener: jest.fn(),
      dispatchEvent: jest.fn(),
    });

    const originalText = "Movement with a clear purpose.";
    const { unmount } = render(
      <SplitText as="h2" split="words,chars" text={originalText} />,
    );
    const heading = screen.getByRole("heading", { name: originalText });

    const generatedUnits = Array.from(heading.querySelectorAll(".word, .char"));
    expect(generatedUnits.length).toBeGreaterThan(0);
    expect(
      generatedUnits.every((unit) => unit.getAttribute("aria-hidden") === "true"),
    ).toBe(true);
    expect(heading).toHaveAttribute("aria-label", originalText);

    unmount();

    expect(heading).toHaveTextContent(originalText);
    expect(heading.querySelector(".word, .char")).not.toBeInTheDocument();
  });
});
