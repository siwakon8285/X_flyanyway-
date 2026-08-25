import { LoaderCircle } from "lucide-react";
import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils/cn";

const buttonVariants = cva(
  "inline-flex shrink-0 items-center justify-center gap-2 rounded-control font-medium tracking-[-0.01em] transition-colors outline-none focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-45 [&_svg]:pointer-events-none [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        primary: "bg-brand text-brand-foreground hover:bg-brand-strong",
        secondary:
          "bg-surface-elevated text-foreground hover:bg-surface-highlight",
        ghost: "text-foreground hover:bg-surface-elevated",
        outline:
          "border border-border bg-transparent text-foreground hover:border-border-strong hover:bg-surface",
        destructive:
          "bg-destructive text-destructive-foreground hover:bg-destructive-strong",
      },
      size: {
        sm: "h-9 px-3 text-sm [&_svg]:size-4",
        md: "h-11 px-5 text-sm [&_svg]:size-4",
        lg: "h-13 px-6 text-base [&_svg]:size-5",
      },
    },
    defaultVariants: {
      size: "md",
      variant: "primary",
    },
  },
);

type ButtonProps = ComponentProps<"button"> &
  VariantProps<typeof buttonVariants> & {
    loading?: boolean;
  };

const Button = ({
  children,
  className,
  disabled,
  loading = false,
  size,
  type = "button",
  variant,
  ...props
}: ButtonProps) => (
  <button
    aria-busy={loading || undefined}
    className={cn(buttonVariants({ size, variant }), className)}
    disabled={disabled || loading}
    type={type}
    {...props}
  >
    {loading ? <LoaderCircle aria-hidden="true" className="animate-spin" /> : null}
    {children}
  </button>
);

export { Button, buttonVariants };
export type { ButtonProps };
