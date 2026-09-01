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

const scrollToPageTop = (lenis: Lenis | null) => {
  window.scrollTo({ behavior: "auto", left: 0, top: 0 });

  lenis?.scrollTo(0, {
    force: true,
    immediate: true,
  });
};

const scrollToLocationHash = (lenis: Lenis | null) => {
  const encodedTargetId = window.location.hash.slice(1);
  if (!encodedTargetId) return false;

  let targetId: string;
  try {
    targetId = decodeURIComponent(encodedTargetId);
  } catch {
    return false;
  }

  const target = document.getElementById(targetId);
  if (!target) return false;

  if (targetId === "top") {
    scrollToPageTop(lenis);
    return true;
  }

  if (lenis) {
    const computedScrollMargin = Number.parseFloat(
      window.getComputedStyle(target).scrollMarginTop,
    );
    const headerOffset = Number.isFinite(computedScrollMargin)
      ? -computedScrollMargin
      : 0;

    lenis.scrollTo(target, {
      immediate: true,
      offset: headerOffset,
    });
  } else {
    target.scrollIntoView({ behavior: "auto", block: "start" });
  }

  return true;
};

export { connectLenisToGsap, scrollToLocationHash, scrollToPageTop };
export type { GsapTickerBridge, ScrollTriggerBridge };
