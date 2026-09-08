import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import type { Metadata } from "next";
import { FlightEditor } from "@/components/admin/flights/FlightEditor";
import { can } from "@/lib/admin/adminAuthorization";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";

export const metadata: Metadata = { title: "Flight Detail · X-Fly Anyway" };
export default async function FlightPage({params}:{params:Promise<{flightId:string}>}){const [principal,{flightId}]=await Promise.all([fetchStaffPrincipal((await cookies()).toString()),params]);if(!principal)redirect("/admin/login");if(!can(principal,"flights:read"))redirect("/admin");return <FlightEditor canWrite={can(principal,"flights:write")} flightId={flightId} mode="detail"/>;}
