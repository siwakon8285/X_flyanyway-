"use client";

import { ArrowUpRight } from "lucide-react";
import { useRef } from "react";

import { BrandWordmark } from "@/components/brand/BrandWordmark";
import { Container } from "@/components/layout/Container";
import { DesktopNavigation } from "@/components/layout/DesktopNavigation";
import { LanguageToggle } from "@/components/layout/LanguageToggle";
import { MobileNavigation } from "@/components/layout/MobileNavigation";
import { bookingHref } from "@/components/layout/navigationItems";
import { buttonVariants } from "@/components/ui/Button";
import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";
import { ScrollTrigger, gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { cn } from "@/lib/utils/cn";
import { useLanguage } from "@/i18n/LanguageProvider";

const Header = () => {
  const header = useRef<HTMLElement>(null);
  const reducedMotion = useReducedMotion();
  const { t } = useLanguage();

  useGSAP(
    () => {
      const element = header.current;
      if (!element || reducedMotion) return;

      gsap.fromTo(
        "[data-header-content]",
        { autoAlpha: 0, y: -10 },
        {
          autoAlpha: 1,
          clearProps: "opacity,transform,visibility",
          duration: motionDurations.ui,
          ease: gsapEasings.enter,
          y: 0,
        },
      );

      ScrollTrigger.create({
        onToggle: (trigger) => element.toggleAttribute("data-scrolled", trigger.isActive),
        start: 12,
      });
    },
    {
      dependencies: [reducedMotion],
      revertOnUpdate: true,
      scope: header,
    },
  );

  return (
    <header className="site-header fixed inset-x-0 top-0 z-40" ref={header}>
      <Container
        className="flex h-header items-center justify-between gap-4"
        data-header-content
      >
        <a
          aria-label={t("navigation.home")}
          className="rounded-control outline-none focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-4 focus-visible:ring-offset-background"
          href="#top"
        >
          <BrandWordmark className="text-xs sm:text-sm" />
        </a>

        <DesktopNavigation />

        <div className="flex items-center gap-3">
          <a
            className={cn(buttonVariants({ size: "sm" }), "hidden lg:inline-flex")}
            href={bookingHref}
          >
            {t("navigation.bookFlight")}
            <ArrowUpRight aria-hidden="true" />
          </a>
          <LanguageToggle />
          <MobileNavigation />
        </div>
      </Container>
    </header>
  );
};

export { Header };
