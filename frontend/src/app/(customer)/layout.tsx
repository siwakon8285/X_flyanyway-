import type { ReactNode } from "react";

import { SiteShell } from "@/components/layout/SiteShell";
import { SmoothScrollProvider } from "@/components/motion/SmoothScrollProvider";

const CustomerLayout = ({ children }: { children: ReactNode }) => (
  <SmoothScrollProvider>
    <SiteShell>{children}</SiteShell>
  </SmoothScrollProvider>
);

export default CustomerLayout;
