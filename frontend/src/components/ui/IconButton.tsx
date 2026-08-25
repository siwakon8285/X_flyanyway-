import type { ReactNode } from "react";

import { Button, type ButtonProps } from "@/components/ui/Button";
import { cn } from "@/lib/utils/cn";

type IconButtonProps = Omit<ButtonProps, "aria-label" | "children"> & {
  children: ReactNode;
  label: string;
};

const IconButton = ({ children, className, label, ...props }: IconButtonProps) => (
  <Button
    aria-label={label}
    className={cn("aspect-square px-0", className)}
    {...props}
  >
    {children}
  </Button>
);

export { IconButton };
export type { IconButtonProps };
