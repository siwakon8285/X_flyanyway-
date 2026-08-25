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

  it("can expose accessible split units without owning their animation", () => {
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

    const originalText = "Go anywhere. Fly different.";
    const { unmount } = render(
      <SplitText animate={false} as="h1" split="words" text={originalText} />,
    );
    const heading = screen.getByRole("heading", { name: originalText });
    const generatedWords = Array.from(heading.querySelectorAll(".word"));

    expect(generatedWords.length).toBeGreaterThan(0);
    expect(
      generatedWords.every((word) => {
        const styles = word.getAttribute("style") ?? "";
        return !/(opacity|visibility|transform)/.test(styles);
      }),
    ).toBe(true);
    expect(
      generatedWords.every((word) => word.getAttribute("aria-hidden") === "true"),
    ).toBe(true);

    unmount();

    expect(heading).toHaveTextContent(originalText);
    expect(heading.querySelector(".word")).not.toBeInTheDocument();
  });
});
