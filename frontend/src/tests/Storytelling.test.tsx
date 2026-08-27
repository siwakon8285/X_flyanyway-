import { act, render, screen, within } from "@testing-library/react";

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
    ];

    expect(storySections.every(Boolean)).toBe(true);
    storySections.slice(0, -1).forEach((section, index) => {
      expect(
        section?.compareDocumentPosition(storySections[index + 1] as Node),
      ).toBe(Node.DOCUMENT_POSITION_FOLLOWING);
    });

    expect(
      screen.getByRole("heading", { level: 2, name: "The world, closer." }),
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
        name: "From first thought to final step.",
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
      expect.stringContaining("x-fly-global-reach-bg-v1.jpg"),
    );

    expect(within(globalReach as HTMLElement).getByText("156")).toBeInTheDocument();
    expect(
      within(globalReach as HTMLElement).getByText("COUNTRIES"),
    ).toBeInTheDocument();
    expect(
      within(globalReach as HTMLElement).queryByRole("button"),
    ).not.toBeInTheDocument();
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

  it("ends with a compact, route-ready conversion choice", () => {
    const { container } = render(<Home />);
    const journey = container.querySelector("#journey-experience");

    expect(journey).not.toBeNull();
    expect(
      within(journey as HTMLElement).getByRole("heading", {
        level: 3,
        name: "Ready to go anywhere?",
      }),
    ).toBeInTheDocument();
    expect(
      within(journey as HTMLElement).getByText("Your next journey starts here."),
    ).toBeInTheDocument();
    expect(
      within(journey as HTMLElement).getByRole("link", { name: "Book a Flight" }),
    ).toHaveAttribute("href", "#journey");
    expect(
      within(journey as HTMLElement).getByRole("link", { name: "Explore Cabins" }),
    ).toHaveAttribute("href", "#cabins");
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

      expect(container.querySelectorAll(".pin-spacer")).toHaveLength(1);
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
