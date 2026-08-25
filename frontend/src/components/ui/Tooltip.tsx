"use client";

import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import type { ComponentProps, ReactNode } from "react";

import { cn } from "@/lib/utils/cn";

type TooltipProps = ComponentProps<typeof TooltipPrimitive.Root> & {
  content: ReactNode;
};

const TooltipProvider = TooltipPrimitive.Provider;

const Tooltip = ({ children, content, ...props }: TooltipProps) => (
  <TooltipPrimitive.Root {...props}>
    <TooltipPrimitive.Trigger asChild>{children}</TooltipPrimitive.Trigger>
    <TooltipPrimitive.Portal>
      <TooltipPrimitive.Content
        className={cn(
          "z-50 max-w-64 rounded-control border border-border bg-surface-elevated px-3 py-2 text-body-sm text-foreground shadow-xl",
        )}
        sideOffset={8}
      >
        {content}
        <TooltipPrimitive.Arrow className="fill-surface-elevated" />
      </TooltipPrimitive.Content>
    </TooltipPrimitive.Portal>
  </TooltipPrimitive.Root>
);

export { Tooltip, TooltipProvider };
export type { TooltipProps };
