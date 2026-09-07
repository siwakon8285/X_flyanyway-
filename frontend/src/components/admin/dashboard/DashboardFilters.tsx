"use client";

import { useState } from "react";
import { useLanguage } from "@/i18n/LanguageProvider";
import { dashboardCabins, type DashboardFilters as Filters, type DashboardProvider } from "@/lib/admin/dashboardTypes";
import { cabinKeys } from "./DashboardCharts";

export function presetFilters(preset: "today" | "seven" | "thirty" | "month", current?: Filters): Filters {
  const to = new Intl.DateTimeFormat("en-CA", { timeZone: "Asia/Bangkok", year: "numeric", month: "2-digit", day: "2-digit" }).format(new Date());
  const date = new Date(`${to}T00:00:00Z`);
  if (preset === "month") date.setUTCDate(1);
  else date.setUTCDate(date.getUTCDate() - (preset === "seven" ? 6 : preset === "thirty" ? 29 : 0));
  return { route: "", cabin: "", provider: "STRIPE", ...current, from: date.toISOString().slice(0, 10), to };
}

type DashboardFiltersProps = {
  initial: Filters;
  loading: boolean;
  routes: string[];
  onApply: (filters: Filters) => void;
};

export function DashboardFilters({ initial, loading, routes, onApply }: DashboardFiltersProps) {
  const { t } = useLanguage();
  const [draft, setDraft] = useState(initial);
  const [invalid, setInvalid] = useState(false);
  const change = (key: keyof Filters, value: string) => setDraft((old) => ({ ...old, [key]: value }));
  const apply = (filters: Filters) => {
    const days = (Date.parse(filters.to) - Date.parse(filters.from)) / 86400000;
    if (!Number.isFinite(days) || days < 0 || days > 365) { setInvalid(true); return; }
    setInvalid(false); onApply(filters);
  };
  const activePreset = (["today", "seven", "thirty", "month"] as const).find((key) => {
    const preset = presetFilters(key, draft);
    return preset.from === draft.from && preset.to === draft.to;
  });
  return <form className="exec-filters" data-exec-filters aria-busy={loading} onSubmit={(event) => { event.preventDefault(); apply(draft); }}>
    <div className="exec-filter-header"><span>{t("dashboard.cohortControls")}</span>{loading && <span className="exec-filter-pulse" aria-live="polite">{t("dashboard.updating")}</span>}</div>
    <div className="exec-presets" aria-label={t("dashboard.range")}>{(["today", "seven", "thirty", "month"] as const).map((key) => <button key={key} type="button" aria-pressed={activePreset === key} onClick={() => { const next = presetFilters(key, draft); setDraft(next); apply(next); }}>{t(`dashboard.${key}`)}</button>)}</div>
    <div className="exec-filter-fields">
      <label>{t("dashboard.from")}<input required type="date" value={draft.from} onChange={(e) => change("from", e.target.value)} /></label>
      <label>{t("dashboard.to")}<input required type="date" value={draft.to} onChange={(e) => change("to", e.target.value)} /></label>
      <label>{t("dashboard.route")}<select value={draft.route} onChange={(e) => change("route", e.target.value)}><option value="">{t("dashboard.allRoutes")}</option>{routes.map((route) => <option key={route}>{route}</option>)}</select></label>
      <label>{t("dashboard.cabin")}<select value={draft.cabin} onChange={(e) => change("cabin", e.target.value)}><option value="">{t("dashboard.allCabins")}</option>{dashboardCabins.map((cabin) => <option key={cabin} value={cabin}>{t(`dashboard.${cabinKeys[cabin]}`)}</option>)}</select></label>
      <label>{t("dashboard.provider")}<select value={draft.provider} onChange={(e) => change("provider", e.target.value as DashboardProvider)}><option value="STRIPE">{t("dashboard.stripe")}</option><option value="MOCK_BITCOIN">{t("dashboard.mockBitcoin")}</option></select></label>
      <button className="exec-apply" type="submit" disabled={loading}><span>{loading ? t("dashboard.updatingShort") : t("dashboard.apply")}</span><i aria-hidden="true">↗</i></button>
    </div>{invalid && <p role="alert">{t("dashboard.invalid")}</p>}
  </form>;
}
