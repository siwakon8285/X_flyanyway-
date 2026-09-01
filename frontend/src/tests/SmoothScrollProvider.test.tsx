import { render, screen, waitFor } from "@testing-library/react";
import Lenis from "lenis";

import { SmoothScrollProvider } from "@/components/motion/SmoothScrollProvider";
import { INITIAL_HASH_POSITIONING_ATTRIBUTE } from "@/lib/motion/initialHash";
import { ScrollTrigger } from "@/lib/motion/gsap";

jest.mock("@/lib/motion/gsap", () => ({
  ScrollTrigger: { refresh: jest.fn(), update: jest.fn() },
  gsap: {
    ticker: { add: jest.fn(), remove: jest.fn() },
  },
}));

jest.mock("lenis", () => ({
  __esModule: true,
  default: jest.fn(() => ({
    destroy: jest.fn(),
    off: jest.fn(),
    on: jest.fn(),
    raf: jest.fn(),
    scrollTo: jest.fn(),
  })),
}));

describe("SmoothScrollProvider", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    document.documentElement.removeAttribute(INITIAL_HASH_POSITIONING_ATTRIBUTE);
    window.history.replaceState(null, "", "/");
  });

  it("owns one Lenis instance and destroys it on unmount", () => {
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

    const { rerender, unmount } = render(
      <SmoothScrollProvider>
        <p>Native content remains available.</p>
      </SmoothScrollProvider>,
    );

    rerender(
      <SmoothScrollProvider>
        <p>Native content remains available.</p>
      </SmoothScrollProvider>,
    );

    expect(screen.getByText("Native content remains available.")).toBeInTheDocument();
    const mockedLenis = jest.mocked(Lenis);
    expect(mockedLenis).toHaveBeenCalledTimes(1);

    unmount();
    const instance = mockedLenis.mock.results[0]?.value;
    expect(instance?.destroy).toHaveBeenCalledTimes(1);
  });

  it("settles a direct hash load through the owned Lenis instance after layout", () => {
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
    window.history.replaceState(null, "", "/?from=DXB#flight-search");
    document.documentElement.setAttribute(INITIAL_HASH_POSITIONING_ATTRIBUTE, "");
    let markerWasPresentDuringScroll = false;
    const requestAnimationFrame = jest
      .spyOn(window, "requestAnimationFrame")
      .mockImplementation((callback) => {
        callback(0);
        return 1;
      });
    jest.mocked(Lenis).mockImplementationOnce(
      () =>
        ({
          destroy: jest.fn(),
          off: jest.fn(),
          on: jest.fn(),
          raf: jest.fn(),
          scrollTo: jest.fn(() => {
            markerWasPresentDuringScroll = document.documentElement.hasAttribute(
              INITIAL_HASH_POSITIONING_ATTRIBUTE,
            );
          }),
        }) as never,
    );

    render(
      <SmoothScrollProvider>
        <section id="flight-search" style={{ scrollMarginTop: "76px" }}>
          Flight Search
        </section>
      </SmoothScrollProvider>,
    );

    const instance = jest.mocked(Lenis).mock.results[0]?.value;
    expect(instance?.scrollTo).toHaveBeenCalledWith(
      screen.getByText("Flight Search"),
      { immediate: true, offset: -76 },
    );
    expect(markerWasPresentDuringScroll).toBe(true);
    expect(document.documentElement).not.toHaveAttribute(
      INITIAL_HASH_POSITIONING_ATTRIBUTE,
    );
    requestAnimationFrame.mockRestore();
  });

  it("uses native hash scrolling with reduced motion and does not create Lenis", () => {
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
    window.history.replaceState(null, "", "/#flight-search");
    document.documentElement.setAttribute(INITIAL_HASH_POSITIONING_ATTRIBUTE, "");
    let markerWasPresentDuringScroll = false;
    jest.mocked(Element.prototype.scrollIntoView).mockImplementationOnce(() => {
      markerWasPresentDuringScroll = document.documentElement.hasAttribute(
        INITIAL_HASH_POSITIONING_ATTRIBUTE,
      );
    });
    const requestAnimationFrame = jest
      .spyOn(window, "requestAnimationFrame")
      .mockImplementation((callback) => {
        callback(0);
        return 1;
      });

    render(
      <SmoothScrollProvider>
        <section id="flight-search">Flight Search</section>
      </SmoothScrollProvider>,
    );

    expect(Lenis).not.toHaveBeenCalled();
    expect(screen.getByText("Flight Search").scrollIntoView).toHaveBeenCalledWith({
      behavior: "auto",
      block: "start",
    });
    expect(markerWasPresentDuringScroll).toBe(true);
    expect(document.documentElement).not.toHaveAttribute(
      INITIAL_HASH_POSITIONING_ATTRIBUTE,
    );
    requestAnimationFrame.mockRestore();
  });

  it("reveals a marked page when the hash target cannot be resolved", () => {
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
    window.history.replaceState(null, "", "/#flight-search");
    document.documentElement.setAttribute(INITIAL_HASH_POSITIONING_ATTRIBUTE, "");
    const requestAnimationFrame = jest
      .spyOn(window, "requestAnimationFrame")
      .mockImplementation((callback) => {
        callback(0);
        return 1;
      });

    render(
      <SmoothScrollProvider>
        <p>No matching target on this page.</p>
      </SmoothScrollProvider>,
    );

    expect(document.documentElement).not.toHaveAttribute(
      INITIAL_HASH_POSITIONING_ATTRIBUTE,
    );
    expect(Element.prototype.scrollIntoView).not.toHaveBeenCalled();
    requestAnimationFrame.mockRestore();
  });

  it("normalizes a normal homepage visit to the canonical top boundary", () => {
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
    const requestAnimationFrame = jest
      .spyOn(window, "requestAnimationFrame")
      .mockImplementation((callback) => {
        callback(0);
        return 1;
      });

    render(
      <SmoothScrollProvider>
        <section id="flight-search">Flight Search</section>
      </SmoothScrollProvider>,
    );

    const instance = jest.mocked(Lenis).mock.results[0]?.value;
    expect(instance?.scrollTo).toHaveBeenCalledWith(0, {
      force: true,
      immediate: true,
    });
    expect(window.scrollTo).toHaveBeenCalledWith({
      behavior: "auto",
      left: 0,
      top: 0,
    });
    requestAnimationFrame.mockRestore();
  });

  it("resets Home after a client route transition instead of retaining Lenis position", () => {
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
    window.history.replaceState(null, "", "/flights");
    const requestAnimationFrame = jest
      .spyOn(window, "requestAnimationFrame")
      .mockImplementation((callback) => {
        callback(0);
        return 1;
      });

    const view = render(
      <SmoothScrollProvider>
        <p>Route content</p>
      </SmoothScrollProvider>,
    );
    const instance = jest.mocked(Lenis).mock.results[0]?.value;
    expect(instance?.scrollTo).not.toHaveBeenCalled();

    window.history.replaceState(null, "", "/#top");
    view.rerender(
      <SmoothScrollProvider>
        <section id="top">Home Hero</section>
      </SmoothScrollProvider>,
    );

    expect(instance?.scrollTo).toHaveBeenCalledWith(0, {
      force: true,
      immediate: true,
    });
    requestAnimationFrame.mockRestore();
  });

  it("refreshes and updates ScrollTrigger once after the canonical top reset", async () => {
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
    window.history.replaceState(null, "", "/flights");
    const requestAnimationFrame = jest
      .spyOn(window, "requestAnimationFrame")
      .mockImplementation((callback) => {
        callback(0);
        return 1;
      });

    const view = render(
      <SmoothScrollProvider>
        <p>Route content</p>
      </SmoothScrollProvider>,
    );
    const instance = jest.mocked(Lenis).mock.results[0]?.value;
    jest.mocked(ScrollTrigger.refresh).mockClear();
    jest.mocked(ScrollTrigger.update).mockClear();

    window.history.replaceState(null, "", "/#top");
    view.rerender(
      <SmoothScrollProvider>
        <section id="top">Home Hero</section>
      </SmoothScrollProvider>,
    );

    await waitFor(() => expect(ScrollTrigger.refresh).toHaveBeenCalledTimes(1));
    expect(ScrollTrigger.update).toHaveBeenCalledTimes(1);
    expect(instance?.scrollTo).toHaveBeenCalledWith(0, {
      force: true,
      immediate: true,
    });

    const nativeResetOrder = jest.mocked(window.scrollTo).mock.invocationCallOrder.at(-1);
    const lenisResetOrder = jest
      .mocked(instance!.scrollTo)
      .mock.invocationCallOrder.at(-1);
    const refreshOrder = jest.mocked(ScrollTrigger.refresh).mock.invocationCallOrder[0];
    const updateOrder = jest.mocked(ScrollTrigger.update).mock.invocationCallOrder[0];
    expect(nativeResetOrder).toBeLessThan(lenisResetOrder as number);
    expect(lenisResetOrder).toBeLessThan(refreshOrder);
    expect(refreshOrder).toBeLessThan(updateOrder);
    requestAnimationFrame.mockRestore();
  });
});
