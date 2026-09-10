import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import type { Metadata } from "next";

import { ApiClientEditor } from "@/components/admin/api-clients/ApiClientEditor";
import { can } from "@/lib/admin/adminAuthorization";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";

export const metadata: Metadata = { title:"Register API Client · X-Fly Anyway" };

export default async function NewApiClientPage() {
  const principal = await fetchStaffPrincipal((await cookies()).toString());
  if (!principal) redirect("/admin/login");
  if (!can(principal, "api_clients:manage")) redirect("/admin");
  return <ApiClientEditor canManage mode="new" />;
}
