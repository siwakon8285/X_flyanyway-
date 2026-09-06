"use client";

import { useEffect } from "react";

import { Button } from "@/components/ui/Button";
import { useLanguage } from "@/i18n/LanguageProvider";

const AdminError = ({ error, reset }: { error: Error & { digest?: string }; reset: () => void }) => {
  const { t } = useLanguage();

  useEffect(() => {
    console.error("Admin access boundary failed", error.digest ?? error.name);
  }, [error]);

  return (
    <main className="grid min-h-dvh place-items-center bg-[#0d0d0d] px-4 text-white">
      <section className="max-w-lg rounded-[2rem] border border-white/10 bg-[#171717] p-8 text-center">
        <p className="text-xs font-semibold uppercase tracking-[0.2em] text-brand">{t("admin.login.eyebrow")}</p>
        <h1 className="mt-4 text-3xl font-semibold tracking-[-0.04em]">{t("admin.workspace.unavailable")}</h1>
        <p className="mt-4 text-sm leading-6 text-white/60">{t("admin.workspace.unavailableDetail")}</p>
        <Button className="mt-7" onClick={reset}>{t("admin.workspace.retry")}</Button>
      </section>
    </main>
  );
};

export default AdminError;
