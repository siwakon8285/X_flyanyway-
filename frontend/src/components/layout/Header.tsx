import { ArrowUpRight } from "lucide-react";

import { BrandWordmark } from "@/components/brand/BrandWordmark";
import { Container } from "@/components/layout/Container";
import { buttonVariants } from "@/components/ui/Button";
import { cn } from "@/lib/utils/cn";

const navigationItems = [
  { href: "#typography", label: "Typography" },
  { href: "#components", label: "Components" },
  { href: "#surfaces", label: "Surfaces" },
] as const;

const Header = () => (
  <header className="border-b border-border/80 bg-background/95">
    <Container className="flex min-h-18 items-center justify-between gap-6 py-3">
      <a
        aria-label="X-Fly Anyway design foundation home"
        className="rounded-control outline-none focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-4 focus-visible:ring-offset-background"
        href="#top"
      >
        <BrandWordmark />
      </a>

      <nav aria-label="Foundation preview" className="hidden md:block">
        <ul className="flex items-center gap-7">
          {navigationItems.map((item) => (
            <li key={item.href}>
              <a
                className="text-body-sm text-muted-foreground outline-none hover:text-foreground focus-visible:rounded-sm focus-visible:ring-2 focus-visible:ring-focus"
                href={item.href}
              >
                {item.label}
              </a>
            </li>
          ))}
        </ul>
      </nav>

      <a
        className={cn(buttonVariants({ size: "sm" }), "hidden sm:inline-flex")}
        href="#components"
      >
        Review UI
        <ArrowUpRight aria-hidden="true" />
      </a>
    </Container>
  </header>
);

export { Header };
