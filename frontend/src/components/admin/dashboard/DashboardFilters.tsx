"use client";

import { useState } from "react";
import { useLanguage } from "@/i18n/LanguageProvider";
import { formatStaffDate } from "@/i18n/formatters";
import { dashboardCabins, type DashboardFilters as Filters, type DashboardProvider } from "@/lib/admin/dashboardTypes";
import { cabinKeys } from "./DashboardCharts";
import { dashboardReportPeriods, resolveDashboardPeriod, type DashboardReportPeriod } from "./dashboardPeriods";

const periodTranslationKeys = {
  daily: "dashboard.reportDaily",
  weekly: "dashboard.reportWeekly",
  monthly: "dashboard.reportMonthly",
  custom: "dashboard.reportCustom",
} as const;

export function presetFilters(preset: DashboardReportPeriod, current?: Filters): Filters {
  const preserved = {
    route: current?.route ?? "",
    cabin: current?.cabin ?? "",
    provider: current?.provider ?? "STRIPE",
  };
  return { ...preserved, ...resolveDashboardPeriod(preset, new Date(), current) };
}

type DashboardFiltersProps = {
  initial: Filters;
  loading: boolean;
  routes: string[];
  onApply: (filters: Filters) => void;
};

export function DashboardFilters({ initial, loading, routes, onApply }: DashboardFiltersProps) {
  const { locale, t } = useLanguage();
  const [draft, setDraft] = useState(initial);
  const [period, setPeriod] = useState<DashboardReportPeriod>(() => {
    const matchingPeriod = dashboardReportPeriods.find((key) => {
      if (key === "custom") return false;
      const range = presetFilters(key, initial);
      return range.from === initial.from && range.to === initial.to;
    });
    return matchingPeriod ?? "custom";
  });
  const [invalid, setInvalid] = useState(false);
  const change = (key: keyof Filters, value: string) => {
    setPeriod("custom");
    setDraft((old) => ({ ...old, [key]: value }));
  };
  const apply = (filters: Filters) => {
    const days = (Date.parse(filters.to) - Date.parse(filters.from)) / 86400000;
    if (!Number.isFinite(days) || days < 0 || days > 365) { setInvalid(true); return; }
    setInvalid(false); onApply(filters);
  };
  const selectPeriod = (nextPeriod: DashboardReportPeriod) => {
    setPeriod(nextPeriod);
    const next = presetFilters(nextPeriod, draft);
    setDraft(next);
    apply(next);
  };
  return <form className="exec-filters" data-exec-filters aria-busy={loading} onSubmit={(event) => { event.preventDefault(); apply(draft); }}>
    <div className="exec-filter-header"><span>{t("dashboard.cohortControls")}</span>{loading && <span className="exec-filter-pulse" aria-live="polite">{t("dashboard.updating")}</span>}</div>
    <div className="exec-presets" aria-label={t("dashboard.range")}>{dashboardReportPeriods.map((key) => <button key={key} type="button" aria-pressed={period === key} onClick={() => selectPeriod(key)}>{t(periodTranslationKeys[key])}</button>)}</div>
    <p className="exec-filter-range"><span>{t("dashboard.resolvedRange")}</span> {formatStaffDate(draft.from, locale)} — {formatStaffDate(draft.to, locale)}</p>
    <div className="exec-filter-fields">
      <label>{t("dashboard.from")}<input className="exec-filter-control" required type="date" value={draft.from} onChange={(e) => change("from", e.target.value)} /></label>
      <label>{t("dashboard.to")}<input className="exec-filter-control" required type="date" value={draft.to} onChange={(e) => change("to", e.target.value)} /></label>
      <label>{t("dashboard.route")}<select className="exec-filter-control" value={draft.route} onChange={(e) => change("route", e.target.value)}><option value="">{t("dashboard.allRoutes")}</option>{routes.map((route) => <option key={route}>{route}</option>)}</select></label>
      <label>{t("dashboard.cabin")}<select className="exec-filter-control" value={draft.cabin} onChange={(e) => change("cabin", e.target.value)}><option value="">{t("dashboard.allCabins")}</option>{dashboardCabins.map((cabin) => <option key={cabin} value={cabin}>{t(`dashboard.${cabinKeys[cabin]}`)}</option>)}</select></label>
      <label>{t("dashboard.provider")}<select className="exec-filter-control" value={draft.provider} onChange={(e) => change("provider", e.target.value as DashboardProvider)}><option value="STRIPE">{t("dashboard.stripe")}</option><option value="MOCK_BITCOIN">{t("dashboard.mockBitcoin")}</option></select></label>
      <button className="exec-apply" type="submit" disabled={loading}><span>{loading ? t("dashboard.updatingShort") : t("dashboard.apply")}</span><i aria-hidden="true">↗</i></button>
    </div>{invalid && <p role="alert">{t("dashboard.invalid")}</p>}
  </form>;
}
