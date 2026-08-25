"use client";

import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";

import { Button } from "@/components/ui/Button";
import { motionDurations } from "@/lib/motion/durations";
import { motionEasings } from "@/lib/motion/easing";
import { useReducedMotion } from "@/lib/motion/reducedMotion";

const MotionPresenceDemo = () => {
  const [visible, setVisible] = useState(true);
  const reducedMotion = useReducedMotion();

  return (
    <div className="space-y-5">
      <div className="min-h-28">
        <AnimatePresence initial={false}>
          {visible ? (
            <motion.div
              animate={{ opacity: 1, y: 0 }}
              className="rounded-control border border-border bg-surface p-5"
              exit={reducedMotion ? undefined : { opacity: 0, y: -8 }}
              initial={reducedMotion ? false : { opacity: 0, y: 8 }}
              role="status"
              transition={{
                duration: reducedMotion ? 0 : motionDurations.ui,
                ease: motionEasings.enter,
              }}
            >
              <p className="text-label text-brand">Component-level UI motion</p>
              <p className="mt-2 text-body-sm text-muted-foreground">
                Motion owns this isolated enter and exit behavior.
              </p>
            </motion.div>
          ) : null}
        </AnimatePresence>
      </div>
      <Button onClick={() => setVisible((current) => !current)} variant="outline">
        Toggle UI notice
      </Button>
    </div>
  );
};

export { MotionPresenceDemo };
