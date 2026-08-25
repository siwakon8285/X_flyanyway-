import type { ComponentProps } from "react";

import { cn } from "@/lib/utils/cn";

type BrandMarkProps = Omit<ComponentProps<"svg">, "children"> & {
  label?: string;
};

const BrandMark = ({ className, label, ...props }: BrandMarkProps) => (
  <svg
    aria-hidden={label ? undefined : true}
    aria-label={label}
    className={cn("size-9 text-brand", className)}
    fill="none"
    role={label ? "img" : undefined}
    viewBox="0 0 48 48"
    xmlns="http://www.w3.org/2000/svg"
    {...props}
  >
    <path
      d="M4 7h11.2L24 18.4 32.8 7H44L30.3 24 44 41H32.8L24 29.6 15.2 41H4l13.7-17L4 7Z"
      fill="currentColor"
    />
  </svg>
);

export { BrandMark };
export type { BrandMarkProps };
