import type { ReactNode } from "react";

import { Footer } from "@/components/layout/Footer";
import { Header } from "@/components/layout/Header";

type SiteShellProps = {
  children: ReactNode;
};

const SiteShell = ({ children }: SiteShellProps) => (
  <>
    <Header />
    <main className="flex-1">{children}</main>
    <Footer />
  </>
);

export { SiteShell };
export type { SiteShellProps };
