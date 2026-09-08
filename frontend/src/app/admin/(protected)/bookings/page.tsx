import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import type { Metadata } from "next";

import { BookingsWorkspace } from "@/components/admin/bookings/BookingsWorkspace";
import { can } from "@/lib/admin/adminAuthorization";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";

export const metadata: Metadata = { title: "Booking Management · X-Fly Anyway" };

export default async function BookingsPage() {
  const principal = await fetchStaffPrincipal((await cookies()).toString());
  if (!principal) redirect("/admin/login");
  if (!can(principal, "bookings:read")) redirect("/admin");
  return <BookingsWorkspace />;
}
