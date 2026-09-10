"use client";

import { ArrowRight, Filter, Plus } from "lucide-react";
import Link from "next/link";
import { FormEvent, useEffect, useState } from "react";

import { scopeTitleKey, statusKey } from "./apiClientPresentation";
import { useLanguage } from "@/i18n/LanguageProvider";
import { formatStaffDateTime } from "@/i18n/formatters";
import type { ApiClientPage, ApiClientRecord, ApiClientScope, ApiClientStatus } from "@/lib/admin/apiClientTypes";
import "./apiClientOperations.css";

type Filters = { search:string; status:"" | ApiClientStatus; scope:"" | ApiClientScope };
const initialFilters: Filters = { search:"", status:"", scope:"" };

const ApiClientsWorkspace = ({ canManage }: { canManage:boolean }) => {
  const { locale, t } = useLanguage();
  const [filters, setFilters] = useState<Filters>(initialFilters);
  const [query, setQuery] = useState("?limit=50&offset=0");
  const [items, setItems] = useState<ApiClientRecord[]>([]);
  const [nextOffset, setNextOffset] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const [retry, setRetry] = useState(0);

  useEffect(() => {
    const controller = new AbortController();
    fetch(`/admin/api/api-clients${query}`, { cache:"no-store", credentials:"same-origin", signal:controller.signal })
      .then(async (response) => { if (!response.ok) throw new Error("api clients"); return response.json() as Promise<ApiClientPage>; })
      .then((page) => { setItems(page.items); setNextOffset(page.nextOffset); setError(false); })
      .catch((cause) => { if (cause?.name !== "AbortError") setError(true); })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, [query, retry]);

  const apply = (event: FormEvent) => {
    event.preventDefault();
    const params = new URLSearchParams({ limit:"50", offset:"0" });
    if (filters.search.trim()) params.set("search", filters.search.trim());
    if (filters.status) params.set("status", filters.status);
    if (filters.scope) params.set("scope", filters.scope);
    setLoading(true);
    setError(false);
    setQuery(`?${params}`);
  };

  const loadMore = async () => {
    if (nextOffset === null) return;
    const params = new URLSearchParams(query);
    params.set("offset", String(nextOffset));
    setLoading(true); setError(false);
    try {
      const response = await fetch(`/admin/api/api-clients?${params}`, { cache:"no-store", credentials:"same-origin" });
      if (!response.ok) throw new Error("api clients");
      const page = await response.json() as ApiClientPage;
      setItems((current) => [...current, ...page.items]);
      setNextOffset(page.nextOffset);
    } catch { setError(true); } finally { setLoading(false); }
  };

  return <section aria-labelledby="api-client-management-title" className="xac-terminal">
    <header className="xac-header">
      <div><p className="xac-kicker">{t("apiClientManagement.eyebrow")}</p><h1 id="api-client-management-title">{t("apiClientManagement.title")}</h1><p className="xac-intro">{t("apiClientManagement.intro")}</p></div>
      <div className="xac-control-plate"><span>XF / INT—24</span><strong>{t("apiClientManagement.terminal")}</strong>{canManage ? <Link className="xac-primary" href="/admin/api-clients/new"><Plus aria-hidden="true" className="size-4" />{t("apiClientManagement.register")}</Link> : <span className="xac-readonly">{t("apiClientManagement.readOnly")}</span>}</div>
    </header>
    <p className="xac-boundary">{t("apiClientManagement.boundary")}</p>

    <form aria-label={t("apiClientManagement.filters")} className="xac-filters" onSubmit={apply}>
      <div className="xac-filter-title"><Filter aria-hidden="true" className="size-4" />{t("apiClientManagement.filters")}</div>
      <label>{t("apiClientManagement.searchLabel")}<input aria-label={t("apiClientManagement.searchLabel")} maxLength={100} onChange={(event) => setFilters((current) => ({ ...current, search:event.target.value }))} placeholder={t("apiClientManagement.searchPlaceholder")} value={filters.search} /></label>
      <label>{t("apiClientManagement.statusLabel")}<select aria-label={t("apiClientManagement.statusLabel")} onChange={(event) => setFilters((current) => ({ ...current, status:event.target.value as Filters["status"] }))} value={filters.status}><option value="">{t("apiClientManagement.all")}</option><option value="ACTIVE">{t(statusKey.ACTIVE)}</option><option value="SUSPENDED">{t(statusKey.SUSPENDED)}</option><option value="REVOKED">{t(statusKey.REVOKED)}</option></select></label>
      <label>{t("apiClientManagement.scopeLabel")}<select aria-label={t("apiClientManagement.scopeLabel")} onChange={(event) => setFilters((current) => ({ ...current, scope:event.target.value as Filters["scope"] }))} value={filters.scope}><option value="">{t("apiClientManagement.all")}</option><option value="analytics:read">{t(scopeTitleKey["analytics:read"])}</option><option value="flights:read">{t(scopeTitleKey["flights:read"])}</option></select></label>
      <div className="xac-filter-actions"><button type="submit">{t("apiClientManagement.apply")}</button><button onClick={() => { setFilters(initialFilters); setLoading(true); setError(false); setQuery("?limit=50&offset=0"); }} type="button">{t("apiClientManagement.clear")}</button></div>
    </form>

    {loading && items.length === 0 ? <p aria-live="polite" className="xac-state">{t("apiClientManagement.loading")}</p> : null}
    {error ? <div className="xac-error" role="alert"><p>{t("apiClientManagement.error")}</p><button onClick={() => { setLoading(true); setError(false); setRetry((value) => value + 1); }} type="button">{t("apiClientManagement.retry")}</button></div> : null}
    {!loading && !error && items.length === 0 ? <div className="xac-empty"><strong>{t("apiClientManagement.empty")}</strong><p>{t("apiClientManagement.emptyHelp")}</p></div> : null}
    {items.length ? <div aria-label={t("apiClientManagement.registry")} className="xac-registry overflow-x-auto" role="region" tabIndex={0}><table><caption className="sr-only">{t("apiClientManagement.registry")}</caption><thead><tr><th scope="col">{t("apiClientManagement.clientId")}</th><th scope="col">{t("apiClientManagement.name")}</th><th scope="col">{t("apiClientManagement.statusLabel")}</th><th scope="col">{t("apiClientManagement.scopes")}</th><th scope="col">{t("apiClientManagement.created")}</th><th scope="col">{t("apiClientManagement.updated")}</th><th scope="col">{t("apiClientManagement.actions")}</th></tr></thead><tbody>{items.map((client) => <tr key={client.clientId}><th scope="row"><code>{client.clientId}</code></th><td><strong>{client.name}</strong>{client.description ? <span>{client.description}</span> : null}</td><td><Status status={client.status} /></td><td><ScopeSummary scopes={client.allowedScopes} /></td><td><time dateTime={client.createdAt}>{formatStaffDateTime(client.createdAt, locale)}</time></td><td><time dateTime={client.updatedAt}>{formatStaffDateTime(client.updatedAt, locale)}</time></td><td><Link aria-label={`${t("apiClientManagement.inspect")} ${client.name}`} className="xac-inspect" href={`/admin/api-clients/${client.clientId}`}>{t("apiClientManagement.inspect")}<ArrowRight aria-hidden="true" className="size-4" /></Link></td></tr>)}</tbody></table></div> : null}
    {nextOffset !== null ? <button className="xac-load" disabled={loading} onClick={() => void loadMore()} type="button">{t("apiClientManagement.loadMore")}</button> : items.length ? <p className="xac-end">{t("apiClientManagement.end")}</p> : null}
  </section>;
};

const Status = ({ status }: { status:ApiClientStatus }) => { const { t } = useLanguage(); return <span className={`xac-status is-${status.toLowerCase()}`}><span aria-hidden="true" />{t(statusKey[status])}</span>; };
const ScopeSummary = ({ scopes }: { scopes:ApiClientScope[] }) => { const { t } = useLanguage(); return scopes.length ? <div className="xac-scope-list">{scopes.map((scope) => <span key={scope}><code>{scope}</code><small>{t(scopeTitleKey[scope])}</small></span>)}</div> : <span>{t("apiClientManagement.noScopes")}</span>; };

export { ApiClientsWorkspace, ScopeSummary, Status as ApiClientStatusBadge };
