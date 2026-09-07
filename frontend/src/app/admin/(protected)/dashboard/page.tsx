import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import { ExecutiveDashboard } from "@/components/admin/dashboard/ExecutiveDashboard";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";
import { canViewExecutiveDashboard } from "@/lib/admin/adminAuthorization";
import type { Metadata } from "next";

export const metadata: Metadata = { title: "Executive dashboard · X-Fly Anyway" };

export default async function DashboardPage() {
  const principal = await fetchStaffPrincipal((await cookies()).toString());
  if (!principal) redirect("/admin/login");
  if (!canViewExecutiveDashboard(principal)) redirect("/admin");
  return <ExecutiveDashboard />;
}
