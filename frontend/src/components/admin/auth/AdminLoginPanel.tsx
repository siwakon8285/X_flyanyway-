"use client";

import { AdminLoginForm } from "@/components/admin/auth/AdminLoginForm";
import { BrandWordmark } from "@/components/brand/BrandWordmark";
import { useLanguage } from "@/i18n/LanguageProvider";

const AdminLoginPanel = () => {
  const { t } = useLanguage();
  return (
    <section
      aria-labelledby="staff-login-title"
      className="relative w-full max-w-md rounded-[2rem] border border-white/10 bg-[#171717]/95 p-7 shadow-2xl shadow-black/40 sm:p-10"
    >
      <BrandWordmark />
      <p className="mt-8 text-xs font-semibold uppercase tracking-[0.2em] text-brand">{t("admin.login.eyebrow")}</p>
      <h1 className="mt-3 text-3xl font-semibold tracking-[-0.04em]" id="staff-login-title">{t("admin.login.heading")}</h1>
      <p className="mb-8 mt-3 text-sm leading-6 text-white/60">{t("admin.login.intro")}</p>
      <AdminLoginForm />
    </section>
  );
};

export { AdminLoginPanel };
