import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils/cn";

const sectionVariants = cva("relative", {
  variants: {
    spacing: {
      sm: "py-section-sm",
      md: "py-section-md",
      lg: "py-section-lg",
    },
  },
  defaultVariants: {
    spacing: "md",
  },
});

type SectionProps = ComponentProps<"section"> &
  VariantProps<typeof sectionVariants>;

const Section = ({ className, spacing, ...props }: SectionProps) => (
  <section className={cn(sectionVariants({ spacing }), className)} {...props} />
);

export { Section };
export type { SectionProps };
