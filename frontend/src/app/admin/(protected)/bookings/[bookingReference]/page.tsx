import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import type { Metadata } from "next";

import { BookingDetailWorkspace } from "@/components/admin/bookings/BookingDetailWorkspace";
import { can } from "@/lib/admin/adminAuthorization";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";

export const metadata: Metadata = { title: "Booking Detail · X-Fly Anyway" };

export default async function BookingDetailPage({ params }: { params: Promise<{ bookingReference: string }> }) {
  const [principal, { bookingReference }] = await Promise.all([
    fetchStaffPrincipal((await cookies()).toString()), params,
  ]);
  if (!principal) redirect("/admin/login");
  if (!can(principal, "bookings:read")) redirect("/admin");
  return <BookingDetailWorkspace bookingReference={bookingReference.toUpperCase()} canManage={can(principal, "bookings:manage")} />;
}
