"use client";

import { useRef, useState } from "react";
import { flushSync } from "react-dom";

import { Button } from "@/components/ui/Button";
import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";
import { Flip, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { cn } from "@/lib/utils/cn";

const FlipDemo = () => {
  const root = useRef<HTMLDivElement>(null);
  const card = useRef<HTMLElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  const [atDestination, setAtDestination] = useState(false);
  const reducedMotion = useReducedMotion();

  useGSAP(
    (_context, contextSafe) => {
      const buttonElement = button.current;
      if (!buttonElement || !contextSafe) return;

      const moveCard = contextSafe(() => {
        if (!card.current || reducedMotion) {
          setAtDestination((current) => !current);
          return;
        }

        const state = Flip.getState(card.current);
        flushSync(() => setAtDestination((current) => !current));
        Flip.from(state, {
          absolute: true,
          duration: motionDurations.ui,
          ease: gsapEasings.enter,
        });
      });

      buttonElement.addEventListener("click", moveCard);
      return () => buttonElement.removeEventListener("click", moveCard);
    },
    {
      dependencies: [reducedMotion],
      revertOnUpdate: true,
      scope: root,
    },
  );

  return (
    <div className="space-y-5" ref={root}>
      <div className="grid min-h-52 grid-cols-2 gap-3">
        <div className="col-start-1 row-start-1 rounded-surface border border-dashed border-border p-3 text-caption text-muted-foreground">
          Origin
        </div>
        <div className="col-start-2 row-start-1 rounded-surface border border-dashed border-border p-3 text-caption text-muted-foreground">
          Destination
        </div>
        <article
          className={cn(
            "z-10 m-5 row-start-1 self-center rounded-control border border-brand/40 bg-brand p-5 text-brand-foreground shadow-xl",
            atDestination ? "col-start-2" : "col-start-1",
          )}
          ref={card}
        >
          <p className="text-label">Layout card</p>
          <p className="mt-2 text-sm">One element, two layout states.</p>
        </article>
      </div>
      <Button aria-pressed={atDestination} ref={button} variant="outline">
        Move demo card
      </Button>
    </div>
  );
};

export { FlipDemo };
