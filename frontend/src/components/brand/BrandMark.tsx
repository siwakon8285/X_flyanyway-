import Image from "next/image";
import type { ComponentProps } from "react";

import brandIcon from "@/app/icon.png";
import { cn } from "@/lib/utils/cn";

type BrandMarkProps = Omit<
  ComponentProps<typeof Image>,
  "alt" | "height" | "src" | "width"
> & {
  dimension?: number;
  label?: string;
};

const BrandMark = ({
  className,
  dimension = 48,
  label,
  ...props
}: BrandMarkProps) => (
  <Image
    alt={label ?? ""}
    aria-hidden={label ? undefined : true}
    className={cn("size-12 object-contain", className)}
    height={dimension}
    src={brandIcon}
    width={dimension}
    {...props}
  />
);

export { BrandMark };
export type { BrandMarkProps };
