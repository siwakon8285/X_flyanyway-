import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils/cn";

const containerVariants = cva("mx-auto w-full px-page-gutter", {
  variants: {
    size: {
      content: "max-w-content",
      reading: "max-w-reading",
      full: "max-w-none",
    },
  },
  defaultVariants: {
    size: "content",
  },
});

type ContainerProps = ComponentProps<"div"> &
  VariantProps<typeof containerVariants>;

const Container = ({ className, size, ...props }: ContainerProps) => (
  <div className={cn(containerVariants({ size }), className)} {...props} />
);

export { Container };
export type { ContainerProps };
