"use client";

import { useLanguage } from "@/i18n/LanguageProvider";

const AdminWorkspace = () => {
  const { t } = useLanguage();
  return (
    <section className="max-w-4xl" aria-labelledby="workspace-title">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-black/50">{t("admin.workspace.eyebrow")}</p>
      <h1 className="mt-3 text-4xl font-semibold tracking-[-0.05em] text-[#171717] sm:text-5xl" id="workspace-title">{t("admin.workspace.heading")}</h1>
      <p className="mt-5 max-w-2xl text-base leading-7 text-black/60">{t("admin.workspace.intro")}</p>
      <div className="mt-12 rounded-[1.75rem] border border-black/10 bg-white/70 p-7 sm:p-10">
        <p className="text-sm font-semibold text-[#171717]">{t("admin.workspace.active")}</p>
        <p className="mt-2 text-sm leading-6 text-black/55">{t("admin.workspace.boundary")}</p>
      </div>
    </section>
  );
};

export { AdminWorkspace };
