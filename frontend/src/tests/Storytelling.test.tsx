import { act, screen, within } from "@testing-library/react";
import { render } from "@/tests/renderWithLanguage";

import Home from "@/app/page";
import { Storytelling } from "@/components/home/story/Storytelling";
import { REDUCED_MOTION_QUERY } from "@/lib/motion/reducedMotion";

const createMediaController = () => {
  let reducedMotion = false;
  let viewportHeight = 900;
  let viewportWidth = 1280;
  const listeners = new Map<MediaQueryList, Set<unknown>>();
  const lastMatches = new Map<MediaQueryList, boolean>();

  const callListener = (listener: unknown, event: MediaQueryListEvent) => {
    if (typeof listener === "function") {
      (listener as (change: MediaQueryListEvent) => void)(event);
    } else if (
      typeof listener === "object" &&
      listener !== null &&
      "handleEvent" in listener
    ) {
      (listener as EventListenerObject).handleEvent(event);
    }
  };

  const matches = (query: string) => {
    if (query === REDUCED_MOTION_QUERY) return reducedMotion;

    const minimum = query.match(/min-width:\s*([\d.]+)rem/);
    if (minimum?.[1] && viewportWidth < Number(minimum[1]) * 16) return false;

    const maximum = query.match(/max-width:\s*([\d.]+)rem/);
    if (maximum?.[1] && viewportWidth > Number(maximum[1]) * 16) return false;

    const minimumHeight = query.match(/min-height:\s*(\d+)rem/);
    if (
      minimumHeight?.[1] &&
      viewportHeight < Number(minimumHeight[1]) * 16
    ) {
      return false;
    }

    return true;
  };

  const matchMedia = jest.fn<MediaQueryList, [string]>((query) => {
    const queryListeners = new Set<unknown>();

    const mediaQuery = {
      addEventListener: (
        _type: string,
        listener: EventListenerOrEventListenerObject,
      ) => queryListeners.add(listener),
      addListener: (listener: MediaQueryList["onchange"]) => {
        if (listener) queryListeners.add(listener);
      },
      dispatchEvent: (event) => {
        queryListeners.forEach((listener) =>
          callListener(listener, event as MediaQueryListEvent),
        );
        return true;
      },
      get matches() {
        return matches(query);
      },
      media: query,
      onchange: null,
      removeEventListener: (
        _type: string,
        listener: EventListenerOrEventListenerObject,
      ) => queryListeners.delete(listener),
      removeListener: (listener: MediaQueryList["onchange"]) =>
        queryListeners.delete(listener),
    } as MediaQueryList;

    listeners.set(mediaQuery, queryListeners);
    lastMatches.set(mediaQuery, mediaQuery.matches);
    return mediaQuery;
  });

  const notify = () => {
    Array.from(listeners.entries()).forEach(([mediaQuery, queryListeners]) => {
      const nextMatch = mediaQuery.matches;
      if (lastMatches.get(mediaQuery) === nextMatch) return;

      lastMatches.set(mediaQuery, nextMatch);
      const event = {
        matches: nextMatch,
        media: mediaQuery.media,
        type: "change",
      } as MediaQueryListEvent;

      queryListeners.forEach((listener) => callListener(listener, event));
    });
  };

  return {
    matchMedia,
    setReducedMotion: (value: boolean) => {
      reducedMotion = value;
      notify();
    },
    setViewportHeight: (value: number) => {
      viewportHeight = value;
      notify();
    },
    setViewportWidth: (value: number) => {
      viewportWidth = value;
      notify();
    },
  };
};

describe("scroll storytelling", () => {
  it("renders the image-supported story beats in semantic order without replacing the hero headline", () => {
    const { container } = render(<Home />);

    expect(screen.getAllByRole("heading", { level: 1 })).toHaveLength(1);

    const storySections = [
      container.querySelector("#global-reach"),
      container.querySelector("#aircraft-story"),
      container.querySelector("#cabins"),
      container.querySelector("#service-story"),
      container.querySelector("#journey-path"),
      container.querySelector("#future-moon"),
      container.querySelector("#journey-experience"),
      container.querySelector("#flight-search"),
    ];

    expect(storySections.every(Boolean)).toBe(true);
    storySections.slice(0, -1).forEach((section, index) => {
      expect(
        section?.compareDocumentPosition(storySections[index + 1] as Node),
      ).toBe(Node.DOCUMENT_POSITION_FOLLOWING);
    });

    expect(
      screen.getByRole("heading", { level: 2, name: "The world, within reach." }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 2, name: "Choose your way to fly." }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 2, name: "Movement, made personal." }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", {
        level: 2,
        name: "A journey considered in every detail.",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", {
        level: 2,
        name: "A continuous journey.",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", {
        level: 2,
        name: /NEXT:\s+THE MOON\./i,
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", {
        level: 2,
        name: "Designed around the journey.",
      }),
    ).toBeInTheDocument();
    expect(screen.queryByText("The journey continues.")).not.toBeInTheDocument();

  });


  it("renders four spatial milestones inside the continuous aircraft corridor", () => {
    const { container } = render(<Home />);
    const aircraftStory = container.querySelector("#aircraft-story");

    expect(aircraftStory).not.toBeNull();
    expect(
      aircraftStory?.querySelector("[data-aircraft-flight-corridor]"),
    ).toBeInTheDocument();

    const aircraft = aircraftStory?.querySelector('img[alt=""]');
    expect(aircraft).toHaveAttribute(
      "src",
      expect.stringContaining("x-fly-aircraft-flythrough-v1.png"),
    );

    const beats = within(aircraftStory as HTMLElement).getAllByRole("listitem");
    expect(beats).toHaveLength(4);
    const expectedBeats = [
      [
        "GLOBAL",
        "GO FURTHER.",
        "One connected journey across continents and time zones.",
      ],
      [
        "PERSONAL",
        "FLY YOUR WAY.",
        "Space shaped around how you choose to travel.",
      ],
      [
        "PREMIUM",
        "ABOVE THE ORDINARY.",
        "Thoughtful comfort, considered at every altitude.",
      ],
      [
        "SEAMLESS",
        "FROM HERE TO THERE.",
        "Every detail considered from departure to arrival.",
      ],
    ] as const;

    expectedBeats.forEach(([label, headline, copy], index) => {
      expect(beats[index]).toHaveAttribute(
        "data-aircraft-zone",
        ["global", "personal", "premium", "seamless"][index],
      );
      expect(beats[index]).toHaveTextContent(label);
      expect(beats[index]).toHaveTextContent(headline);
      expect(beats[index]).toHaveTextContent(copy);
      expect(
        beats[index]?.querySelector("[data-aircraft-beat-meta]"),
      ).toHaveTextContent(
        `${String(index + 1).padStart(2, "0")} / 04 · ${label}`,
      );
      expect(
        beats[index]?.querySelector("[data-aircraft-beat-headline]"),
      ).toHaveTextContent(headline);
      expect(
        beats[index]?.querySelector("[data-aircraft-beat-copy]"),
      ).toHaveTextContent(copy);
    });

    expect(
      aircraftStory?.querySelector("[data-aircraft-milestone-canvas]"),
    ).toBeInTheDocument();
    expect(
      aircraftStory?.querySelector("[data-aircraft-milestone-canvas]"),
    ).not.toHaveClass("md:grid-cols-2", "md:grid-rows-2");
    expect(
      aircraftStory?.querySelector("[data-aircraft-milestone-canvas]"),
    ).toHaveClass(
      "motion-safe:[@media(min-height:52rem)]:lg:grid-cols-2",
      "motion-safe:[@media(min-height:52rem)]:lg:grid-rows-2",
    );
    expect(
      aircraftStory?.querySelector("[data-aircraft-safe-canvas]"),
    ).toBeInTheDocument();
    expect(
      aircraftStory?.querySelector("[data-aircraft-sticky-canvas]"),
    ).not.toHaveClass("md:sticky", "md:h-svh");
    for (const beat of beats) {
      expect(beat).not.toHaveClass("md:absolute");
    }
    expect(
      aircraftStory?.querySelector("[data-aircraft-progress]"),
    ).not.toBeInTheDocument();
  });

  it("uses photographic Global Reach visual while preserving the 156-country message", () => {
    const { container } = render(<Home />);
    const globalReach = container.querySelector("#global-reach");

    expect(globalReach).not.toBeNull();
    expect(
      globalReach?.querySelector("svg[data-world-map]"),
    ).not.toBeInTheDocument();

    const globalImgs = globalReach?.querySelectorAll("img");
    expect(globalImgs?.length).toBeGreaterThanOrEqual(1);
    expect(globalImgs?.[0]).toHaveAttribute(
      "src",
      expect.stringContaining("x-fly-global-reach-luxury.jpg"),
    );

    expect(within(globalReach as HTMLElement).getByText("156")).toBeInTheDocument();
    expect(
      within(globalReach as HTMLElement).getByText("COUNTRIES"),
    ).toBeInTheDocument();
    expect(
      within(globalReach as HTMLElement).queryByRole("button"),
    ).not.toBeInTheDocument();
  });

  it("presents Global Reach as a grounded, accessible travel network", () => {
    const { container } = render(<Home />);
    const globalReach = container.querySelector("#global-reach");

    expect(globalReach).not.toBeNull();
    expect(globalReach).toHaveAttribute("data-travel-network", "cartographic");
    expect(globalReach?.querySelector("[data-global-globe]"))
      .not.toBeInTheDocument();
    expect(globalReach?.querySelector("[data-global-space]"))
      .not.toBeInTheDocument();
    expect(
      within(globalReach as HTMLElement).getByText(
        "Connected across every horizon.",
        { exact: false },
      ),
    ).toBeInTheDocument();
    expect(
      within(globalReach as HTMLElement).getByText("7 GLOBAL HUBS"),
    ).toBeVisible();
    expect(
      within(globalReach as HTMLElement).getByText("24/7 OPERATIONS"),
    ).toBeVisible();
  });

  it("exposes immutable stage-owned Journey slots for persistent cards", () => {
    const { container } = render(<Home />);
    const journey = container.querySelector("#journey-path");
    const desktop = journey?.querySelector("[data-journey-desktop]");
    const stages = Array.from(
      desktop?.querySelectorAll<HTMLElement>("[data-journey-stage]") ?? [],
    );

    expect(journey).not.toBeNull();
    expect(desktop).toHaveAttribute("data-desktop-pin", "bounded");
    expect(desktop).toHaveAttribute("data-promotion-duration", "0.88");
    expect(desktop).toHaveAttribute("data-pair-starts", "0.55,2.45");
    expect(desktop).toHaveAttribute("data-copy-change-offset", "0.98");
    expect(desktop).toHaveAttribute("data-scroll-distance-vh", "2.3");
    expect(desktop).toHaveAttribute("data-scrub-smoothing", "0.45");
    expect(desktop).toHaveAttribute("data-interpolation-ease", "none");
    expect(stages).toHaveLength(2);
    expect(stages.map((stage) => stage.dataset.journeyStage)).toEqual([
      "left",
      "right",
    ]);
    expect(stages.map((stage) => stage.dataset.stageEmphasis)).toEqual([
      "secondary",
      "primary",
    ]);

    for (const stage of stages) {
      const anchors = Array.from(
        stage.querySelectorAll<HTMLElement>("[data-journey-anchor]"),
      );
      const cards = Array.from(
        stage.querySelectorAll<HTMLElement>("[data-journey-card]"),
      );
      const cardIds = cards.map((card) => card.dataset.journeyCardId);

      expect(cards).toHaveLength(4);
      expect(stage).toHaveAttribute("data-stage-bounds", "contained");
      expect(stage).toHaveAttribute("data-slot-model", "immutable");
      expect(anchors.map((anchor) => anchor.dataset.journeyAnchor)).toEqual([
        "front",
        "queued",
        "deep",
        "off-deck",
      ]);
      anchors.forEach((anchor) => {
        expect(anchor).toHaveAttribute("data-anchor-bounds", "contained");
        expect(anchor).toHaveAttribute(
          "data-slot-id",
          `${stage.dataset.journeyStage}-${anchor.dataset.journeyAnchor}`,
        );
        expect(anchor).toHaveAttribute(
          "data-slot-owner",
          `${stage.dataset.journeyStage}-stage`,
        );
        expect(anchor).toHaveAttribute("data-slot-geometry", "absolute");
        expect(anchor.style.top).not.toBe("");
        expect(anchor.style.height).not.toBe("");
      });
      expect(stage.querySelector('[data-journey-anchor="front"]')).toHaveAttribute(
        "data-visual-layer",
        stage.dataset.journeyStage === "right"
          ? "foremost-front"
          : "supporting-front",
      );
      expect(stage.querySelector('[data-journey-anchor="queued"]')).toHaveAttribute(
        "data-slot-direction",
        "inward",
      );
      expect(stage.querySelector('[data-journey-anchor="queued"]')).toHaveAttribute(
        "data-composition-zone",
        "center-biased",
      );
      expect(stage.querySelector('[data-journey-anchor="queued"]')).toHaveAttribute(
        "data-depth-rank",
        stage.dataset.journeyStage === "left" ? "queued-near" : "queued-far",
      );
      expect(stage.querySelector('[data-journey-anchor="deep"]')).toHaveAttribute(
        "data-slot-direction",
        "inward",
      );
      expect(stage.querySelector('[data-journey-anchor="deep"]')).toHaveAttribute(
        "data-depth-rank",
        stage.dataset.journeyStage === "left" ? "deep-near" : "deep-far",
      );
      expect(cardIds.every(Boolean)).toBe(true);
      expect(new Set(cardIds).size).toBe(cards.length);
      cards.forEach((card) => {
        expect(card).toHaveAttribute("data-persistent-card", "true");
        expect(card).toHaveAttribute("data-layout-owner", "stage-slot");
        expect(card).toHaveAttribute("data-transform-accumulation", "none");
        expect(card.style.transform).toBe("");
        expect(card.style.top).not.toBe("");
        expect(card.style.height).not.toBe("");
      });
      expect(stage.querySelector('[data-depth-role="front"]')).toBeVisible();
      expect(stage.querySelector('[data-depth-role="queued"]')).toBeVisible();
      expect(stage.querySelector('[data-depth-role="deep"]')).toBeVisible();
      expect(stage.querySelector('[data-depth-role="off-deck"]')).toBeInTheDocument();
    }
  });

  it("promotes persistent Journey pairs together into the shared front role", () => {
    const { container } = render(<Home />);
    const desktop = container.querySelector("[data-journey-desktop]");
    const stages = Array.from(
      desktop?.querySelectorAll<HTMLElement>("[data-journey-stage]") ?? [],
    );

    const promotionBeats = Array.from(
      desktop?.querySelectorAll<HTMLElement>("[data-promotion-beat]") ?? [],
    );
    expect(promotionBeats).toHaveLength(2);
    expect(
      promotionBeats.map((beat) => [
        beat.dataset.promotionMode,
        beat.dataset.promotesStackIndex,
        beat.dataset.chapterAdvance,
        beat.dataset.leftTargetSlot,
        beat.dataset.rightTargetSlot,
      ]),
    ).toEqual([
      ["pair", "1", "move", "left-front", "right-front"],
      ["pair", "2", "arrive", "left-front", "right-front"],
    ]);

    promotionBeats.forEach((beat) => {
      expect(beat).not.toHaveAttribute("data-promotion-side");
      expect(beat).toHaveAttribute("data-slot-interpolation", "continuous");

      stages.forEach((stage) => {
        const promotedCard = stage.querySelector(
          `[data-stack-index="${beat.dataset.promotesStackIndex}"]`,
        );

        expect(promotedCard).toBeInTheDocument();
        expect(promotedCard).toHaveAttribute("data-persistent-card", "true");
        expect(beat).toHaveAttribute(
          `data-${stage.dataset.journeyStage}-card-id`,
          promotedCard?.getAttribute("data-journey-card-id"),
        );
      });
    });
  });

  it("keeps Journey mobile and reduced-motion content in normal document flow", () => {
    const { container } = render(<Home />);
    const journey = container.querySelector("#journey-path");
    const fallback = journey?.querySelector("[data-journey-mobile]");

    expect(fallback).toHaveAttribute("data-mobile-flow", "vertical");
    expect(fallback).toHaveAttribute("data-reduced-motion-fallback", "true");
    expect(fallback?.querySelectorAll("[data-mobile-chapter]")).toHaveLength(3);
    expect(journey?.outerHTML).not.toContain("overflow-x-auto");
    expect(journey?.outerHTML).not.toContain("snap-x");
  });

  it("uses the approved interior and service assets without reusing aircraft imagery", () => {
    const { container } = render(<Home />);
    const serviceStory = container.querySelector("#service-story");

    expect(serviceStory).not.toBeNull();
    expect(
      within(serviceStory as HTMLElement).getByText("X-FLY SERVICE"),
    ).toBeInTheDocument();

    const serviceImages = serviceStory?.querySelectorAll("img");
    expect(serviceImages).toHaveLength(2);
    expect(serviceImages?.[0]).toHaveAttribute(
      "src",
      expect.stringContaining("x-fly-interior-premium-v1.png"),
    );
    expect(serviceImages?.[1]).toHaveAttribute(
      "src",
      expect.stringContaining("x-fly-service-dining-v1.png"),
    );

    for (const image of Array.from(serviceImages ?? [])) {
      expect(image.getAttribute("src")).not.toContain("aircraft");
    }

    expect(
      within(serviceStory as HTMLElement).getByText("Cabin comfort"),
    ).toBeInTheDocument();
    expect(
      within(serviceStory as HTMLElement).getByText(
        "Personal service",
      ),
    ).toBeInTheDocument();
    expect(
      serviceStory?.querySelector("[data-service-layout]"),
    ).toBeInTheDocument();
    expect(
      serviceStory?.querySelectorAll("[data-service-heading-line]"),
    ).toHaveLength(2);
    expect(
      serviceStory?.querySelector("[data-service-eyebrow-line]"),
    ).toBeInTheDocument();
    expect(
      serviceStory?.querySelectorAll("[data-service-image-frame]"),
    ).toHaveLength(2);
    expect(
      serviceStory?.querySelectorAll("[data-service-image-parallax]"),
    ).toHaveLength(2);
    expect(
      serviceStory?.querySelectorAll("[data-service-caption-rule]"),
    ).toHaveLength(2);
    expect(
      serviceStory?.querySelector("[data-service-transition-entry]"),
    ).toBeInTheDocument();
    expect(
      serviceStory?.querySelector("[data-service-transition-exit]"),
    ).toBeInTheDocument();
    expect(
      serviceStory?.querySelector("[data-service-sticky-canvas]"),
    ).not.toBeInTheDocument();
    expect(serviceStory?.outerHTML).not.toContain("sticky");
    expect(serviceStory?.outerHTML).not.toContain("180svh");
    expect(serviceStory?.outerHTML).not.toContain("240svh");
    expect(
      serviceStory?.querySelector('[data-service-panel="primary"]'),
    ).toContainElement(serviceImages?.[0] ?? null);
    expect(
      serviceStory?.querySelector('[data-service-panel="secondary"]'),
    ).toContainElement(serviceImages?.[1] ?? null);
    expect(
      serviceStory?.querySelector("[data-service-detail]"),
    ).not.toBeInTheDocument();
    expect(
      serviceStory?.querySelector('[data-service-panel="secondary"]'),
    ).toHaveClass("md:mt-24", "lg:mt-36");
  });

  it("ends storytelling with the brand summary before the single flight search", () => {
    const { container } = render(<Home />);
    const journey = container.querySelector("#journey-experience");
    const flightSearch = container.querySelector("#flight-search");

    expect(journey).not.toBeNull();
    expect(flightSearch).not.toBeNull();
    expect(
      within(journey as HTMLElement).getByRole("heading", {
        level: 2,
        name: "Designed around the journey.",
      }),
    ).toBeInTheDocument();
    expect(within(journey as HTMLElement).getByText("Comfort")).toBeInTheDocument();
    expect(within(journey as HTMLElement).getByText("Control")).toBeInTheDocument();
    expect(within(journey as HTMLElement).getByText("Choice")).toBeInTheDocument();
    expect(within(journey as HTMLElement).queryByText("Your next horizon")).not.toBeInTheDocument();
    expect(within(journey as HTMLElement).queryByText("Ready to go anywhere?")).not.toBeInTheDocument();
    expect(within(journey as HTMLElement).queryByText("Your next journey starts here.")).not.toBeInTheDocument();
    expect(within(journey as HTMLElement).queryByRole("link", { name: "Book a Flight" })).not.toBeInTheDocument();
    expect(within(journey as HTMLElement).queryByRole("link", { name: "Explore Cabins" })).not.toBeInTheDocument();
    expect(journey?.querySelector("[data-journey-cta]")).not.toBeInTheDocument();
    expect(journey?.compareDocumentPosition(flightSearch as Node)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
    expect(container.querySelectorAll("#flight-search")).toHaveLength(1);
  });

  it("communicates global scale and presents 4 dedicated cabin tiers with photography and without interactive controls", () => {
    const { container } = render(<Home />);
    const globalReach = container.querySelector("#global-reach");
    const cabins = container.querySelector("#cabins");

    expect(globalReach).not.toBeNull();
    expect(cabins).not.toBeNull();
    expect(within(globalReach as HTMLElement).getByText("156")).toBeInTheDocument();

    const stages = Array.from(cabins?.querySelectorAll("[data-cabin-stage]") ?? []);
    expect(stages).toHaveLength(4);

    for (const stage of stages) {
      expect(within(stage as HTMLElement).queryByRole("button")).not.toBeInTheDocument();
      expect(within(stage as HTMLElement).queryByRole("radio")).not.toBeInTheDocument();
    }

    const cabinLabels = ["Economy", "Premium Economy", "Business", "First"];
    cabinLabels.forEach((label) => {
      expect(within(cabins as HTMLElement).getByRole("heading", { level: 3, name: label })).toBeInTheDocument();
    });

    const cabinImages = cabins?.querySelectorAll("img");
    expect(cabinImages?.length).toBeGreaterThanOrEqual(4);
    expect(cabins?.querySelectorAll("[data-cabin-atmosphere]")).toHaveLength(4);
    expect(cabins?.querySelector("[data-cabin-media-frame]")).toBeInTheDocument();
  });

  it("keeps reduced-motion storytelling visible in normal document flow", () => {
    const { container } = render(<Home />);
    const stages = Array.from(container.querySelectorAll("[data-cabin-stage]"));
    const aircraft = container.querySelector("[data-story-aircraft]");
    const servicePanels = Array.from(
      container.querySelectorAll("[data-service-panel]"),
    );

    expect(stages).toHaveLength(4);
    for (const stage of stages) {
      expect(stage).toBeVisible();
      expect(stage).not.toHaveStyle({ opacity: "0" });
      expect(stage).not.toHaveStyle({ position: "absolute" });
    }
    expect(aircraft).toBeVisible();
    expect(aircraft).not.toHaveStyle({ opacity: "0" });
    expect(servicePanels).toHaveLength(2);
    for (const panel of servicePanels) {
      expect(panel).toBeVisible();
      expect(panel).not.toHaveStyle({ opacity: "0" });
      expect((panel as HTMLElement).style.clipPath).toBe("");
    }
    expect(container.querySelector(".pin-spacer")).not.toBeInTheDocument();
  });

  it("removes the cabin pin and restores normal flow across responsive and motion changes", () => {
    const initialMatchMedia = window.matchMedia;
    const media = createMediaController();
    const originalNow = Date.now();
    let mediaEventTime = originalNow;
    const now = jest.spyOn(Date, "now").mockImplementation(() => mediaEventTime);
    window.matchMedia = media.matchMedia;

    const { container, unmount } = render(<Storytelling />);

    try {
      const serviceFrames = Array.from(
        container.querySelectorAll<HTMLElement>("[data-service-image-frame]"),
      );
      const serviceImages = Array.from(
        container.querySelectorAll<HTMLElement>("[data-service-image]"),
      );
      const serviceParallaxLayers = Array.from(
        container.querySelectorAll<HTMLElement>(
          "[data-service-image-parallax]",
        ),
      );

      expect(container.querySelectorAll(".pin-spacer")).toHaveLength(2);
      expect(
        container.querySelectorAll(
          '[data-aircraft-beat][style*="position: absolute"]',
        ),
      ).toHaveLength(0);
      expect(container.querySelectorAll('[data-cabin-stage][style*="position: absolute"]')).toHaveLength(
        4,
      );
      for (const frame of serviceFrames) {
        expect(frame.style.clipPath).not.toContain("100%");
      }
      const desktopParallaxTransform = serviceParallaxLayers[0]?.style.transform;

      mediaEventTime += 10;
      act(() => media.setViewportWidth(800));

      expect(container.querySelectorAll(".pin-spacer")).toHaveLength(1);
      expect(serviceParallaxLayers[0]?.style.transform).not.toBe(
        desktopParallaxTransform,
      );

      const tabletParallaxTransform = serviceParallaxLayers[0]?.style.transform;

      mediaEventTime += 10;
      act(() => media.setViewportWidth(1280));

      expect(container.querySelectorAll(".pin-spacer")).toHaveLength(2);
      expect(serviceParallaxLayers[0]?.style.transform).not.toBe(
        tabletParallaxTransform,
      );

      mediaEventTime += 10;
      act(() => media.setViewportWidth(800));

      for (const beat of Array.from(
        container.querySelectorAll<HTMLElement>("[data-aircraft-beat]"),
      )) {
        expect(beat.style.opacity).toBe("");
        expect(beat.style.transform).toBe("");
      }

      mediaEventTime += 10;
      act(() => media.setViewportWidth(390));

      expect(container.querySelector(".pin-spacer")).not.toBeInTheDocument();
      for (const frame of serviceFrames) {
        expect(frame.style.clipPath).toBe("");
      }
      for (const image of serviceImages) {
        expect(image.style.transform).toBe("");
      }
      for (const layer of serviceParallaxLayers) {
        expect(layer.style.transform).toBe("");
      }
      expect(
        container.querySelectorAll(
          '[data-aircraft-beat][style*="position: absolute"]',
        ),
      ).toHaveLength(0);
      expect(container.querySelectorAll('[data-cabin-stage][style*="position: absolute"]')).toHaveLength(
        0,
      );

      mediaEventTime += 10;
      act(() => media.setViewportWidth(1280));
      expect(container.querySelectorAll(".pin-spacer")).toHaveLength(2);
      expect(
        container.querySelectorAll(
          '[data-aircraft-beat][style*="position: absolute"]',
        ),
      ).toHaveLength(0);

      mediaEventTime += 10;
      act(() => media.setViewportHeight(700));

      expect(container.querySelectorAll(".pin-spacer")).toHaveLength(2);
      for (const beat of Array.from(
        container.querySelectorAll<HTMLElement>("[data-aircraft-beat]"),
      )) {
        expect(beat.style.opacity).toBe("");
        expect(beat.style.transform).toBe("");
      }

      mediaEventTime += 10;
      act(() => media.setViewportHeight(900));
      expect(container.querySelectorAll(".pin-spacer")).toHaveLength(2);

      mediaEventTime += 10;
      act(() => media.setReducedMotion(true));

      expect(container.querySelector(".pin-spacer")).not.toBeInTheDocument();
      for (const frame of serviceFrames) {
        expect(frame.style.clipPath).toBe("");
      }
      for (const image of serviceImages) {
        expect(image.style.transform).toBe("");
      }
      for (const layer of serviceParallaxLayers) {
        expect(layer.style.transform).toBe("");
      }
      expect(
        container.querySelectorAll(
          '[data-aircraft-beat][style*="position: absolute"]',
        ),
      ).toHaveLength(0);
      expect(container.querySelectorAll('[data-cabin-stage][style*="position: absolute"]')).toHaveLength(
        0,
      );
    } finally {
      unmount();
      now.mockRestore();
      window.matchMedia = initialMatchMedia;
    }

    expect(document.querySelector(".pin-spacer")).not.toBeInTheDocument();
  });
});
