import type { ComponentProps } from "react";

import { cn } from "@/lib/utils/cn";

type InputProps = ComponentProps<"input">;

const Input = ({ className, type = "text", ...props }: InputProps) => (
  <input
    className={cn(
      "h-12 w-full rounded-control border border-border bg-surface px-4 text-base text-foreground outline-none transition-colors placeholder:text-muted-foreground/70 hover:border-border-strong focus-visible:border-focus focus-visible:ring-2 focus-visible:ring-focus/25 disabled:cursor-not-allowed disabled:bg-muted disabled:opacity-55 aria-invalid:border-destructive aria-invalid:ring-2 aria-invalid:ring-destructive/25",
      className,
    )}
    type={type}
    {...props}
  />
);

export { Input };
export type { InputProps };
