"use client";

import * as TabsPrimitive from "@radix-ui/react-tabs";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils/cn";

const Tabs = ({ className, ...props }: ComponentProps<typeof TabsPrimitive.Root>) => (
  <TabsPrimitive.Root className={cn("w-full", className)} {...props} />
);

const TabsList = ({
  className,
  ...props
}: ComponentProps<typeof TabsPrimitive.List>) => (
  <TabsPrimitive.List
    className={cn(
      "inline-flex min-h-11 w-full items-center gap-1 rounded-control bg-muted p-1 sm:w-auto",
      className,
    )}
    {...props}
  />
);

const TabsTrigger = ({
  className,
  ...props
}: ComponentProps<typeof TabsPrimitive.Trigger>) => (
  <TabsPrimitive.Trigger
    className={cn(
      "flex min-h-9 flex-1 items-center justify-center rounded-[calc(var(--radius-control)-0.125rem)] px-4 text-sm font-medium text-muted-foreground outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-surface-elevated data-[state=active]:text-foreground sm:flex-none",
      className,
    )}
    {...props}
  />
);

const TabsContent = ({
  className,
  ...props
}: ComponentProps<typeof TabsPrimitive.Content>) => (
  <TabsPrimitive.Content
    className={cn("mt-6 outline-none focus-visible:ring-2 focus-visible:ring-focus", className)}
    {...props}
  />
);

export { Tabs, TabsContent, TabsList, TabsTrigger };
