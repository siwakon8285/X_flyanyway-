"use client";

import * as SeparatorPrimitive from "@radix-ui/react-separator";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils/cn";

type SeparatorProps = ComponentProps<typeof SeparatorPrimitive.Root>;

const Separator = ({
  className,
  decorative = true,
  orientation = "horizontal",
  ...props
}: SeparatorProps) => (
  <SeparatorPrimitive.Root
    className={cn(
      "shrink-0 bg-border",
      orientation === "horizontal" ? "h-px w-full" : "h-full w-px",
      className,
    )}
    decorative={decorative}
    orientation={orientation}
    {...props}
  />
);

export { Separator };
export type { SeparatorProps };
