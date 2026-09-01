import { fireEvent, screen, within } from "@testing-library/react";
import { render } from "@/tests/renderWithLanguage";

import Home from "@/app/page";
import { LanguageToggle } from "@/components/layout/LanguageToggle";

describe("cinematic hero", () => {
  it("renders one public headline, the booking form, and an aircraft media layer", () => {
    const { container } = render(<Home />);

    const headings = screen.getAllByRole("heading", { level: 1 });
    expect(headings).toHaveLength(1);
    expect(headings[0]).toHaveAccessibleName(
      "Go anywhere. Fly different.",
    );

    const hero = screen.getByRole("region", {
      name: "Go anywhere. Fly different.",
    });
    expect(hero).toHaveAttribute("id", "top");
    expect(container.querySelectorAll("#top")).toHaveLength(1);

    expect(within(hero).getByRole("form", { name: "Search Earth flights" })).toBeInTheDocument();
    expect(container.querySelector("#journey")).toBeInTheDocument();
    expect(container.querySelector("#flight-search")).toBeInTheDocument();
    for (const id of ["explore", "offers", "cabins", "experience"]) {
      expect(container.querySelectorAll(`#${id}`)).toHaveLength(1);
    }
    const heroVisual = container.querySelector("[data-hero-visual]");
    const heroSearch = container.querySelector("[data-hero-search]");
    expect(heroVisual).toHaveClass("relative", "min-h-svh", "overflow-hidden");
    expect(heroSearch).toHaveClass(
      "relative",
      "z-20",
      "-mt-16",
      "sm:-mt-20",
      "lg:-mt-24",
    );
    expect(heroVisual?.compareDocumentPosition(heroSearch as Node)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
    expect(container.querySelector("#global-reach")).not.toBeInTheDocument();

    const mediaLayer = container.querySelector("[data-hero-media]");
    expect(mediaLayer).toBeInTheDocument();
    expect(mediaLayer).toHaveClass("absolute", "inset-0", "overflow-hidden");
    const mediaFrame = mediaLayer?.querySelector("[data-hero-media-frame]");
    expect(mediaFrame).toHaveClass(
      "absolute",
      "-inset-y-3",
      "inset-x-0",
    );
    expect(mediaFrame).not.toHaveClass("will-change-transform");
    const aircraft = mediaLayer?.querySelector('img[alt=""]');
    expect(aircraft).toHaveClass(
      "object-cover",
      "object-[73%_center]",
      "sm:object-[69%_center]",
      "lg:object-center",
    );
    expect(aircraft).toHaveAttribute(
      "src",
      "/images/hero/x-fly-aircraft-hero-sharp-v1.png",
    );
    expect(aircraft?.getAttribute("style")).not.toContain("filter");
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

  it("updates the visible split headline immediately from English to Thai and back", () => {
    render(
      <>
        <LanguageToggle />
        <Home />
      </>,
    );

    const english = "Go anywhere. Fly different.";
    const thai = "ไปได้ทุกที่ บินในแบบที่แตกต่าง";
    const visibleText = (element: HTMLElement) =>
      element.textContent?.replaceAll(/\s/g, "");
    expect(visibleText(screen.getByRole("heading", { level: 1, name: english }))).toBe(
      english.replaceAll(/\s/g, ""),
    );

    fireEvent.click(
      screen.getByRole("button", {
        name: "Current language: English. Switch to Thai.",
      }),
    );

    const thaiHeading = screen.getByRole("heading", { level: 1, name: thai });
    expect(visibleText(thaiHeading)).toBe(thai.replaceAll(/\s/g, ""));
    expect(thaiHeading.querySelector(".word .word")).not.toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", {
        name: "ภาษาปัจจุบัน: ไทย เปลี่ยนเป็นภาษาอังกฤษ",
      }),
    );

    expect(visibleText(screen.getByRole("heading", { level: 1, name: english }))).toBe(
      english.replaceAll(/\s/g, ""),
    );
  });
});
