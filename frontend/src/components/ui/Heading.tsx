import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils/cn";

const headingVariants = cva("text-balance text-foreground", {
  variants: {
    size: {
      display: "text-display",
      h1: "text-h1",
      h2: "text-h2",
      h3: "text-h3",
    },
  },
  defaultVariants: {
    size: "h2",
  },
});

type HeadingElement = "h1" | "h2" | "h3";

type HeadingProps = Omit<ComponentProps<"h2">, "size"> &
  VariantProps<typeof headingVariants> & {
    as?: HeadingElement;
  };

const Heading = ({
  as: Component = "h2",
  className,
  size,
  ...props
}: HeadingProps) => (
  <Component className={cn(headingVariants({ size }), className)} {...props} />
);

export { Heading };
export type { HeadingProps };
