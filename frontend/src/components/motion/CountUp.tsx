"use client";

import type { ComponentPropsWithoutRef } from "react";
import { useRef } from "react";

import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";
import { formatCount } from "@/lib/motion/formatCount";
import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";

type CountUpProps = Omit<ComponentPropsWithoutRef<"output">, "children"> & {
  decimals?: number;
  end: number;
  prefix?: string;
  start?: number;
  suffix?: string;
};

const CountUp = ({
  "aria-label": ariaLabel,
  decimals = 0,
  end,
  prefix = "",
  start = 0,
  suffix = "",
  ...props
}: CountUpProps) => {
  const output = useRef<HTMLOutputElement>(null);
  const visibleValue = useRef<HTMLSpanElement>(null);
  const reducedMotion = useReducedMotion();
  const finalValue = formatCount(end, { decimals, prefix, suffix });

  useGSAP(
    () => {
      if (reducedMotion || !output.current || !visibleValue.current) return;

      const counter = { value: start };
      visibleValue.current.textContent = formatCount(start, {
        decimals,
        prefix,
        suffix,
      });

      gsap.to(counter, {
        duration: motionDurations.reveal,
        ease: gsapEasings.standard,
        onUpdate: () => {
          if (!visibleValue.current) return;
          visibleValue.current.textContent = formatCount(counter.value, {
            decimals,
            prefix,
            suffix,
          });
        },
        scrollTrigger: {
          once: true,
          start: "top 90%",
          trigger: output.current,
        },
        value: end,
      });

      return () => {
        if (visibleValue.current) visibleValue.current.textContent = finalValue;
      };
    },
    {
      dependencies: [decimals, end, prefix, reducedMotion, start, suffix],
      revertOnUpdate: true,
      scope: output,
    },
  );

  return (
    <output aria-label={ariaLabel ?? finalValue} ref={output} {...props}>
      <span aria-hidden="true" ref={visibleValue}>
        {finalValue}
      </span>
    </output>
  );
};

export { CountUp };
export type { CountUpProps };
