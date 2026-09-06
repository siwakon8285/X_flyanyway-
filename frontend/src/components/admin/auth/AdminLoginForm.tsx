"use client";

import { useState, type FormEvent } from "react";
import { useRouter } from "next/navigation";

import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { useLanguage } from "@/i18n/LanguageProvider";
import { loginStaff, StaffAuthClientError } from "@/lib/admin/adminAuthClient";

const AdminLoginForm = () => {
  const { t } = useLanguage();
  const router = useRouter();
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError("");
    setLoading(true);
    const form = new FormData(event.currentTarget);
    try {
      await loginStaff({
        email: String(form.get("email") ?? ""),
        password: String(form.get("password") ?? ""),
      });
      router.replace("/admin");
      router.refresh();
    } catch (cause) {
      setError(
        cause instanceof StaffAuthClientError && cause.code === "STAFF_LOGIN_THROTTLED"
          ? t("admin.login.tryLater")
          : cause instanceof StaffAuthClientError && cause.status === 401
            ? t("admin.login.invalid")
            : t("admin.login.unavailable"),
      );
      setLoading(false);
    }
  };

  return (
    <form className="space-y-5" onSubmit={submit}>
      <div className="space-y-2">
        <label className="text-sm font-medium text-white" htmlFor="staff-email">
          {t("admin.login.email")}
        </label>
        <Input
          autoComplete="username"
          className="border-white/15 bg-white/10 text-white placeholder:text-white/40"
          disabled={loading}
          id="staff-email"
          name="email"
          placeholder="staff@x-fly.internal"
          required
          type="email"
        />
      </div>
      <div className="space-y-2">
        <label className="text-sm font-medium text-white" htmlFor="staff-password">
          {t("admin.login.password")}
        </label>
        <Input
          autoComplete="current-password"
          className="border-white/15 bg-white/10 text-white"
          disabled={loading}
          id="staff-password"
          minLength={12}
          name="password"
          required
          type="password"
        />
      </div>
      {error ? <p aria-live="polite" className="text-sm text-red-300" role="alert">{error}</p> : null}
      <Button className="w-full" loading={loading} size="lg" type="submit">
        {loading ? t("admin.login.signingIn") : t("admin.login.signIn")}
      </Button>
    </form>
  );
};

export { AdminLoginForm };
