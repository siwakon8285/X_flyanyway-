import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import type { ReactNode } from "react";
import type { Metadata } from "next";

import { AdminShell } from "@/components/admin/shell/AdminShell";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";

export const metadata: Metadata = { title: "Staff workspace · X-Fly Anyway" };

const ProtectedAdminLayout = async ({ children }: { children: ReactNode }) => {
  const principal = await fetchStaffPrincipal((await cookies()).toString());
  if (!principal) redirect("/admin/login");
  return <AdminShell principal={principal}>{children}</AdminShell>;
};

export default ProtectedAdminLayout;
