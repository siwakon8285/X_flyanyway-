import { render, screen, within } from "@testing-library/react";

import Home from "@/app/page";

describe("cinematic hero", () => {
  it("renders one public headline, the CTA hierarchy, and an aircraft media layer", () => {
    const { container } = render(<Home />);

    const headings = screen.getAllByRole("heading", { level: 1 });
    expect(headings).toHaveLength(1);
    expect(headings[0]).toHaveAccessibleName(
      "Go anywhere. Fly different.",
    );

    const hero = screen.getByRole("region", {
      name: "Go anywhere. Fly different.",
    });

    expect(
      within(hero).getByRole("link", { name: "Book a Flight" }),
    ).toHaveAttribute("href", "/#flight-search");
    expect(within(hero).getByRole("link", { name: "Explore X-Fly" })).toHaveAttribute(
      "href",
      "#journey",
    );
    expect(container.querySelector("#journey")).toBeInTheDocument();
    expect(container.querySelector("#flight-search")).toBeInTheDocument();
    expect(
      container.querySelector("#global-reach")?.compareDocumentPosition(
        container.querySelector("#flight-search") as Node,
      ),
    ).toBe(Node.DOCUMENT_POSITION_FOLLOWING);

    const mediaLayer = container.querySelector("[data-hero-media]");
    expect(mediaLayer).toBeInTheDocument();
    expect(mediaLayer?.querySelector('img[alt=""]')).toHaveAttribute(
      "src",
      expect.stringContaining("x-fly-aircraft-hero-sharp-v1.png"),
    );
  });

  it("keeps the complete hero usable when reduced motion is requested", () => {
    render(<Home />);

    const hero = screen.getByRole("region", {
      name: "Go anywhere. Fly different.",
    });

    expect(hero).toHaveTextContent("X-FLY ANYWAY · GLOBAL AVIATION");
    expect(hero).toHaveTextContent(
      "From the world’s great cities to what comes next. Travel without limits.",
    );
    expect(hero).not.toHaveAttribute("aria-hidden");
    expect(hero).not.toHaveStyle({ opacity: "0", visibility: "hidden" });
  });

  it("prepares an accessible motion headline and reverts it on unmount", () => {
    window.matchMedia = jest.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
      addListener: jest.fn(),
      removeListener: jest.fn(),
      dispatchEvent: jest.fn(),
    }));

    const { unmount } = render(<Home />);
    const heading = screen.getByRole("heading", {
      name: "Go anywhere. Fly different.",
    });
    const words = Array.from(heading.querySelectorAll(".word"));

    expect(words.length).toBeGreaterThan(0);
    expect(
      words.every((word) => word.getAttribute("aria-hidden") === "true"),
    ).toBe(true);

    unmount();

    expect(heading).toHaveTextContent("Go anywhere. Fly different.");
    expect(heading.querySelector(".word")).not.toBeInTheDocument();
  });
});
