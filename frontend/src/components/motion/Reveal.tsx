"use client";

import type { ComponentPropsWithoutRef, ReactNode } from "react";
import { useRef } from "react";

import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";
import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";

type RevealElement = "article" | "div" | "section";
type RevealVariant = "fade" | "fade-up";

type RevealProps = Omit<ComponentPropsWithoutRef<"div">, "children"> & {
  as?: RevealElement;
  children: ReactNode;
  delay?: number;
  stagger?: number;
  variant?: RevealVariant;
};

const Reveal = ({
  as: Component = "div",
  children,
  delay = 0,
  stagger = 0,
  variant = "fade-up",
  ...props
}: RevealProps) => {
  const container = useRef<HTMLElement>(null);
  const reducedMotion = useReducedMotion();

  useGSAP(
    () => {
      const element = container.current;
      if (!element || reducedMotion) return;

      const targets = stagger > 0 ? Array.from(element.children) : element;

      gsap.fromTo(
        targets,
        { autoAlpha: 0, y: variant === "fade-up" ? 24 : 0 },
        {
          autoAlpha: 1,
          clearProps: "opacity,transform,visibility",
          delay,
          duration: motionDurations.reveal,
          ease: gsapEasings.enter,
          stagger,
          y: 0,
          scrollTrigger: {
            once: true,
            start: "top 88%",
            trigger: element,
          },
        },
      );
    },
    {
      dependencies: [delay, reducedMotion, stagger, variant],
      revertOnUpdate: true,
      scope: container,
    },
  );

  return (
    <Component
      ref={(node) => {
        container.current = node;
      }}
      {...props}
    >
      {children}
    </Component>
  );
};

export { Reveal };
export type { RevealProps };
