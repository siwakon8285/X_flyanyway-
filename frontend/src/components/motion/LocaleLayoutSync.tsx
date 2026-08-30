"use client";

import { useEffect, useRef } from "react";

import { useLanguage } from "@/i18n/LanguageProvider";
import { ScrollTrigger } from "@/lib/motion/gsap";

const LocaleLayoutSync = () => {
  const { locale } = useLanguage();
  const initialRender = useRef(true);

  useEffect(() => {
    if (initialRender.current) {
      initialRender.current = false;
      return;
    }

    let cancelled = false;
    let frame = 0;

    const refresh = () => {
      if (cancelled) return;
      frame = window.requestAnimationFrame(() => {
        if (!cancelled) ScrollTrigger.refresh();
      });
    };

    if (document.fonts) {
      void document.fonts.ready.then(refresh);
    } else {
      refresh();
    }

    return () => {
      cancelled = true;
      if (frame) window.cancelAnimationFrame(frame);
    };
  }, [locale]);

  return null;
};

export { LocaleLayoutSync };
