"use client";

import Lenis from "lenis";
import { usePathname } from "next/navigation";
import type { ReactNode } from "react";
import { useEffect, useRef } from "react";

import {
  connectLenisToGsap,
  scrollToLocationHash,
  scrollToPageTop,
} from "@/lib/motion/lenis";
import { clearInitialHashPositioning } from "@/lib/motion/initialHash";
import { ScrollTrigger } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";

type SmoothScrollProviderProps = {
  children: ReactNode;
};

const SmoothScrollProvider = ({ children }: SmoothScrollProviderProps) => {
  const pathname = usePathname();
  const reducedMotion = useReducedMotion();
  const lenisRef = useRef<Lenis | null>(null);

  useEffect(() => {
    const previousScrollRestoration = window.history.scrollRestoration;
    window.history.scrollRestoration = "manual";

    if (reducedMotion) {
      lenisRef.current = null;

      return () => {
        window.history.scrollRestoration = previousScrollRestoration;
      };
    }

    const lenis = new Lenis({
      anchors: true,
      autoRaf: false,
      syncTouch: false,
    });
    const disconnect = connectLenisToGsap(lenis);
    lenisRef.current = lenis;

    return () => {
      lenisRef.current = null;
      disconnect();
      window.history.scrollRestoration = previousScrollRestoration;
    };
  }, [reducedMotion]);

  useEffect(() => {
    if (pathname !== "/") {
      clearInitialHashPositioning();
      return;
    }

    let cancelled = false;
    let layoutFrame = 0;
    let reconciliationFrame = 0;

    const reconcile = () => {
      if (cancelled) return;

      layoutFrame = window.requestAnimationFrame(() => {
        reconciliationFrame = window.requestAnimationFrame(() => {
          if (cancelled) return;

          try {
            if (!scrollToLocationHash(lenisRef.current)) {
              scrollToPageTop(lenisRef.current);
            }
            ScrollTrigger.refresh();
            ScrollTrigger.update();
          } finally {
            clearInitialHashPositioning();
          }
        });
      });
    };

    if (document.fonts) {
      void document.fonts.ready.then(reconcile);
    } else {
      reconcile();
    }

    return () => {
      cancelled = true;
      if (layoutFrame) window.cancelAnimationFrame(layoutFrame);
      if (reconciliationFrame) window.cancelAnimationFrame(reconciliationFrame);
    };
  }, [pathname, reducedMotion]);

  return children;
};

export { SmoothScrollProvider };
