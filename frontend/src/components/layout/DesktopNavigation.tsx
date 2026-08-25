import { navigationItems } from "@/components/layout/navigationItems";

const DesktopNavigation = () => (
  <nav aria-label="Primary navigation" className="hidden lg:block">
    <ul className="flex items-center gap-7">
      {navigationItems.map((item) => (
        <li key={item.id}>
          <a
            className="group relative py-2 text-body-sm text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:rounded-sm focus-visible:text-foreground focus-visible:ring-2 focus-visible:ring-focus after:absolute after:inset-x-0 after:bottom-0 after:h-px after:origin-left after:scale-x-0 after:bg-brand after:transition-transform after:duration-200 group-hover:after:scale-x-100 focus-visible:after:scale-x-100"
            href={item.href}
          >
            {item.label}
          </a>
        </li>
      ))}
    </ul>
  </nav>
);

export { DesktopNavigation };
