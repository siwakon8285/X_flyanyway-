"use client";

import { ArrowRight, Filter, Plus } from "lucide-react";
import Link from "next/link";
import { useEffect, useState } from "react";

import { DatePickerField } from "@/components/admin/flights/DatePickerField";
import { FlightStatusBadge } from "@/components/admin/flights/FlightOperationsPrimitives";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";
import type { FlightPage } from "@/lib/admin/flightTypes";
import "./flightOperations.css";

const columns: TranslationKey[] = [
  "flightManagement.flight", "flightManagement.route", "flightManagement.departure",
  "flightManagement.cabins", "flightManagement.status", "flightManagement.actions",
];

const FlightsWorkspace = ({ canWrite }: { canWrite: boolean }) => {
  const { t } = useLanguage();
  const [data, setData] = useState<FlightPage | null>(null);
  const [error, setError] = useState(false);
  const [filters, setFilters] = useState({ search:"", origin:"", destination:"", date:"", status:"" });
  const [query, setQuery] = useState("");
  const [retryToken, setRetryToken] = useState(0);

  useEffect(() => {
    const controller = new AbortController();
    fetch(`/admin/api/flights${query}`, { cache:"no-store", credentials:"same-origin", signal:controller.signal })
      .then(async (response) => { if (!response.ok) throw new Error("flight list"); return response.json() as Promise<FlightPage>; })
      .then(setData)
      .catch((cause) => { if (cause?.name !== "AbortError") setError(true); });
    return () => controller.abort();
  }, [query, retryToken]);

  const apply = () => {
    setError(false);
    const params = new URLSearchParams();
    Object.entries(filters).forEach(([key, value]) => { if (value) params.set(key, value); });
    setQuery(params.size ? `?${params}` : "");
  };

  return <section aria-labelledby="flight-management-title" className="flight-operations">
    <header className="xfo-page-header">
      <div className="xfo-page-header-copy">
        <p className="xfo-kicker">{t("flightManagement.eyebrow")}</p>
        <h1 id="flight-management-title">{t("flightManagement.title")}</h1>
        <p>{t("flightManagement.intro")}</p>
      </div>
      <div className="xfo-page-index">
        <span>XF / OPS—21</span>
        <strong>{t("flightManagement.flightOperations")}</strong>
        {canWrite ? <Link className="xfo-primary-action" href="/admin/flights/new"><Plus aria-hidden="true" className="size-4" />{t("flightManagement.create")}</Link> : <span className="xfo-readonly">{t("flightManagement.readOnly")}</span>}
      </div>
    </header>

    <form aria-label={t("flightManagement.filters")} className="xfo-filter-panel" onSubmit={(event) => { event.preventDefault(); apply(); }}>
      <div className="xfo-filter-heading"><span>{t("flightManagement.filterTitle")}</span><Filter aria-hidden="true" className="size-4" /></div>
      <div className="xfo-filter-grid">
        {(["search", "origin", "destination"] as const).map((field) => {
          const key = field === "search" ? "flightManagement.flightNumber" : `flightManagement.${field}` as TranslationKey;
          return <label key={field}>{t(key)}<input aria-label={t(key)} onChange={(event) => setFilters((current) => ({ ...current, [field]:event.target.value }))} type="text" value={filters[field]} /></label>;
        })}
        <DatePickerField label={t("flightManagement.departureDate")} onChange={(date) => setFilters((current) => ({ ...current, date }))} value={filters.date} />
        <label>{t("flightManagement.status")}<select aria-label={t("flightManagement.status")} onChange={(event) => setFilters((current) => ({ ...current, status:event.target.value }))} value={filters.status}><option value="">{t("flightManagement.all")}</option><option value="SCHEDULED">{t("flightManagement.scheduled")}</option><option value="CANCELLED">{t("flightManagement.cancelled")}</option></select></label>
        <div className="xfo-filter-actions"><button type="submit">{t("flightManagement.apply")}</button><button onClick={() => { setFilters({ search:"", origin:"", destination:"", date:"", status:"" }); setQuery(""); }} type="button">{t("flightManagement.clear")}</button></div>
      </div>
    </form>

    {!data && !error ? <p aria-live="polite" className="mt-8 text-sm text-black/55">{t("flightManagement.loading")}</p> : null}
    {error ? <div className="mt-8 border-l-2 border-red-700 pl-4" role="alert"><p>{t("flightManagement.error")}</p><button className="mt-3 min-h-11 font-semibold underline decoration-[#ffd400] decoration-2 underline-offset-4" onClick={() => { setError(false); setRetryToken((value) => value + 1); }} type="button">{t("flightManagement.retry")}</button></div> : null}
    {data && data.items.length === 0 ? <p className="mt-8 border-block border-black/15 py-8 text-black/60">{t("flightManagement.empty")}</p> : null}

    {data?.items.length ? <div aria-label={t("flightManagement.serviceRegistry")} className="xfo-board" role="region" tabIndex={0}>
      <table><caption className="sr-only">{t("flightManagement.serviceRegistry")}</caption><thead><tr>{columns.map((key) => <th key={key} scope="col">{t(key)}</th>)}</tr></thead>
        <tbody>{data.items.map((flight) => <tr key={flight.id}>
          <th className="xfo-flight-number" scope="row">{flight.flightNumber}</th>
          <td className="xfo-route">{flight.originCode}<b aria-hidden="true">→</b>{flight.destinationCode}</td>
          <td><time className="block font-medium" dateTime={flight.operatingDate ?? undefined}>{flight.operatingDate ?? t("flightManagement.recurring")}</time><span className="mt-1 block text-xs text-black/55">{flight.departureTime?.slice(0,5) ?? t("flightManagement.unavailable")}</span></td>
          <td className="xfo-cabins"><span>{t("flightManagement.business")}</span><span>{t("flightManagement.first")}</span></td>
          <td><FlightStatusBadge status={flight.status} /></td>
          <td><Link aria-label={`${t("flightManagement.view")} ${flight.flightNumber}`} className="xfo-view-link" href={`/admin/flights/${flight.id}`}>{t("flightManagement.view")}<ArrowRight aria-hidden="true" className="size-4" /></Link></td>
        </tr>)}</tbody>
      </table>
    </div> : null}
    {data ? <p aria-live="polite" className="mt-4 text-xs uppercase tracking-wider text-black/50">{t("flightManagement.showing", { count:data.items.length, total:data.total })}</p> : null}
  </section>;
};

export { FlightsWorkspace };
