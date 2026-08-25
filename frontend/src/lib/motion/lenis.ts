import type Lenis from "lenis";

import { ScrollTrigger, gsap } from "@/lib/motion/gsap";

type GsapTickerBridge = {
  add: (callback: (time: number) => void) => void;
  remove: (callback: (time: number) => void) => void;
};

type ScrollTriggerBridge = {
  refresh: () => void;
  update: () => void;
};

const connectLenisToGsap = (
  lenis: Lenis,
  ticker: GsapTickerBridge = gsap.ticker,
  scrollTrigger: ScrollTriggerBridge = ScrollTrigger,
) => {
  const handleScroll = () => scrollTrigger.update();
  const handleTick = (time: number) => lenis.raf(time * 1000);

  lenis.on("scroll", handleScroll);
  ticker.add(handleTick);
  scrollTrigger.refresh();

  return () => {
    lenis.off("scroll", handleScroll);
    ticker.remove(handleTick);
    lenis.destroy();
  };
};

export { connectLenisToGsap };
export type { GsapTickerBridge, ScrollTriggerBridge };
