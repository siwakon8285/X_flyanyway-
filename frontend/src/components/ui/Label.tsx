"use client";

import * as LabelPrimitive from "@radix-ui/react-label";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils/cn";

type LabelProps = ComponentProps<typeof LabelPrimitive.Root>;

const Label = ({ className, ...props }: LabelProps) => (
  <LabelPrimitive.Root
    className={cn(
      "text-label text-foreground peer-disabled:cursor-not-allowed peer-disabled:opacity-50",
      className,
    )}
    {...props}
  />
);

export { Label };
export type { LabelProps };
