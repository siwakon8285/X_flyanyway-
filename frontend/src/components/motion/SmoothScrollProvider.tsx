"use client";

import Lenis from "lenis";
import type { ReactNode } from "react";
import { useEffect } from "react";

import { connectLenisToGsap } from "@/lib/motion/lenis";
import { useReducedMotion } from "@/lib/motion/reducedMotion";

type SmoothScrollProviderProps = {
  children: ReactNode;
};

const SmoothScrollProvider = ({ children }: SmoothScrollProviderProps) => {
  const reducedMotion = useReducedMotion();

  useEffect(() => {
    if (reducedMotion) return;

    const lenis = new Lenis({
      anchors: true,
      autoRaf: false,
      syncTouch: false,
    });

    return connectLenisToGsap(lenis);
  }, [reducedMotion]);

  return children;
};

export { SmoothScrollProvider };
