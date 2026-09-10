import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import type { Metadata } from "next";

import { ApiClientEditor } from "@/components/admin/api-clients/ApiClientEditor";
import { can } from "@/lib/admin/adminAuthorization";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";

export const metadata: Metadata = { title:"API Client Detail · X-Fly Anyway" };

export default async function ApiClientDetailPage({ params }: { params:Promise<{ clientId:string }> }) {
  const [principal, { clientId }] = await Promise.all([
    fetchStaffPrincipal((await cookies()).toString()),
    params,
  ]);
  if (!principal) redirect("/admin/login");
  if (!can(principal, "api_clients:read")) redirect("/admin");
  return <ApiClientEditor canManage={can(principal, "api_clients:manage")} clientId={clientId} mode="detail" />;
}
