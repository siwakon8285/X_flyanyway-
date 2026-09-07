import { AdminWorkspace } from "@/components/admin/shell/AdminWorkspace";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import { fetchStaffPrincipal } from "@/lib/admin/adminBackend";
import { canViewExecutiveDashboard } from "@/lib/admin/adminAuthorization";

const AdminWorkspacePage = async () => {
  const principal = await fetchStaffPrincipal((await cookies()).toString());
  if (!principal) redirect("/admin/login");
  if (canViewExecutiveDashboard(principal)) redirect("/admin/dashboard");
  return <AdminWorkspace />;
};

export default AdminWorkspacePage;
