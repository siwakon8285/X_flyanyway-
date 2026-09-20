"use client";

import { ArrowLeft, Copy, KeyRound, RefreshCw, ShieldAlert } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { FormEvent, useCallback, useEffect, useRef, useState } from "react";

import { ApiClientStatusBadge } from "./ApiClientsWorkspace";
import { safeAuditActionKey, scopeDescriptionKey, scopeTitleKey, statusKey } from "./apiClientPresentation";
import { Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/Dialog";
import { useLanguage } from "@/i18n/LanguageProvider";
import { formatStaffDateTime } from "@/i18n/formatters";
import type { ApiClientCredentialMetadata, ApiClientDetail, ApiClientRecord, ApiClientScope, ApiScopeDefinition, IssuedCredentialResponse } from "@/lib/admin/apiClientTypes";
import "./apiClientOperations.css";

type Mode = "new" | "detail";
type FormState = { name:string; description:string; status:"ACTIVE" | "SUSPENDED"; allowedScopes:ApiClientScope[] };
type LifecycleAction = "activate" | "suspend" | "revoke";
const blank: FormState = { name:"", description:"", status:"SUSPENDED", allowedScopes:[] };
const emptyCredentialMetadata:ApiClientCredentialMetadata = { hasLiveCredential:false, issuedAt:null, revokedAt:null };
const formFromRecord = (record:ApiClientRecord):FormState => ({
  name:record.name,
  description:record.description ?? "",
  status:record.status === "ACTIVE" ? "ACTIVE" : "SUSPENDED",
  allowedScopes:record.allowedScopes,
});

const ApiClientEditor = ({ mode, clientId, canManage }: { mode:Mode; clientId?:string; canManage:boolean }) => {
  const { locale, t } = useLanguage();
  const router = useRouter();
  const [catalog, setCatalog] = useState<ApiScopeDefinition[]>([]);
  const [client, setClient] = useState<ApiClientRecord | null>(null);
  const [audit, setAudit] = useState<ApiClientDetail["audit"]>([]);
  const [credentialMetadata, setCredentialMetadata] = useState<ApiClientCredentialMetadata>(emptyCredentialMetadata);
  const [form, setForm] = useState<FormState>(blank);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [action, setAction] = useState<LifecycleAction | null>(null);
  const [credentialActionOpen, setCredentialActionOpen] = useState(false);
  const [credentialRevoking, setCredentialRevoking] = useState(false);
  const [issuePending, setIssuePending] = useState(false);
  const [issuedCredential, setIssuedCredential] = useState<IssuedCredentialResponse | null>(null);
  const [credentialAcknowledged, setCredentialAcknowledged] = useState(false);
  const [copiedCredentialField, setCopiedCredentialField] = useState<"clientId" | "clientSecret" | null>(null);
  const [issueRecovery, setIssueRecovery] = useState(false);
  const [recoveryHasLiveCredential, setRecoveryHasLiveCredential] = useState(false);
  const [credentialStatusStale, setCredentialStatusStale] = useState(false);
  const [createdClientId, setCreatedClientId] = useState<string | null>(null);
  const lifecycleTriggerRef = useRef<HTMLButtonElement | null>(null);
  const lifecycleCancelRef = useRef<HTMLButtonElement | null>(null);
  const credentialTriggerRef = useRef<HTMLButtonElement | null>(null);
  const credentialCancelRef = useRef<HTMLButtonElement | null>(null);

  const applyRecord = useCallback((record:ApiClientRecord) => {
    setClient(record);
    setForm(formFromRecord(record));
  }, []);

  const applyDetail = useCallback((detail:ApiClientDetail) => {
    applyRecord(detail);
    setAudit(detail.audit);
    setCredentialMetadata(detail.credentialMetadata ?? emptyCredentialMetadata);
  }, [applyRecord]);

  const requestDetail = useCallback(async (signal?:AbortSignal) => {
    if (!clientId) throw new Error("missing-client-id");
    const response = await fetch(`/admin/api/api-clients/${clientId}`, {
      cache:"no-store",
      credentials:"same-origin",
      signal,
    });
    if (!response.ok) throw new Error("detail-load");
    return response.json() as Promise<ApiClientDetail>;
  }, [clientId]);

  useEffect(() => {
    const controller = new AbortController();
    const get = async <T,>(url:string):Promise<T> => {
      const response = await fetch(url, { cache:"no-store", credentials:"same-origin", signal:controller.signal });
      if (!response.ok) throw new Error("load");
      return response.json() as Promise<T>;
    };
    Promise.all([
      get<ApiScopeDefinition[]>("/admin/api/api-clients/scopes"),
      mode === "detail" && clientId ? requestDetail(controller.signal) : Promise.resolve(null),
    ])
      .then(([nextCatalog, detail]) => {
        setCatalog(nextCatalog);
        if (detail) applyDetail(detail);
      })
      .catch((cause) => { if (cause?.name !== "AbortError") setError(t("apiClientManagement.error")); })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, [applyDetail, clientId, mode, requestDetail, t]);

  const synchronizeDetail = async (record:ApiClientRecord) => {
    applyRecord(record);
    try {
      applyDetail(await requestDetail());
      return true;
    } catch {
      setError(t("apiClientManagement.errors.synchronization"));
      return false;
    }
  };

  const refreshDetail = async () => {
    const latest = await requestDetail();
    applyDetail(latest);
    return latest;
  };

  const toggleScope = (scope:ApiClientScope) => setForm((current) => ({
    ...current,
    allowedScopes:current.allowedScopes.includes(scope)
      ? current.allowedScopes.filter((value) => value !== scope)
      : [...current.allowedScopes, scope].sort(),
  }));

  const validate = () => {
    const nameLength = form.name.trim().length;
    if (nameLength < 1 || nameLength > 100) return t("apiClientManagement.validation.name");
    if (form.description.trim().length > 500) return t("apiClientManagement.validation.description");
    if ((mode === "new" ? form.status === "ACTIVE" : client?.status === "ACTIVE") && form.allowedScopes.length === 0) return t("apiClientManagement.validation.activeScope");
    return "";
  };

  const errorMessage = async (response:Response) => {
    const data = await response.json().catch(() => null);
    switch (data?.error?.code) {
      case "API_CLIENT_MANAGEMENT_UNAVAILABLE": return t("apiClientManagement.error");
      case "STAFF_PERMISSION_DENIED": return t("apiClientManagement.errors.forbidden");
      case "API_CLIENT_STALE_VERSION": return t("apiClientManagement.errors.conflict");
      case "API_CLIENT_STATUS_CONFLICT": return t("apiClientManagement.errors.status");
      case "API_CLIENT_NOT_FOUND": return t("apiClientManagement.errors.notFound");
      default: return t("apiClientManagement.validation.generic");
    }
  };

  const credentialErrorMessage = async (response:Response) => {
    const data = await response.json().catch(() => null);
    switch (data?.error?.code) {
      case "STAFF_PERMISSION_DENIED": return t("apiClientManagement.errors.forbidden");
      case "API_CLIENT_STALE_VERSION": return t("apiClientManagement.errors.credentialConflict");
      case "API_CLIENT_CREDENTIAL_NOT_FOUND": return t("apiClientManagement.errors.credentialConflict");
      case "API_CLIENT_CREDENTIAL_EXISTS": return t("apiClientManagement.errors.credentialConflict");
      case "API_CLIENT_STATUS_CONFLICT": return t("apiClientManagement.errors.credentialConflict");
      case "API_CLIENT_NOT_FOUND": return t("apiClientManagement.errors.notFound");
      default: return t("apiClientManagement.errors.credentialUnavailable");
    }
  };

  const clearIssuedCredential = () => {
    setIssuedCredential(null);
    setCredentialAcknowledged(false);
    setCopiedCredentialField(null);
  };

  const issueCredential = async () => {
    if (!client || client.status === "REVOKED" || credentialMetadata.hasLiveCredential || issuePending || issueRecovery || credentialStatusStale) return;
    setIssuePending(true);
    setIssueRecovery(false);
    setRecoveryHasLiveCredential(false);
    setError(""); setMessage("");
    let responseReceived = false;
    try {
      const response = await fetch(`/admin/api/api-clients/${client.clientId}/credentials`, {
        method:"POST", credentials:"same-origin", cache:"no-store",
        headers:{ "content-type":"application/json", "x-x-fly-csrf":"1" },
        body:JSON.stringify({ version:client.version }),
      });
      if (!response.ok) {
        responseReceived = response.status < 500;
        const message = await credentialErrorMessage(response);
        if (response.status === 409) {
          await refreshDetail().catch(() => undefined);
        }
        if (responseReceived) { setError(message); return; }
        throw new Error("credential-issuance-unavailable");
      }
      const issued = await response.json() as Partial<IssuedCredentialResponse>;
      if (typeof issued.clientId !== "string" || typeof issued.clientSecret !== "string" || typeof issued.issuedAt !== "string") {
        throw new Error(t("apiClientManagement.errors.credentialUnavailable"));
      }
      responseReceived = true;
      setIssuedCredential({ clientId:issued.clientId, clientSecret:issued.clientSecret, issuedAt:issued.issuedAt });
      setCredentialAcknowledged(false);
      setCopiedCredentialField(null);
      await refreshDetail().catch(() => {
        setError(t("apiClientManagement.errors.synchronization"));
      });
    } catch {
      if (!responseReceived) {
        setIssueRecovery(true);
        setError(t("apiClientManagement.credentialRecovery"));
        try {
          const latest = await refreshDetail();
          const hasLiveCredential = latest.credentialMetadata.hasLiveCredential;
          setRecoveryHasLiveCredential(hasLiveCredential);
          setIssueRecovery(hasLiveCredential);
        } catch {
          setRecoveryHasLiveCredential(false);
        }
      } else setError(t("apiClientManagement.errors.credentialUnavailable"));
    } finally {
      setIssuePending(false);
    }
  };

  const refreshCredentialStatus = async () => {
    setError("");
    try {
      const latest = await refreshDetail();
      const hasLiveCredential = latest.credentialMetadata.hasLiveCredential;
      setRecoveryHasLiveCredential(hasLiveCredential);
      setIssueRecovery(hasLiveCredential);
      setCredentialStatusStale(false);
    } catch {
      setError(t("apiClientManagement.errors.synchronization"));
    }
  };

  const revokeCredential = async () => {
    if (!client || credentialRevoking || credentialStatusStale || !credentialMetadata.hasLiveCredential) return;
    setCredentialRevoking(true); setError(""); setMessage("");
    try {
      const response = await fetch(`/admin/api/api-clients/${client.clientId}/credentials/revoke`, {
        method:"POST", credentials:"same-origin", cache:"no-store",
        headers:{ "content-type":"application/json", "x-x-fly-csrf":"1" },
        body:JSON.stringify({ version:client.version }),
      });
      if (!response.ok) {
        const message = await credentialErrorMessage(response);
        if (response.status === 409) await refreshDetail().catch(() => undefined);
        setError(message);
        return;
      }
      setCredentialActionOpen(false);
      setIssueRecovery(false);
      setRecoveryHasLiveCredential(false);
      setCredentialMetadata((current) => ({ ...current, hasLiveCredential:false }));
      setCredentialStatusStale(true);
      try {
        await refreshDetail();
        setCredentialStatusStale(false);
        setMessage(t("apiClientManagement.credentialRevoked"));
      } catch {
        setMessage(t("apiClientManagement.credentialRevoked"));
        setError(t("apiClientManagement.errors.credentialRevokedRefresh"));
      }
    } catch {
      setError(t("apiClientManagement.errors.credentialUnavailable"));
    } finally {
      setCredentialRevoking(false);
    }
  };

  const copyCredential = async (field:"clientId" | "clientSecret", value:string) => {
    try {
      await navigator.clipboard.writeText(value);
      setCopiedCredentialField(field);
    } catch {
      setError(t("apiClientManagement.copyFailed"));
    }
  };

  const save = async (event:FormEvent) => {
    event.preventDefault();
    if (saving || createdClientId) return;
    setError(""); setMessage("");
    const invalid = validate(); if (invalid) { setError(invalid); return; }
    setSaving(true);
    const payload = mode === "new"
      ? { name:form.name, description:form.description.trim() || null, status:form.status, allowedScopes:form.allowedScopes }
      : { name:form.name, description:form.description.trim() || null, allowedScopes:form.allowedScopes, version:client?.version };
    try {
      const response = await fetch(mode === "new" ? "/admin/api/api-clients" : `/admin/api/api-clients/${clientId}`, {
        method:mode === "new" ? "POST" : "PUT",
        credentials:"same-origin",
        cache:"no-store",
        headers:{ "content-type":"application/json", "x-x-fly-csrf":"1" },
        body:JSON.stringify(payload),
      });
      if (!response.ok) { setError(await errorMessage(response)); return; }
      const saved = await response.json() as ApiClientRecord;
      if (mode === "new") {
        applyRecord(saved);
        setMessage(t("apiClientManagement.registered"));
        setCreatedClientId(saved.clientId);
        try {
          router.replace(`/admin/api-clients/${saved.clientId}`);
        } catch {
          // Keep the authoritative result visible and creation disabled if navigation cannot start.
        }
      } else if (await synchronizeDetail(saved)) {
        setMessage(t("apiClientManagement.saved"));
      }
    } catch { setError(t("apiClientManagement.error")); }
    finally { setSaving(false); }
  };

  const transition = async () => {
    if (!client || !action) return;
    setSaving(true); setError(""); setMessage("");
    try {
      const response = await fetch(`/admin/api/api-clients/${client.clientId}/${action}`, {
        method:"POST", credentials:"same-origin", cache:"no-store",
        headers:{ "content-type":"application/json", "x-x-fly-csrf":"1" },
        body:JSON.stringify({ version:client.version }),
      });
      if (!response.ok) { setError(await errorMessage(response)); return; }
      const updated = await response.json() as ApiClientRecord;
      setAction(null);
      await synchronizeDetail(updated);
    } catch { setError(t("apiClientManagement.error")); }
    finally { setSaving(false); }
  };

  if (loading) return <p aria-live="polite">{t("apiClientManagement.loading")}</p>;
  if (error && catalog.length === 0) return <p role="alert">{error}</p>;
  const revoked = client?.status === "REVOKED";
  const disabled = !canManage || revoked || createdClientId !== null || saving || issuePending || credentialRevoking;
  const heading = mode === "new" ? t("apiClientManagement.registerHeading") : client?.name ?? t("apiClientManagement.errors.notFound");
  const actionCopy = action ? {
    activate:["apiClientManagement.activateTitle", "apiClientManagement.activateDescription", "apiClientManagement.confirmActivate"],
    suspend:["apiClientManagement.suspendTitle", "apiClientManagement.suspendDescription", "apiClientManagement.confirmSuspend"],
    revoke:["apiClientManagement.revokeTitle", "apiClientManagement.revokeDescription", "apiClientManagement.confirmRevoke"],
  }[action] as ["apiClientManagement.activateTitle" | "apiClientManagement.suspendTitle" | "apiClientManagement.revokeTitle", "apiClientManagement.activateDescription" | "apiClientManagement.suspendDescription" | "apiClientManagement.revokeDescription", "apiClientManagement.confirmActivate" | "apiClientManagement.confirmSuspend" | "apiClientManagement.confirmRevoke"] : null;

  return <section className="xac-terminal">
    <header className="xac-editor-header"><div><Link className="xac-back" href="/admin/api-clients"><ArrowLeft aria-hidden="true" className="size-4" />{t("apiClientManagement.back")}</Link><p className="xac-kicker">{t("apiClientManagement.detailEyebrow")}</p><h1>{heading}</h1>{client ? <div className="xac-client-line"><code>{client.clientId}</code><ApiClientStatusBadge status={client.status} /></div> : null}</div><div className="xac-control-plate"><span>XF / IDENTITY</span><strong>{t("apiClientManagement.terminal")}</strong>{!canManage ? <span className="xac-readonly">{t("apiClientManagement.readOnly")}</span> : null}</div></header>
    <p className="xac-boundary"><ShieldAlert aria-hidden="true" className="size-4" />{t("apiClientManagement.eligibleBoundary")}</p>
    <form aria-label={heading} className="xac-editor" onSubmit={(event) => void save(event)}>
      <fieldset><legend><span>01</span>{t("apiClientManagement.identity")}</legend>{client ? <div className="xac-immutable"><label>{t("apiClientManagement.clientId")}</label><code>{client.clientId}</code><small>{t("apiClientManagement.immutable")}</small></div> : null}<label>{t("apiClientManagement.displayName")}<input aria-label={t("apiClientManagement.displayName")} disabled={disabled} maxLength={100} onChange={(event) => setForm((current) => ({ ...current, name:event.target.value }))} required value={form.name} /></label><label>{t("apiClientManagement.description")}<textarea aria-label={t("apiClientManagement.description")} disabled={disabled} maxLength={500} onChange={(event) => setForm((current) => ({ ...current, description:event.target.value }))} rows={4} value={form.description} /></label>{mode === "new" ? <label>{t("apiClientManagement.initialStatus")}<select aria-label={t("apiClientManagement.initialStatus")} disabled={disabled} onChange={(event) => setForm((current) => ({ ...current, status:event.target.value as FormState["status"] }))} value={form.status}><option value="SUSPENDED">{t(statusKey.SUSPENDED)}</option><option value="ACTIVE">{t(statusKey.ACTIVE)}</option></select></label> : null}</fieldset>
      <fieldset><legend><span>02</span>{t("apiClientManagement.configuration")}</legend><p>{t("apiClientManagement.selectScopes")}</p><div className="xac-scope-options">{catalog.map((scope) => <label key={scope.code}><input checked={form.allowedScopes.includes(scope.code)} disabled={disabled} onChange={() => toggleScope(scope.code)} type="checkbox" /><span><strong>{t(scopeTitleKey[scope.code])}</strong><code>{scope.code}</code><small>{t(scopeDescriptionKey[scope.code])}</small></span></label>)}</div></fieldset>
      <div aria-live="polite" className="xac-feedback">{error ? <p role="alert">{error}</p> : null}{message ? <p>{message}</p> : null}{mode === "new" && client ? <Link href={`/admin/api-clients/${client.clientId}`}>{client.clientId}</Link> : null}</div>
      {canManage && !revoked ? <div className="xac-editor-actions"><button className="xac-primary" disabled={disabled} type="submit">{saving ? t("apiClientManagement.saving") : t(mode === "new" ? "apiClientManagement.register" : "apiClientManagement.save")}</button>{mode === "detail" && client?.status === "SUSPENDED" ? <button disabled={saving} onClick={(event) => { lifecycleTriggerRef.current = event.currentTarget; setAction("activate"); }} type="button">{t("apiClientManagement.activate")}</button> : null}{mode === "detail" && client?.status === "ACTIVE" ? <button disabled={saving} onClick={(event) => { lifecycleTriggerRef.current = event.currentTarget; setAction("suspend"); }} type="button">{t("apiClientManagement.suspend")}</button> : null}{mode === "detail" ? <button className="is-danger" disabled={saving || credentialRevoking} onClick={(event) => { lifecycleTriggerRef.current = event.currentTarget; setAction("revoke"); }} type="button">{t("apiClientManagement.revoke")}</button> : null}</div> : null}
    </form>
    {mode === "detail" && client ? <section aria-labelledby="api-client-credentials" className="xac-credentials"><div className="xac-credentials-heading"><div><p className="xac-kicker">{t("apiClientManagement.credentials")}</p><h2 id="api-client-credentials">{credentialMetadata.hasLiveCredential ? t("apiClientManagement.credentialStatusLive") : t("apiClientManagement.credentialStatusNone")}</h2></div><KeyRound aria-hidden="true" className="size-6" /></div><dl className="xac-credential-metadata"><div><dt>{t("apiClientManagement.credentialIssuedAt")}</dt><dd>{credentialMetadata.issuedAt ? <time dateTime={credentialMetadata.issuedAt}>{formatStaffDateTime(credentialMetadata.issuedAt, locale)}</time> : "—"}</dd></div><div><dt>{t("apiClientManagement.credentialRevokedAt")}</dt><dd>{credentialMetadata.revokedAt ? <time dateTime={credentialMetadata.revokedAt}>{formatStaffDateTime(credentialMetadata.revokedAt, locale)}</time> : "—"}</dd></div></dl>{canManage && !revoked && !credentialMetadata.hasLiveCredential && !credentialStatusStale ? <button className="xac-primary" disabled={issuePending || saving || issueRecovery} onClick={(event) => { credentialTriggerRef.current = event.currentTarget; void issueCredential(); }} type="button">{issuePending ? t("apiClientManagement.issuingCredential") : t("apiClientManagement.issueCredential")}</button> : null}{canManage && !revoked && credentialMetadata.hasLiveCredential && !credentialStatusStale ? <button className="xac-credential-danger" disabled={credentialRevoking || saving} onClick={(event) => { credentialTriggerRef.current = event.currentTarget; setCredentialActionOpen(true); }} type="button">{t("apiClientManagement.revokeCredential")}</button> : null}{issueRecovery ? <div className="xac-credential-recovery" role="status"><p>{recoveryHasLiveCredential ? t("apiClientManagement.credentialRecoveryReady") : t("apiClientManagement.credentialRecovery")}</p><button onClick={() => void refreshCredentialStatus()} type="button"><RefreshCw aria-hidden="true" className="size-4" />{t("apiClientManagement.refreshCredentialStatus")}</button></div> : null}{credentialStatusStale ? <div className="xac-credential-recovery" role="status"><p>{t("apiClientManagement.errors.credentialRevokedRefresh")}</p><button onClick={() => void refreshCredentialStatus()} type="button"><RefreshCw aria-hidden="true" className="size-4" />{t("apiClientManagement.refreshCredentialStatus")}</button></div> : null}</section> : null}
    {client ? <section aria-labelledby="api-client-audit" className="xac-audit"><h2 id="api-client-audit">{t("apiClientManagement.history")}</h2><div className="xac-actor"><span>{t("apiClientManagement.createdBy", { actor:client.createdBy })}</span><span>{t("apiClientManagement.actor", { actor:client.updatedBy })}</span></div>{audit.length ? <ol>{audit.map((entry, index) => { const actionKey = safeAuditActionKey(entry.action); return <li key={`${entry.createdAt}-${entry.action}-${index}`}><div><strong>{t(actionKey)}</strong><span>{t("apiClientManagement.audit.by", { action:t(actionKey), actor:entry.actorEmail })}</span></div><time dateTime={entry.createdAt}>{formatStaffDateTime(entry.createdAt, locale)}</time></li>; })}</ol> : <p>{t("apiClientManagement.noHistory")}</p>}</section> : null}
    <Dialog onOpenChange={(open) => { if (!open) setAction(null); }} open={action !== null}><DialogContent onCloseAutoFocus={(event) => { event.preventDefault(); lifecycleTriggerRef.current?.focus(); }} onOpenAutoFocus={(event) => { event.preventDefault(); lifecycleCancelRef.current?.focus(); }} showCloseButton={false}><DialogHeader><DialogTitle>{actionCopy ? t(actionCopy[0]) : ""}</DialogTitle><DialogDescription>{actionCopy ? t(actionCopy[1]) : ""}</DialogDescription></DialogHeader><DialogFooter><DialogClose asChild><button ref={lifecycleCancelRef} type="button">{t("apiClientManagement.cancelAction")}</button></DialogClose><button className={action === "revoke" ? "xac-dialog-danger" : "xac-dialog-confirm"} disabled={saving} onClick={() => void transition()} type="button">{actionCopy ? t(actionCopy[2]) : ""}</button></DialogFooter></DialogContent></Dialog>
    <Dialog onOpenChange={(open) => { if (!open && credentialAcknowledged) clearIssuedCredential(); }} open={issuedCredential !== null}><DialogContent aria-describedby="credential-issued-description" aria-labelledby="credential-issued-title" onEscapeKeyDown={(event) => { if (!credentialAcknowledged) event.preventDefault(); }} onInteractOutside={(event) => { if (!credentialAcknowledged) event.preventDefault(); }} onPointerDownOutside={(event) => { if (!credentialAcknowledged) event.preventDefault(); }} onCloseAutoFocus={(event) => { event.preventDefault(); credentialTriggerRef.current?.focus(); }} showCloseButton={false}><DialogHeader><DialogTitle id="credential-issued-title">{t("apiClientManagement.credentialIssuedTitle")}</DialogTitle><DialogDescription id="credential-issued-description">{t("apiClientManagement.credentialIssuedDescription")}</DialogDescription></DialogHeader>{issuedCredential ? <div className="xac-secret-panel"><div><span>{t("apiClientManagement.credentialClientId")}</span><code data-testid="credential-client-id">{issuedCredential.clientId}</code><button aria-label={t("apiClientManagement.copyClientId")} onClick={() => void copyCredential("clientId", issuedCredential.clientId)} type="button"><Copy aria-hidden="true" className="size-4" />{copiedCredentialField === "clientId" ? t("apiClientManagement.copied") : t("apiClientManagement.copyClientId")}</button></div><div><span>{t("apiClientManagement.credentialClientSecret")}</span><code data-testid="credential-secret">{issuedCredential.clientSecret}</code><button aria-label={t("apiClientManagement.copyClientSecret")} onClick={() => void copyCredential("clientSecret", issuedCredential.clientSecret)} type="button"><Copy aria-hidden="true" className="size-4" />{copiedCredentialField === "clientSecret" ? t("apiClientManagement.copied") : t("apiClientManagement.copyClientSecret")}</button></div>{copiedCredentialField ? <p aria-live="polite">{t("apiClientManagement.copied")}</p> : null}<p>{t("apiClientManagement.credentialCannotRecover")}</p><label className="xac-acknowledgement"><input checked={credentialAcknowledged} onChange={(event) => setCredentialAcknowledged(event.target.checked)} type="checkbox" />{t("apiClientManagement.credentialAcknowledgement")}</label></div> : null}<DialogFooter><button disabled={!credentialAcknowledged} onClick={clearIssuedCredential} type="button">{t("apiClientManagement.continue")}</button></DialogFooter></DialogContent></Dialog>
    <Dialog onOpenChange={setCredentialActionOpen} open={credentialActionOpen}><DialogContent onCloseAutoFocus={(event) => { event.preventDefault(); credentialTriggerRef.current?.focus(); }} onOpenAutoFocus={(event) => { event.preventDefault(); credentialCancelRef.current?.focus(); }} showCloseButton={false}><DialogHeader><DialogTitle>{t("apiClientManagement.revokeCredentialTitle")}</DialogTitle><DialogDescription>{t("apiClientManagement.revokeCredentialDescription")}</DialogDescription></DialogHeader><DialogFooter><DialogClose asChild><button ref={credentialCancelRef} type="button">{t("apiClientManagement.cancelCredential")}</button></DialogClose><button className="xac-dialog-danger" disabled={credentialRevoking} onClick={() => void revokeCredential()} type="button">{credentialRevoking ? t("apiClientManagement.revokingCredential") : t("apiClientManagement.confirmCredentialRevoke")}</button></DialogFooter></DialogContent></Dialog>
  </section>;
};

export { ApiClientEditor };
