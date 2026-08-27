import { render, screen } from "@testing-library/react";
import Lenis from "lenis";

import { SmoothScrollProvider } from "@/components/motion/SmoothScrollProvider";
import { INITIAL_HASH_POSITIONING_ATTRIBUTE } from "@/lib/motion/initialHash";

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

  it("does not scroll a normal homepage visit", () => {
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
    expect(instance?.scrollTo).not.toHaveBeenCalled();
    expect(Element.prototype.scrollIntoView).not.toHaveBeenCalled();
    requestAnimationFrame.mockRestore();
  });
});
