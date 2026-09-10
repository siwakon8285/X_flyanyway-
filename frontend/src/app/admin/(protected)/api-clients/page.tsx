import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import type { Metadata } from "next";

import { ApiClientsWorkspace } from "@/components/admin/api-clients/ApiClientsWorkspace";
import { can } from "@/lib/admin/adminAuthorization";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";

export const metadata: Metadata = { title:"API Client Management · X-Fly Anyway" };

export default async function ApiClientsPage() {
  const principal = await fetchStaffPrincipal((await cookies()).toString());
  if (!principal) redirect("/admin/login");
  if (!can(principal, "api_clients:read")) redirect("/admin");
  return <ApiClientsWorkspace canManage={can(principal, "api_clients:manage")} />;
}
