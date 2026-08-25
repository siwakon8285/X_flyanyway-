import type { ComponentProps } from "react";

import { cn } from "@/lib/utils/cn";

type SkeletonProps = ComponentProps<"div">;

const Skeleton = ({ className, ...props }: SkeletonProps) => (
  <div
    aria-hidden="true"
    className={cn("animate-pulse rounded-control bg-muted", className)}
    {...props}
  />
);

export { Skeleton };
export type { SkeletonProps };
