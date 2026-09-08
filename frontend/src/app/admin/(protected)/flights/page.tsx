import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import type { Metadata } from "next";
import { FlightsWorkspace } from "@/components/admin/flights/FlightsWorkspace";
import { can } from "@/lib/admin/adminAuthorization";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";

export const metadata: Metadata = { title: "Flight Management · X-Fly Anyway" };
export default async function FlightsPage() {
  const principal=await fetchStaffPrincipal((await cookies()).toString());
  if(!principal)redirect("/admin/login");
  if(!can(principal,"flights:read"))redirect("/admin");
  return <FlightsWorkspace canWrite={can(principal,"flights:write")}/>;
}
