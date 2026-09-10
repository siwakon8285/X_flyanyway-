"use client";

import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { AirportCombobox } from "@/components/admin/flights/AirportCombobox";
import { DatePickerField } from "@/components/admin/flights/DatePickerField";
import { OperationalSummary, OperationsSectionLegend } from "@/components/admin/flights/FlightOperationsPrimitives";
import { TimePickerField } from "@/components/admin/flights/TimePickerField";
import { Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/Dialog";
import { useLanguage } from "@/i18n/LanguageProvider";
import { formatStaffDate, formatStaffDateTime } from "@/i18n/formatters";
import type { FlightDetail, FlightReferenceData, ManagedFlight } from "@/lib/admin/flightTypes";
import "./flightOperations.css";

type FormState = { flightNumber: string; originCode: string; destinationCode: string; operatingDate: string; departureTime: string; arrivalTime: string; nextDay: boolean; aircraftCode: string; businessPrice: string; firstPrice: string; businessCapacity: string; firstCapacity: string };
const blank: FormState = { flightNumber: "", originCode: "", destinationCode: "", operatingDate: "", departureTime: "", arrivalTime: "", nextDay: false, aircraftCode: "", businessPrice: "", firstPrice: "", businessCapacity: "16", firstCapacity: "4" };
const fromFlight = (flight: ManagedFlight): FormState => ({ flightNumber: flight.flightNumber, originCode: flight.originCode, destinationCode: flight.destinationCode, operatingDate: flight.operatingDate ?? "", departureTime: flight.departureTime?.slice(0, 5) ?? "", arrivalTime: flight.arrivalTime?.slice(0, 5) ?? "", nextDay: flight.arrivalDayOffset === 1, aircraftCode: flight.aircraftCode, businessPrice: String(flight.business.priceAmount ?? ""), firstPrice: String(flight.first.priceAmount ?? ""), businessCapacity: String(flight.business.capacity), firstCapacity: String(flight.first.capacity) });
const auditAction = { FLIGHT_CREATED: "flightManagement.actionCreated", FLIGHT_EDITED: "flightManagement.actionEdited", FLIGHT_CANCELLED: "flightManagement.actionCancelled" } as const;

const FlightEditor = ({ mode, flightId, canWrite }: { mode: "new" | "detail"; flightId?: string; canWrite: boolean }) => {
  const { locale, t } = useLanguage();
  const [references, setReferences] = useState<FlightReferenceData | null>(null);
  const [flight, setFlight] = useState<FlightDetail | null>(null);
  const [form, setForm] = useState<FormState>(blank);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const [cancelOpen, setCancelOpen] = useState(false);
  const cancelTriggerRef = useRef<HTMLButtonElement | null>(null);
  const keepScheduledRef = useRef<HTMLButtonElement | null>(null);

  useEffect(() => {
    const controller = new AbortController(); setLoading(true);
    Promise.all([fetch("/admin/api/flights/reference-data", { cache: "no-store", credentials: "same-origin", signal: controller.signal }).then(async r => { if (!r.ok) throw new Error(); return r.json() as Promise<FlightReferenceData>; }),
    mode === "detail" && flightId ? fetch(`/admin/api/flights/${flightId}`, { cache: "no-store", credentials: "same-origin", signal: controller.signal }).then(async r => { if (!r.ok) throw new Error(); return r.json() as Promise<FlightDetail> }) : Promise.resolve(null)])
      .then(([refs, detail]) => { setReferences(refs); if (detail) { setFlight(detail); setForm(fromFlight(detail)); } }).catch(cause => { if (cause?.name !== "AbortError") setError(t("flightManagement.error")); }).finally(() => setLoading(false)); return () => controller.abort();
  }, [flightId, mode, t]);

  const set = (key: keyof FormState, value: string | boolean) => setForm(current => ({ ...current, [key]: value }));
  const valid = () => /^XF\s\d{3}$/i.test(form.flightNumber.trim()) && form.originCode !== form.destinationCode && [form.originCode, form.destinationCode, form.departureTime, form.arrivalTime, form.aircraftCode].every(Boolean) && (mode === "detail" || Boolean(form.operatingDate))
    && Number(form.businessPrice) > 0 && Number(form.firstPrice) > 0 && Number(form.businessCapacity) > 0 && Number(form.businessCapacity) % 4 === 0 && Number(form.firstCapacity) > 0 && Number(form.firstCapacity) % 2 === 0;
  const payload = () => ({ flightNumber: form.flightNumber, originCode: form.originCode, destinationCode: form.destinationCode, operatingDate: form.operatingDate || null, departureTime: `${form.departureTime}:00`, arrivalTime: `${form.arrivalTime}:00`, arrivalDayOffset: form.nextDay ? 1 : 0, aircraftCode: form.aircraftCode, businessPriceAmount: Number(form.businessPrice), firstPriceAmount: Number(form.firstPrice), currencyCode: "THB", businessCapacity: Number(form.businessCapacity), firstCapacity: Number(form.firstCapacity) });
  const save = async () => {
    setMessage(""); setError(""); if (!valid()) { setError(t("flightManagement.validation")); return; } setSaving(true);
    try {
      const response = await fetch(mode === "new" ? "/admin/api/flights" : `/admin/api/flights/${flightId}`, { method: mode === "new" ? "POST" : "PUT", credentials: "same-origin", headers: { "content-type": "application/json", "X-X-Fly-CSRF": "1" }, body: JSON.stringify(mode === "new" ? payload() : { ...payload(), version: flight?.version }) });
      if (!response.ok) { const body = await response.json().catch(() => null); const code = body?.error?.code; throw new Error(code === "STAFF_PERMISSION_DENIED" ? t("flightManagement.forbidden") : code?.includes("CONFLICT") || code === "FLIGHT_STALE_VERSION" ? t("flightManagement.conflict") : t("flightManagement.validation")); }
      const next = await response.json() as ManagedFlight; setFlight({ ...next, audit: flight?.audit ?? [] }); setForm(fromFlight(next)); setMessage(t("flightManagement.saved"));
    } catch (cause) { setError(cause instanceof Error ? cause.message : t("flightManagement.error")); } finally { setSaving(false); }
  };
  const cancel = async () => {
    if (!flight) return;
    setSaving(true); setError("");
    try {
      const response = await fetch(`/admin/api/flights/${flight.id}/cancel`, { method: "POST", credentials: "same-origin", headers: { "content-type": "application/json", "X-X-Fly-CSRF": "1" }, body: JSON.stringify({ version: flight.version }) });
      if (!response.ok) throw new Error(t("flightManagement.conflict"));
      const next = await response.json() as ManagedFlight;
      setFlight({ ...next, audit: flight.audit }); setForm(fromFlight(next)); setCancelOpen(false);
      try {
        const detailResponse = await fetch(`/admin/api/flights/${flight.id}`, { cache: "no-store", credentials: "same-origin" });
        if (!detailResponse.ok) throw new Error();
        const detail = await detailResponse.json() as FlightDetail;
        setFlight(detail); setForm(fromFlight(detail));
      } catch {
        setError(t("flightManagement.cancelSyncWarning"));
      }
    } catch (cause) { setError(cause instanceof Error ? cause.message : t("flightManagement.error")); } finally { setSaving(false); }
  };

  if (loading) return <p aria-live="polite">{t("flightManagement.loading")}</p>;
  if (error && !references) return <p role="alert">{error}</p>;
  const heading = mode === "new" ? t("flightManagement.create") : flight?.flightNumber ?? t("flightManagement.notFound");
  const disabled = !canWrite || flight?.status === "CANCELLED";
  const inputClass = "xfo-control disabled:bg-black/5";
  const label = (text: string, key: keyof FormState, type = "text") => <label>{text}<input aria-label={text} className={inputClass} disabled={disabled} onChange={event => set(key, event.target.value)} required type={type} value={String(form[key])} /></label>;
  const airport = (text: string, key: "originCode" | "destinationCode") => <AirportCombobox airports={references?.airports ?? []} disabled={disabled} label={text} onChange={value => set(key, value)} value={form[key]} />;

  return <section className="flight-operations"><header className="xfo-editor-header"><div><Link className="xfo-back" href="/admin/flights"><ArrowLeft aria-hidden="true" className="size-4" />{t("flightManagement.back")}</Link><p className="xfo-kicker mt-6">{t("flightManagement.serviceConfiguration")}</p><h1 className="xfo-editor-title">{heading}</h1>{flight ? <p className="xfo-editor-copy">{flight.originCode} → {flight.destinationCode} · {t(flight.status === "SCHEDULED" ? "flightManagement.scheduled" : "flightManagement.cancelled")}</p> : null}{!canWrite ? <p className="xfo-readonly">{t("flightManagement.readOnly")}</p> : null}</div><OperationalSummary aircraftCode={form.aircraftCode} airports={references?.airports ?? []} arrivalTime={form.arrivalTime} businessCapacity={form.businessCapacity} departureTime={form.departureTime} destinationCode={form.destinationCode} firstCapacity={form.firstCapacity} flightNumber={form.flightNumber} operatingDate={form.operatingDate} originCode={form.originCode} status={flight?.status} /></header>
    <form aria-label={heading} className="xfo-editor-form" onSubmit={e => { e.preventDefault(); void save(); }} role="form">
      <fieldset className="xfo-form-section"><OperationsSectionLegend index="01" title={t("flightManagement.identity")} />{label(t("flightManagement.flightNumber"), "flightNumber")}{airport(t("flightManagement.origin"), "originCode")}{airport(t("flightManagement.destination"), "destinationCode")}</fieldset>
      <fieldset className="xfo-form-section"><OperationsSectionLegend index="02" title={t("flightManagement.schedule")} /><DatePickerField disabled={disabled} label={t("flightManagement.departureDate")} onChange={value => set("operatingDate", value)} required value={form.operatingDate} /><TimePickerField disabled={disabled} label={t("flightManagement.departure")} onChange={value => set("departureTime", value)} required value={form.departureTime} /><TimePickerField disabled={disabled} label={t("flightManagement.arrival")} onChange={value => set("arrivalTime", value)} required value={form.arrivalTime} /><label className="xfo-checkbox"><input checked={form.nextDay} className="xfo-checkbox-input" disabled={disabled} onChange={e => set("nextDay", e.target.checked)} type="checkbox" />{t("flightManagement.nextDay")}</label></fieldset>
      <fieldset className="xfo-form-section"><OperationsSectionLegend index="03" title={t("flightManagement.inventory")} /><label>{t("flightManagement.aircraftContext")}<select aria-label={t("flightManagement.aircraftContext")} className={inputClass} disabled={disabled} onChange={e => set("aircraftCode", e.target.value)} required value={form.aircraftCode}><option value="">{t("flightManagement.chooseAircraft")}</option>{references?.aircraft.map(item => <option key={item}>{item}</option>)}</select></label>{label(t("flightManagement.businessCapacity"), "businessCapacity", "number")}{label(t("flightManagement.firstCapacity"), "firstCapacity", "number")}</fieldset>
      <fieldset className="xfo-form-section is-commercial"><OperationsSectionLegend index="04" title={t("flightManagement.commercial")} />{label(t("flightManagement.businessPrice"), "businessPrice", "number")}{label(t("flightManagement.firstPrice"), "firstPrice", "number")}</fieldset>
      <div aria-live="polite" className="xfo-feedback">{error ? <p role="alert" className="text-red-700">{error}</p> : null}{message ? <p className="font-semibold text-green-800">{message}</p> : null}</div>
      {canWrite && !disabled ? <div className="xfo-form-actions"><button className="xfo-save" disabled={saving} type="submit">{saving ? t("flightManagement.saving") : t("flightManagement.save")}</button>{mode === "detail" ? <button className="xfo-destructive" onClick={(event) => { cancelTriggerRef.current = event.currentTarget; setCancelOpen(true); }} type="button">{t("flightManagement.cancelFlight")}</button> : null}</div> : null}
    </form>
    {flight ? <section aria-labelledby="audit-title" className="xfo-audit"><h2 id="audit-title">{t("flightManagement.audit")}</h2>{flight.audit.length ? <ol className="mt-4">{flight.audit.map(item => <li key={item.id}><span>{t("flightManagement.createdBy", { action: t(auditAction[item.action]), actor: item.actorEmail })}</span><time className="text-sm text-black/55">{formatStaffDateTime(item.createdAt, locale)}</time></li>)}</ol> : <p className="mt-3 text-black/55">{t("flightManagement.noAudit")}</p>}</section> : null}
    {flight ? <Dialog onOpenChange={setCancelOpen} open={cancelOpen}><DialogContent onCloseAutoFocus={(event) => { event.preventDefault(); cancelTriggerRef.current?.focus(); }} onOpenAutoFocus={(event) => { event.preventDefault(); keepScheduledRef.current?.focus(); }} showCloseButton={false}><DialogHeader><DialogTitle>{t("flightManagement.cancelTitle", { flight: flight.flightNumber })}</DialogTitle><DialogDescription>{t("flightManagement.cancelDescription", { route: `${flight.originCode} → ${flight.destinationCode}`, departure: `${flight.operatingDate ? formatStaffDate(flight.operatingDate, locale) : t("flightManagement.recurring")} ${flight.departureTime?.slice(0, 5) ?? ""}` })}</DialogDescription></DialogHeader><DialogFooter><DialogClose asChild><button className="min-h-11 rounded-full border border-black/15 px-5" ref={keepScheduledRef} type="button">{t("flightManagement.keepScheduled")}</button></DialogClose><button className="min-h-11 rounded-full bg-red-800 px-5 font-semibold text-white" disabled={saving} onClick={() => void cancel()} type="button">{saving ? t("flightManagement.cancelling") : t("flightManagement.confirmCancel")}</button></DialogFooter></DialogContent></Dialog> : null}
  </section>;
};

export { FlightEditor };
