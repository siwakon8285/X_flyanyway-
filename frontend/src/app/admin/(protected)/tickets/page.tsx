import { cookies } from "next/headers"; import { redirect } from "next/navigation"; import type { Metadata } from "next";
import { TicketsWorkspace } from "@/components/admin/tickets/TicketsWorkspace"; import { can } from "@/lib/admin/adminAuthorization"; import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";
export const metadata:Metadata={title:"Ticket / Passenger Operations · X-Fly Anyway"};
export default async function TicketsPage(){const principal=await fetchStaffPrincipal((await cookies()).toString());if(!principal)redirect("/admin/login");if(!can(principal,"tickets:read")||!can(principal,"passengers:read"))redirect("/admin");return <TicketsWorkspace/>;}
