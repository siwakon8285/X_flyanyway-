import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils/cn";

const cardVariants = cva("rounded-surface p-6 md:p-8", {
  variants: {
    variant: {
      surface: "bg-surface",
      elevated: "border border-border/70 bg-surface-elevated shadow-2xl shadow-black/20",
      bordered: "border border-border bg-transparent",
      glass:
        "border border-white/10 bg-surface-elevated/75 backdrop-blur-md supports-[backdrop-filter]:bg-surface-elevated/60",
    },
  },
  defaultVariants: {
    variant: "surface",
  },
});

type CardProps = ComponentProps<"article"> & VariantProps<typeof cardVariants>;

const Card = ({ className, variant, ...props }: CardProps) => (
  <article className={cn(cardVariants({ variant }), className)} {...props} />
);

const CardHeader = ({ className, ...props }: ComponentProps<"header">) => (
  <header className={cn("mb-6 space-y-2", className)} {...props} />
);

const CardTitle = ({ className, ...props }: ComponentProps<"h3">) => (
  <h3 className={cn("text-h3", className)} {...props} />
);

const CardDescription = ({ className, ...props }: ComponentProps<"p">) => (
  <p className={cn("text-body-sm text-muted-foreground", className)} {...props} />
);

const CardContent = ({ className, ...props }: ComponentProps<"div">) => (
  <div className={cn("text-body", className)} {...props} />
);

export { Card, CardContent, CardDescription, CardHeader, CardTitle };
export type { CardProps };
