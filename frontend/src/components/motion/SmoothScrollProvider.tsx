"use client";

import Lenis from "lenis";
import type { ReactNode } from "react";
import { useEffect } from "react";

import { connectLenisToGsap, scrollToLocationHash } from "@/lib/motion/lenis";
import { clearInitialHashPositioning } from "@/lib/motion/initialHash";
import { useReducedMotion } from "@/lib/motion/reducedMotion";

type SmoothScrollProviderProps = {
  children: ReactNode;
};

const SmoothScrollProvider = ({ children }: SmoothScrollProviderProps) => {
  const reducedMotion = useReducedMotion();

  useEffect(() => {
    const positionInitialHash = (lenis: Lenis | null) => {
      try {
        scrollToLocationHash(lenis);
      } finally {
        clearInitialHashPositioning();
      }
    };

    if (reducedMotion) {
      const frame = window.requestAnimationFrame(() => positionInitialHash(null));
      return () => window.cancelAnimationFrame(frame);
    }

    const lenis = new Lenis({
      anchors: true,
      autoRaf: false,
      syncTouch: false,
    });
    const disconnect = connectLenisToGsap(lenis);
    const frame = window.requestAnimationFrame(() => positionInitialHash(lenis));

    return () => {
      window.cancelAnimationFrame(frame);
      disconnect();
    };
  }, [reducedMotion]);

  return children;
};

export { SmoothScrollProvider };
