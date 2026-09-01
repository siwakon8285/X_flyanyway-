"use client";

import { Menu, X } from "lucide-react";
import Link from "next/link";
import { useRef, useState } from "react";

import { bookingHref, navigationItems } from "@/components/layout/navigationItems";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/Dialog";
import { buttonVariants } from "@/components/ui/Button";
import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";
import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { cn } from "@/lib/utils/cn";
import { useLanguage } from "@/i18n/LanguageProvider";

const mobileNavigationId = "mobile-navigation-menu";

const MobileNavigation = () => {
  const [open, setOpen] = useState(false);
  const menu = useRef<HTMLDivElement>(null);
  const reducedMotion = useReducedMotion();
  const { t } = useLanguage();

  useGSAP(
    () => {
      if (!open || reducedMotion || !menu.current) return;

      const items = menu.current.querySelectorAll("[data-mobile-navigation-item]");

      gsap.fromTo(
        menu.current,
        { autoAlpha: 0 },
        {
          autoAlpha: 1,
          duration: motionDurations.micro,
          ease: gsapEasings.standard,
        },
      );
      gsap.fromTo(
        items,
        { autoAlpha: 0, y: 16 },
        {
          autoAlpha: 1,
          delay: 0.06,
          duration: motionDurations.ui,
          ease: gsapEasings.enter,
          stagger: 0.055,
          y: 0,
        },
      );
    },
    {
      dependencies: [open, reducedMotion],
      revertOnUpdate: true,
      scope: menu,
    },
  );

  return (
    <Dialog onOpenChange={setOpen} open={open}>
      <DialogTrigger asChild>
        <button
          aria-controls={mobileNavigationId}
          aria-expanded={open}
          aria-label={t("navigation.openMenu")}
          className="inline-flex size-11 items-center justify-center rounded-control border border-border/80 bg-background/70 text-foreground outline-none transition-colors hover:border-border-strong hover:bg-surface focus-visible:ring-2 focus-visible:ring-focus lg:hidden"
          type="button"
        >
          <Menu aria-hidden="true" className="size-5" />
        </button>
      </DialogTrigger>
      <DialogContent
        aria-describedby={undefined}
        className="inset-0 left-0 top-0 h-dvh w-full max-w-none -translate-x-0 -translate-y-0 rounded-none border-0 bg-background p-0"
        id={mobileNavigationId}
        showCloseButton={false}
      >
        <DialogTitle className="sr-only">{t("navigation.menuTitle")}</DialogTitle>
        <DialogDescription className="sr-only">
          {t("navigation.dialogDescription")}
        </DialogDescription>
        <div
          className="flex min-h-full flex-col justify-between px-page-gutter pb-8 pt-24"
          ref={menu}
        >
          <nav aria-label={t("navigation.mobile")}>
            <ul className="space-y-2">
              {navigationItems.map((item) => (
                <li data-mobile-navigation-item key={item.id}>
                  <DialogClose asChild>
                    <Link
                      className="inline-flex rounded-sm py-2 text-h1 text-foreground outline-none transition-colors hover:text-brand focus-visible:ring-2 focus-visible:ring-focus"
                      href={item.href}
                    >
                      {t(item.labelKey)}
                    </Link>
                  </DialogClose>
                </li>
              ))}
            </ul>
          </nav>
          <div className="space-y-8" data-mobile-navigation-item>
            <DialogClose asChild>
              <a className={cn(buttonVariants({ size: "lg" }), "w-full sm:w-auto")} href={bookingHref}>
                {t("navigation.bookFlight")}
              </a>
            </DialogClose>
            <p className="text-caption text-muted-foreground">
              {t("navigation.tagline")}
            </p>
          </div>
        </div>
        <DialogClose
          aria-label={t("navigation.closeMenu")}
          className="absolute right-page-gutter top-5 inline-flex size-11 items-center justify-center rounded-control border border-border/80 bg-background/70 text-foreground outline-none transition-colors hover:border-border-strong hover:bg-surface focus-visible:ring-2 focus-visible:ring-focus"
        >
          <X aria-hidden="true" className="size-5" />
        </DialogClose>
      </DialogContent>
    </Dialog>
  );
};

export { MobileNavigation };
