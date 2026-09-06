"use client";

import { LogOut } from "lucide-react";
import { useState } from "react";
import { useRouter } from "next/navigation";

import { useLanguage } from "@/i18n/LanguageProvider";
import { logoutStaff } from "@/lib/admin/adminAuthClient";

const AdminLogoutButton = () => {
  const { t } = useLanguage();
  const router = useRouter();
  const [loading, setLoading] = useState(false);

  const logout = async () => {
    setLoading(true);
    try {
      await logoutStaff();
    } finally {
      router.replace("/admin/login");
      router.refresh();
    }
  };

  return (
    <button
      className="mt-5 inline-flex min-h-11 w-full items-center justify-center gap-2 rounded-control border border-white/15 px-4 text-sm font-semibold text-white outline-none transition-colors hover:border-brand/50 hover:text-brand focus-visible:ring-2 focus-visible:ring-brand disabled:opacity-50 motion-reduce:transition-none"
      disabled={loading}
      onClick={logout}
      type="button"
    >
      <LogOut aria-hidden="true" className="size-4" />
      {loading ? t("admin.shell.signingOut") : t("admin.shell.signOut")}
    </button>
  );
};

export { AdminLogoutButton };
