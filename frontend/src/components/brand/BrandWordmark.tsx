import type { ComponentProps } from "react";

import { BrandMark } from "@/components/brand/BrandMark";
import { cn } from "@/lib/utils/cn";

type BrandWordmarkProps = ComponentProps<"span"> & {
  interactive?: boolean;
  showMark?: boolean;
};

const BrandWordmark = ({
  className,
  interactive = false,
  showMark = true,
  ...props
}: BrandWordmarkProps) => (
  <span
    className={cn(
      "inline-flex items-center gap-2.5 text-sm font-semibold tracking-[0.12em] text-foreground",
      interactive && "relative",
      className,
    )}
    {...props}
  >
    {showMark ? (
      interactive ? (
        <span className="relative inline-flex size-7 shrink-0 overflow-hidden rounded-[0.2rem]">
          <BrandMark className="size-7 motion-safe:transition-transform motion-safe:duration-200 motion-safe:group-hover/brand:scale-[1.03] motion-safe:group-focus-visible/brand:scale-[1.03] motion-reduce:transform-none" />
          <span
            aria-hidden="true"
            className="pointer-events-none absolute inset-y-0 -left-1/2 w-1/3 -translate-x-full skew-x-[-18deg] bg-gradient-to-r from-transparent via-white/55 to-transparent opacity-0 motion-safe:transition-[transform,opacity] motion-safe:duration-300 motion-safe:group-hover/brand:translate-x-[350%] motion-safe:group-hover/brand:opacity-100 motion-safe:group-focus-visible/brand:translate-x-[350%] motion-safe:group-focus-visible/brand:opacity-100 motion-reduce:hidden"
            data-brand-glint
          />
        </span>
      ) : (
        <BrandMark className="size-7" />
      )
    ) : null}
    <span
      className={cn(
        interactive &&
          "transition-colors duration-200 group-hover/brand:text-white motion-safe:transition-transform motion-safe:group-hover/brand:translate-x-px motion-safe:group-focus-visible/brand:translate-x-px motion-reduce:transform-none",
      )}
    >
      X-FLY ANYWAY
    </span>
    {interactive ? (
      <span
        aria-hidden="true"
        className="absolute -bottom-1 left-0 h-px w-full origin-left scale-x-0 bg-brand/80 opacity-0 transition-[transform,opacity] duration-200 group-hover/brand:scale-x-100 group-hover/brand:opacity-100 group-focus-visible/brand:scale-x-100 group-focus-visible/brand:opacity-100 motion-reduce:transition-none"
      />
    ) : null}
  </span>
);

export { BrandWordmark };
export type { BrandWordmarkProps };
