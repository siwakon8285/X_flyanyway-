import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import type { Metadata } from "next";
import { FlightEditor } from "@/components/admin/flights/FlightEditor";
import { can } from "@/lib/admin/adminAuthorization";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";

export const metadata: Metadata = { title: "Create Flight · X-Fly Anyway" };
export default async function NewFlightPage(){const principal=await fetchStaffPrincipal((await cookies()).toString());if(!principal)redirect("/admin/login");if(!can(principal,"flights:write"))redirect("/admin/flights");return <FlightEditor canWrite mode="new"/>;}
