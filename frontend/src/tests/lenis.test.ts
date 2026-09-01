import type Lenis from "lenis";

import { connectLenisToGsap, scrollToLocationHash } from "@/lib/motion/lenis";

describe("connectLenisToGsap", () => {
  it("synchronizes scroll and ticker work and releases every owned resource", () => {
    let scrollHandler: (() => void) | undefined;
    let tickerHandler: ((time: number) => void) | undefined;
    const raf = jest.fn();
    const destroy = jest.fn();
    const off = jest.fn();
    const lenis = {
      destroy,
      off,
      on: jest.fn((_event: string, handler: () => void) => {
        scrollHandler = handler;
      }),
      raf,
    } as unknown as Lenis;
    const ticker = {
      add: jest.fn((handler: (time: number) => void) => {
        tickerHandler = handler;
      }),
      remove: jest.fn(),
    };
    const scrollTrigger = {
      refresh: jest.fn(),
      update: jest.fn(),
    };

    const disconnect = connectLenisToGsap(lenis, ticker, scrollTrigger);
    scrollHandler?.();
    tickerHandler?.(1.25);

    expect(scrollTrigger.update).toHaveBeenCalledTimes(1);
    expect(raf).toHaveBeenCalledWith(1250);
    expect(scrollTrigger.refresh).toHaveBeenCalledTimes(1);

    disconnect();

    expect(off).toHaveBeenCalledWith("scroll", scrollHandler);
    expect(ticker.remove).toHaveBeenCalledWith(tickerHandler);
    expect(destroy).toHaveBeenCalledTimes(1);
  });
});

describe("scrollToLocationHash", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    window.history.replaceState(null, "", "/");
  });

  it("uses the existing Lenis instance with the target scroll margin as an offset", () => {
    window.history.replaceState(null, "", "/?from=DXB#flight-search");
    const target = document.createElement("section");
    target.id = "flight-search";
    target.style.scrollMarginTop = "76px";
    document.body.append(target);
    const scrollTo = jest.fn();

    expect(scrollToLocationHash({ scrollTo } as unknown as Lenis)).toBe(true);
    expect(scrollTo).toHaveBeenCalledWith(target, {
      immediate: true,
      offset: -76,
    });

    target.remove();
  });

  it("uses native target scrolling when Lenis is disabled", () => {
    window.history.replaceState(null, "", "/#flight-search");
    const target = document.createElement("section");
    target.id = "flight-search";
    document.body.append(target);

    expect(scrollToLocationHash(null)).toBe(true);
    expect(target.scrollIntoView).toHaveBeenCalledWith({
      behavior: "auto",
      block: "start",
    });

    target.remove();
  });

  it("resets both Lenis and native scrolling for the canonical Home Hero", () => {
    window.history.replaceState(null, "", "/#top");
    const target = document.createElement("section");
    target.id = "top";
    document.body.append(target);
    const scrollTo = jest.fn();

    expect(scrollToLocationHash({ scrollTo } as unknown as Lenis)).toBe(true);
    expect(scrollTo).toHaveBeenCalledWith(0, {
      force: true,
      immediate: true,
    });
    expect(window.scrollTo).toHaveBeenCalledWith({
      behavior: "auto",
      left: 0,
      top: 0,
    });

    target.remove();
  });

  it("does nothing for a normal homepage visit or an unknown target", () => {
    expect(scrollToLocationHash(null)).toBe(false);

    window.history.replaceState(null, "", "/#missing-section");
    expect(scrollToLocationHash(null)).toBe(false);
    expect(Element.prototype.scrollIntoView).not.toHaveBeenCalled();
  });
});
