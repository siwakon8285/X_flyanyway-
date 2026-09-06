import type { Metadata } from "next";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { AdminLoginPanel } from "@/components/admin/auth/AdminLoginPanel";
import { LanguageToggle } from "@/components/layout/LanguageToggle";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";

export const metadata: Metadata = { title: "Staff sign in · X-Fly Anyway" };

const AdminLoginPage = async () => {
  const cookieStore = await cookies();
  const cookieHeader = cookieStore.toString();
  if (await fetchStaffPrincipal(cookieHeader)) redirect("/admin");

  return (
    <main className="relative grid min-h-dvh place-items-center overflow-hidden bg-[#0d0d0d] px-4 py-12 text-white">
      <div aria-hidden="true" className="absolute -right-32 -top-32 size-[34rem] rounded-full bg-brand/10 blur-3xl" />
      <div className="absolute right-5 top-5"><LanguageToggle /></div>
      <AdminLoginPanel />
    </main>
  );
};

export default AdminLoginPage;
