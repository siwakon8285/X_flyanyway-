"use client";

import type { ComponentPropsWithoutRef, ReactNode } from "react";
import { useRef } from "react";

import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { motionMediaQueries } from "@/lib/motion/scroll";
import { cn } from "@/lib/utils/cn";

type ParallaxMediaProps = Omit<ComponentPropsWithoutRef<"div">, "children"> & {
  children: ReactNode;
  strength?: number;
};

const ParallaxMedia = ({
  children,
  className,
  strength = 8,
  ...props
}: ParallaxMediaProps) => {
  const container = useRef<HTMLDivElement>(null);
  const media = useRef<HTMLDivElement>(null);
  const reducedMotion = useReducedMotion();

  useGSAP(
    () => {
      if (reducedMotion || !container.current || !media.current) return;

      const mediaQueries = gsap.matchMedia();
      mediaQueries.add(motionMediaQueries.parallax, () => {
        gsap.fromTo(
          media.current,
          { yPercent: -strength },
          {
            ease: "none",
            scrollTrigger: {
              end: "bottom top",
              scrub: true,
              start: "top bottom",
              trigger: container.current,
            },
            yPercent: strength,
          },
        );
      });

      return () => mediaQueries.revert();
    },
    {
      dependencies: [reducedMotion, strength],
      revertOnUpdate: true,
      scope: container,
    },
  );

  return (
    <div className={cn("overflow-hidden", className)} ref={container} {...props}>
      <div className="motion-parallax-media" ref={media}>
        {children}
      </div>
    </div>
  );
};

export { ParallaxMedia };
export type { ParallaxMediaProps };
