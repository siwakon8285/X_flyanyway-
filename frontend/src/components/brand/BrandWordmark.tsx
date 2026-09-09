import type { ComponentProps } from "react";

import { BrandMark } from "@/components/brand/BrandMark";
import { cn } from "@/lib/utils/cn";

type BrandWordmarkProps = ComponentProps<"span"> & {
  interactive?: boolean;
  markSize?: "compact" | "header" | "sidebar";
  showMark?: boolean;
};

const markSizes = {
  compact: { className: "size-12", dimension: 48 },
  header: { className: "size-14 sm:size-16", dimension: 64 },
  sidebar: { className: "size-14", dimension: 56 },
} as const;

const BrandWordmark = ({
  className,
  interactive = false,
  markSize = "compact",
  showMark = true,
  ...props
}: BrandWordmarkProps) => {
  const mark = markSizes[markSize];

  return (
    <span
      aria-label="X-Fly Anyway"
      className={cn(
        "inline-flex items-center gap-2.5 text-sm font-semibold tracking-[0.12em] text-foreground",
        interactive && "relative",
        className,
      )}
      role="img"
      {...props}
    >
      {showMark ? (
        interactive ? (
          <span className={cn("inline-flex shrink-0", mark.className)}>
            <BrandMark
              className={cn(
                mark.className,
                "motion-safe:transition-transform motion-safe:duration-200 motion-safe:group-hover/brand:scale-[1.03] motion-safe:group-focus-visible/brand:scale-[1.03] motion-reduce:transform-none",
              )}
              dimension={mark.dimension}
            />
          </span>
        ) : (
          <BrandMark className={mark.className} dimension={mark.dimension} />
        )
      ) : null}
      <span
        aria-hidden="true"
        className={cn(
          interactive &&
            "transition-colors duration-200 group-hover/brand:text-white motion-safe:transition-transform motion-safe:group-hover/brand:translate-x-px motion-safe:group-focus-visible/brand:translate-x-px motion-reduce:transform-none",
        )}
      >
        -FLY ANYWAY
      </span>
      {interactive ? (
        <span
          aria-hidden="true"
          className="absolute -bottom-1 left-0 h-px w-full origin-left scale-x-0 bg-brand/80 opacity-0 transition-[transform,opacity] duration-200 group-hover/brand:scale-x-100 group-hover/brand:opacity-100 group-focus-visible/brand:scale-x-100 group-focus-visible/brand:opacity-100 motion-reduce:transition-none"
        />
      ) : null}
    </span>
  );
};

export { BrandWordmark };
export type { BrandWordmarkProps };
