import { act, fireEvent, render, screen, within } from "@testing-library/react";

import Home from "@/app/page";
import {
  HorizontalJourneyStory,
  journeyStages,
} from "@/components/home/story/HorizontalJourneyStory";

describe("Horizontal Journey Section", () => {
  it("renders 5 journey stages in semantic order", () => {
    const { container } = render(<HorizontalJourneyStory />);
    const section = container.querySelector("#journey-path");

    expect(section).not.toBeNull();
    const stages = within(section as HTMLElement).getAllByRole("listitem");
    expect(stages).toHaveLength(5);

    const expectedStages = [
      { id: "discover", name: "Where the journey begins." },
      { id: "book", name: "One considered path." },
      { id: "fly", name: "Space at 38,000 feet." },
      { id: "arrive", name: "A seamless arrival." },
      { id: "beyond", name: "The next horizon." },
    ];

    expectedStages.forEach((stage, index) => {
      expect(stages[index]).toHaveAttribute("data-journey-stage", stage.id);
      expect(
        within(stages[index]).getByRole("heading", {
          level: 3,
          name: stage.name,
        }),
      ).toBeInTheDocument();
    });
  });

  it("uses an accessible section H2 without replacing the hero H1", () => {
    render(<Home />);

    const h1s = screen.getAllByRole("heading", { level: 1 });
    expect(h1s).toHaveLength(1);

    expect(
      screen.getByRole("heading", {
        level: 2,
        name: "From first thought to final step.",
      }),
    ).toBeInTheDocument();
  });

  it("contains no interactive buttons, radios, or booking inputs in Book stage", () => {
    const { container } = render(<HorizontalJourneyStory />);
    const section = container.querySelector("#journey-path");
    const bookStage = section?.querySelector('[data-journey-stage="book"]');

    expect(bookStage).not.toBeNull();
    expect(
      within(bookStage as HTMLElement).queryByRole("button"),
    ).not.toBeInTheDocument();
    expect(
      within(bookStage as HTMLElement).queryByRole("combobox"),
    ).not.toBeInTheDocument();
    expect(
      within(bookStage as HTMLElement).queryByRole("textbox"),
    ).not.toBeInTheDocument();
    expect(
      within(bookStage as HTMLElement).queryByRole("radio"),
    ).not.toBeInTheDocument();
  });

  it("provides balanced right-side photographic visuals across all 5 stages", () => {
    const { container } = render(<HorizontalJourneyStory />);
    const section = container.querySelector("#journey-path");

    expect(section).not.toBeNull();

    for (const stageId of ["discover", "book", "fly", "arrive", "beyond"]) {
      const stage = section?.querySelector(`[data-journey-stage="${stageId}"]`);
      expect(stage).not.toBeNull();
      const visual = stage?.querySelector(`[data-journey-visual="${stageId}"]`);
      expect(visual).toBeInTheDocument();
      expect(visual?.querySelector("img")).toBeInTheDocument();
    }

    // Discover image
    const discoverStage = section?.querySelector('[data-journey-stage="discover"]');
    const discoverImg = discoverStage?.querySelector("img");
    expect(discoverImg).toHaveAttribute(
      "src",
      expect.stringContaining("x-fly-journey-discover-v1.jpg"),
    );

    // Book image
    const bookStage = section?.querySelector('[data-journey-stage="book"]');
    const bookImg = bookStage?.querySelector("img");
    expect(bookImg).toHaveAttribute(
      "src",
      expect.stringContaining("x-fly-journey-book-v1.jpg"),
    );

    // Fly image
    const flyStage = section?.querySelector('[data-journey-stage="fly"]');
    const flyImg = flyStage?.querySelector("img");
    expect(flyImg).toHaveAttribute(
      "src",
      expect.stringContaining("x-fly-journey-fly-v1.jpg"),
    );

    // Arrive image
    const arriveStage = section?.querySelector('[data-journey-stage="arrive"]');
    const arriveImg = arriveStage?.querySelector("img");
    expect(arriveImg).toHaveAttribute(
      "src",
      expect.stringContaining("x-fly-journey-arrive-v1.jpg"),
    );

    // Beyond image
    const beyondStage = section?.querySelector('[data-journey-stage="beyond"]');
    const beyondImg = beyondStage?.querySelector("img");
    expect(beyondImg).toHaveAttribute(
      "src",
      expect.stringContaining("x-fly-journey-beyond-v1.jpg"),
    );
  });

  it("renders 5 mobile stage navigation controls with accessible labels and supports tapping", () => {
    const { container } = render(<HorizontalJourneyStory />);
    const mobileNav = container.querySelector("[data-journey-mobile-nav]");

    expect(mobileNav).not.toBeNull();
    const buttons = within(mobileNav as HTMLElement).getAllByRole("button");
    expect(buttons).toHaveLength(5);

    const expectedLabels = [
      "Go to Discover",
      "Go to Book",
      "Go to Fly",
      "Go to Arrive",
      "Go to Beyond",
    ];

    expectedLabels.forEach((label, index) => {
      expect(buttons[index]).toHaveAttribute("aria-label", label);
    });

    // First button has aria-current="step" by default
    expect(buttons[0]).toHaveAttribute("aria-current", "step");

    // Mock scrollIntoView for stage elements
    const scrollIntoViewMock = jest.fn();
    const originalScrollIntoView = Element.prototype.scrollIntoView;
    Element.prototype.scrollIntoView = scrollIntoViewMock;

    try {
      fireEvent.click(buttons[2]); // click Fly stage
      expect(scrollIntoViewMock).toHaveBeenCalledWith(
        expect.objectContaining({
          behavior: "smooth",
        }),
      );
    } finally {
      Element.prototype.scrollIntoView = originalScrollIntoView;
    }
  });

  it("contains no autoplay timers or intervals for sliding", () => {
    jest.useFakeTimers();
    const setIntervalSpy = jest.spyOn(window, "setInterval");

    render(<HorizontalJourneyStory />);

    // Fast-forward time to ensure no automated slide timers were registered
    jest.advanceTimersByTime(10000);

    expect(setIntervalSpy).not.toHaveBeenCalled();

    setIntervalSpy.mockRestore();
    jest.useRealTimers();
  });

  it("includes a subtle swipe exploration hint on the first card", () => {
    const { container } = render(<HorizontalJourneyStory />);
    const firstStage = container.querySelector('[data-journey-stage="discover"]');

    expect(firstStage).toHaveTextContent("SWIPE TO EXPLORE →");
  });

  it("updates active navigation step when a stage intersects", () => {
    let observerCallback: IntersectionObserverCallback = () => undefined;
    const observeMock = jest.fn();
    const disconnectMock = jest.fn();

    class MockIntersectionObserver implements IntersectionObserver {
      readonly root: Element | Document | null = null;
      readonly rootMargin: string = "";
      readonly thresholds: readonly number[] = [];
      constructor(callback: IntersectionObserverCallback) {
        observerCallback = callback;
      }
      observe = observeMock;
      unobserve = jest.fn();
      disconnect = disconnectMock;
      takeRecords = () => [];
    }

    const originalIO = window.IntersectionObserver;
    window.IntersectionObserver = MockIntersectionObserver;

    try {
      const { container } = render(<HorizontalJourneyStory />);
      const mobileNav = container.querySelector("[data-journey-mobile-nav]");
      const buttons = within(mobileNav as HTMLElement).getAllByRole("button");
      expect(buttons[0]).toHaveAttribute("aria-current", "step");

      const bookStage = container.querySelector('[data-journey-stage="book"]');
      if (bookStage) {
        act(() => {
          observerCallback(
            [
              {
                isIntersecting: true,
                target: bookStage,
              } as unknown as IntersectionObserverEntry,
            ],
            {} as IntersectionObserver,
          );
        });

        expect(buttons[1]).toHaveAttribute("aria-current", "step");
        expect(buttons[0]).not.toHaveAttribute("aria-current");
      }
    } finally {
      window.IntersectionObserver = originalIO;
    }
  });

  it("marks HUD and decorative graphics as aria-hidden", () => {
    const { container } = render(<HorizontalJourneyStory />);
    const hud = container.querySelector("[data-journey-hud]");

    expect(hud).toHaveAttribute("aria-hidden", "true");
  });

  it("uses minimal dot pagination without visible numbers or text inside buttons", () => {
    const { container } = render(<HorizontalJourneyStory />);
    const mobileNav = container.querySelector("[data-journey-mobile-nav]");
    const buttons = within(mobileNav as HTMLElement).getAllByRole("button");

    expect(buttons).toHaveLength(5);
    buttons.forEach((button) => {
      expect(button.textContent?.trim()).toBe("");
    });
  });

  it("contains left-to-right reveal animation selectors and headline masks on each stage", () => {
    const { container } = render(<HorizontalJourneyStory />);
    const stages = container.querySelectorAll("[data-journey-stage]");

    expect(stages).toHaveLength(5);
    stages.forEach((stage) => {
      expect(stage.querySelector("[data-journey-content]")).toBeInTheDocument();
      expect(stage.querySelector("[data-journey-eyebrow]")).toBeInTheDocument();
      expect(stage.querySelector("[data-journey-eyebrow-line]")).toBeInTheDocument();
      expect(stage.querySelector("[data-journey-eyebrow-text]")).toBeInTheDocument();
      expect(stage.querySelector("[data-journey-headline-mask]")).toBeInTheDocument();
      expect(stage.querySelector("[data-journey-headline]")).toBeInTheDocument();
      expect(stage.querySelector("[data-journey-body]")).toBeInTheDocument();
      expect(stage.querySelector("[data-journey-meta]")).toBeInTheDocument();
    });
  });

  it("does not render a duplicate internal pseudo-header near the top site logo", () => {
    const { container } = render(<HorizontalJourneyStory />);
    const hud = container.querySelector("[data-journey-hud]");

    expect(hud).not.toBeNull();
    expect(within(hud as HTMLElement).queryByText("X-FLY JOURNEY")).not.toBeInTheDocument();
    expect(within(hud as HTMLElement).queryByText("// FIVE STAGES")).not.toBeInTheDocument();
  });

  it("does not render horizontal divider lines across metadata blocks", () => {
    const { container } = render(<HorizontalJourneyStory />);
    const metaBlocks = container.querySelectorAll("[data-journey-meta]");

    metaBlocks.forEach((meta) => {
      expect(meta.className).not.toContain("border-t");
    });
  });

  it("exports journey stage data correctly", () => {
    expect(journeyStages).toHaveLength(5);
    expect(journeyStages.map((s) => s.id)).toEqual([
      "discover",
      "book",
      "fly",
      "arrive",
      "beyond",
    ]);
  });
});


