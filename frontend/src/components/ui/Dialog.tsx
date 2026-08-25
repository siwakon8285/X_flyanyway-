"use client";

import * as DialogPrimitive from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils/cn";

const Dialog = DialogPrimitive.Root;
const DialogTrigger = DialogPrimitive.Trigger;
const DialogClose = DialogPrimitive.Close;

const DialogOverlay = ({
  className,
  ...props
}: ComponentProps<typeof DialogPrimitive.Overlay>) => (
  <DialogPrimitive.Overlay
    className={cn("fixed inset-0 z-50 bg-black/75 backdrop-blur-sm", className)}
    {...props}
  />
);

const DialogContent = ({
  children,
  className,
  ...props
}: ComponentProps<typeof DialogPrimitive.Content>) => (
  <DialogPrimitive.Portal>
    <DialogOverlay />
    <DialogPrimitive.Content
      className={cn(
        "fixed left-1/2 top-1/2 z-50 grid w-[calc(100%-2rem)] max-w-lg -translate-x-1/2 -translate-y-1/2 gap-6 rounded-surface border border-border bg-surface-elevated p-6 shadow-2xl outline-none focus-visible:ring-2 focus-visible:ring-focus md:p-8",
        className,
      )}
      {...props}
    >
      {children}
      <DialogPrimitive.Close className="absolute right-4 top-4 rounded-control p-2 text-muted-foreground outline-none hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus">
        <X aria-hidden="true" className="size-4" />
        <span className="sr-only">Close dialog</span>
      </DialogPrimitive.Close>
    </DialogPrimitive.Content>
  </DialogPrimitive.Portal>
);

const DialogHeader = ({ className, ...props }: ComponentProps<"header">) => (
  <header className={cn("space-y-2 pr-8", className)} {...props} />
);

const DialogTitle = ({
  className,
  ...props
}: ComponentProps<typeof DialogPrimitive.Title>) => (
  <DialogPrimitive.Title className={cn("text-h3", className)} {...props} />
);

const DialogDescription = ({
  className,
  ...props
}: ComponentProps<typeof DialogPrimitive.Description>) => (
  <DialogPrimitive.Description
    className={cn("text-body-sm text-muted-foreground", className)}
    {...props}
  />
);

const DialogFooter = ({ className, ...props }: ComponentProps<"footer">) => (
  <footer
    className={cn("flex flex-col-reverse gap-3 sm:flex-row sm:justify-end", className)}
    {...props}
  />
);

export {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
};
