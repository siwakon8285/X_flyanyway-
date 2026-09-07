"use client";

import Link from "next/link";
import { useEffect, useState } from "react";

import { useLanguage } from "@/i18n/LanguageProvider";
import { formatDate, formatPrice } from "@/i18n/formatters";
import type { DashboardData, DashboardFilters as Filters } from "@/lib/admin/dashboardTypes";
import { DailyFigures, DemandChart, RevenueChart } from "./DashboardCharts";
import { DashboardDetail } from "./DashboardDetail";
import { DashboardFilters, presetFilters } from "./DashboardFilters";
import { DashboardMotion } from "./DashboardMotion";
import "./dashboard.css";

export function ExecutiveDashboard() {
  const { t, locale } = useLanguage();
  const [filters, setFilters] = useState<Filters>(() => presetFilters("thirty"));
  const [data, setData] = useState<DashboardData>();
  const [error, setError] = useState<number>();
  const [loading, setLoading] = useState(true);
  const [attempt, setAttempt] = useState(0);
  const [dataRevision, setDataRevision] = useState(0);
  const [routes, setRoutes] = useState<string[]>([]);

  useEffect(() => {
    const controller = new AbortController();
    const params = new URLSearchParams({ from: filters.from, to: filters.to });
    if (filters.route) params.set("route", filters.route);
    if (filters.cabin) params.set("cabin", filters.cabin);
    params.set("provider", filters.provider);
    void (async () => {
      try {
        const response = await fetch(`/admin/api/dashboard?${params}`, { cache: "no-store", credentials: "same-origin", signal: controller.signal });
        if (!response.ok) {
          if (!controller.signal.aborted) {
            if (response.status === 401 || response.status === 403) setData(undefined);
            setError(response.status);
          }
          return;
        }
        const next: DashboardData = await response.json();
        if (!controller.signal.aborted) {
          setData(next);
          setDataRevision((value) => value + 1);
          setRoutes(next.availableRoutes);
        }
      } catch {
        if (!controller.signal.aborted) setError(503);
      } finally {
        if (!controller.signal.aborted) setLoading(false);
      }
    })();
    return () => controller.abort();
  }, [filters, attempt]);

  const number = (value: number) => new Intl.NumberFormat(locale, { maximumFractionDigits: 1 }).format(value);
  const money = (value: number | null) => value === null ? "—" : formatPrice(value, locale);
  const fatalError = error && !data;
  const dataKey = data ? String(dataRevision) : "";
  const applyFilters = (next: Filters) => {
    setLoading(true);
    setError(undefined);
    setFilters({ ...next });
  };
  const retry = () => {
    setLoading(true);
    setError(undefined);
    setAttempt((value) => value + 1);
  };

  return <DashboardMotion dataKey={dataKey}><div className="executive-dashboard" aria-busy={loading}>
    <header className="exec-heading">
      <div className="exec-heading-copy"><p className="exec-eyebrow" data-exec-eyebrow>{t("dashboard.eyebrow")}</p><h1 aria-label={t("dashboard.title")}><span><i data-exec-title-line>{t("dashboard.titleLineOne")}</i></span><span><i data-exec-title-line>{t("dashboard.titleLineTwo")}</i></span></h1><p data-exec-intro>{t("dashboard.intro")}</p></div>
      <div className="exec-hero-instrument" aria-hidden="true">
        <span className="exec-hero-index">XF / OI—20</span>
        <svg viewBox="0 0 520 180"><path className="exec-hero-grid" d="M0 145H520M54 0v180M165 0v180M276 0v180M387 0v180M498 0v180" /><path data-exec-route-path pathLength="1" className="exec-hero-route" d="M20 145C118 145 143 52 246 68s117 72 252-28" /><circle cx="20" cy="145" r="4" /><circle cx="498" cy="40" r="4" /><path className="exec-hero-vector" d="m482 34 16 6-7 15" /></svg>
        <div className="exec-heading-mark" data-exec-mark>XF<span>↗</span></div>
        <p>{t("dashboard.decorativeRoute")}</p>
      </div>
    </header>

    <DashboardFilters initial={filters} loading={loading} routes={routes} onApply={applyFilters} />

    {fatalError ? <div className="exec-state" role="alert"><span>SYS / {error}</span><h2>{t(error === 401 ? "dashboard.unauthenticated" : error === 403 ? "dashboard.forbidden" : error === 422 || error === 400 ? "dashboard.invalid" : "dashboard.error")}</h2>{error === 401 ? <Link href="/admin/login">{t("dashboard.signIn")}</Link> : error === 403 ? <Link href="/admin">{t("dashboard.workspace")}</Link> : <button type="button" onClick={retry}>{t("dashboard.retry")} <i aria-hidden="true">↗</i></button>}</div> : !data ? <div className="exec-loading" role="status"><div className="exec-loader" aria-hidden="true"><span /><span /><span /></div><div><strong>{t("dashboard.loading")}</strong><p>{t("dashboard.loadingDetail")}</p></div></div> : <div className={loading ? "exec-data is-refreshing" : "exec-data"} data-dashboard-data>
      {error && <div className="exec-inline-error" role="alert"><span>{t("dashboard.refreshError")}</span><button type="button" onClick={retry}>{t("dashboard.retry")}</button></div>}
      <div className="exec-context"><p><i aria-hidden="true" />{t(data.provider === "STRIPE" ? "dashboard.stripe" : "dashboard.mockBitcoin")}</p><span>{formatDate(data.from, locale)} — {formatDate(data.to, locale)}</span><span>{t("dashboard.refreshed")} {new Intl.DateTimeFormat(locale, { dateStyle: "short", timeStyle: "short", timeZone: "Asia/Bangkok" }).format(new Date(data.generatedAt))} / UTC+7</span></div>
      <p className="exec-source">{t("dashboard.source")}</p>
      {data.summary.totalBookings === 0 && <div className="exec-empty" role="status"><span>00 / {t("dashboard.emptyCode")}</span><strong>{t("dashboard.empty")}</strong><p>{t("dashboard.emptyHelp")}</p></div>}

      <section className="exec-command" aria-label={t("dashboard.summary")} data-exec-reveal>
        <div className="exec-command-meta"><span>00 / {t("dashboard.commandDeck")}</span><span>{data.currency} / {data.timeZone}</span></div>
        <div className="exec-command-core">
          <div className="exec-revenue-hero" data-exec-follow><p className="exec-eyebrow">{t("dashboard.revenue")}</p><div className="exec-revenue-value">{money(data.summary.grossRevenue)}</div><p>{t("dashboard.grossQualifier")}</p></div>
          <dl className="exec-kpis">
            <div data-exec-follow><span>01</span><dt>{t("dashboard.bookings")}</dt><dd>{number(data.summary.totalBookings)}</dd></div>
            <div data-exec-follow><span>02</span><dt>{t("dashboard.tickets")}</dt><dd>{number(data.summary.ticketsIssued)}</dd></div>
            <div data-exec-follow><span>03</span><dt>{t("dashboard.average")}</dt><dd>{money(data.summary.averageBookingValue)}</dd></div>
            <div data-exec-follow><span>04</span><dt>{t("dashboard.cancellationRate")}</dt><dd>{data.summary.cancellationRatePercent === null ? "—" : `${number(data.summary.cancellationRatePercent)}%`}</dd></div>
          </dl>
        </div>
        <p className="exec-basis">{t("dashboard.basis")}</p>
        <div className="exec-market-floor">
          <section className="exec-revenue-trend" data-exec-follow><div className="exec-chart-heading"><div><span>{t("dashboard.marketSignal")} / 01</span><h2>{t("dashboard.revenueTrend")}</h2></div><strong>THB</strong></div><RevenueChart key={`revenue-${dataRevision}-${loading ? "loading" : "ready"}`} points={data.trends} title={t("dashboard.revenueTrend")} /></section>
          <section className="exec-demand-trend" data-exec-follow><div className="exec-chart-heading"><div><span>{t("dashboard.demandSignal")} / 02</span><h2>{t("dashboard.demandTrend")}</h2></div><strong>{t("dashboard.daily")}</strong></div><DemandChart key={`demand-${dataRevision}-${loading ? "loading" : "ready"}`} points={data.trends} title={t("dashboard.demandTrend")} /></section>
        </div>
        <DailyFigures points={data.trends} />
      </section>
      <DashboardDetail data={data} />
    </div>}
  </div></DashboardMotion>;
}
