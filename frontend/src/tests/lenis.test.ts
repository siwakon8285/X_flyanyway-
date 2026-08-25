import type Lenis from "lenis";

import { connectLenisToGsap } from "@/lib/motion/lenis";

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
