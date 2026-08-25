"use client";

import type { ComponentPropsWithoutRef } from "react";
import { useRef } from "react";
import SplitType from "split-type";

import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";
import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { cn } from "@/lib/utils/cn";

type SplitTextElement = "h1" | "h2" | "h3" | "p" | "span";
type SplitTextMode = "words" | "words,chars";

type SplitTextProps = Omit<ComponentPropsWithoutRef<"p">, "children"> & {
  as?: SplitTextElement;
  split?: SplitTextMode;
  text: string;
};

const SplitText = ({
  as: Component = "p",
  className,
  split = "words",
  text,
  ...props
}: SplitTextProps) => {
  const elementRef = useRef<HTMLElement>(null);
  const reducedMotion = useReducedMotion();

  useGSAP(
    () => {
      const element = elementRef.current;
      if (!element || reducedMotion) return;

      const splitType = new SplitType(element, { types: split });
      const units = split === "words,chars" ? splitType.chars : splitType.words;
      const generatedUnits = [
        ...(splitType.words ?? []),
        ...(splitType.chars ?? []),
      ];

      generatedUnits.forEach((unit) => unit.setAttribute("aria-hidden", "true"));

      if (units?.length) {
        gsap.fromTo(
          units,
          { autoAlpha: 0, yPercent: 65 },
          {
            autoAlpha: 1,
            duration: motionDurations.reveal,
            ease: gsapEasings.enter,
            stagger: 0.035,
            yPercent: 0,
            scrollTrigger: {
              once: true,
              start: "top 88%",
              trigger: element,
            },
          },
        );
      }

      return () => splitType.revert();
    },
    {
      dependencies: [reducedMotion, split, text],
      revertOnUpdate: true,
      scope: elementRef,
    },
  );

  return (
    <Component
      aria-label={text}
      className={cn("motion-split-text", className)}
      ref={(node) => {
        elementRef.current = node;
      }}
      {...props}
    >
      {text}
    </Component>
  );
};

export { SplitText };
export type { SplitTextProps };
