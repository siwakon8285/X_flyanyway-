import type { ComponentProps } from "react";

import { BrandMark } from "@/components/brand/BrandMark";
import { cn } from "@/lib/utils/cn";

type BrandWordmarkProps = ComponentProps<"span"> & {
  showMark?: boolean;
};

const BrandWordmark = ({
  className,
  showMark = true,
  ...props
}: BrandWordmarkProps) => (
  <span
    className={cn(
      "inline-flex items-center gap-2.5 text-sm font-semibold tracking-[0.12em] text-foreground",
      className,
    )}
    {...props}
  >
    {showMark ? <BrandMark className="size-7" /> : null}
    <span>X-FLY ANYWAY</span>
  </span>
);

export { BrandWordmark };
export type { BrandWordmarkProps };
