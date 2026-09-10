"use client";

import { AlertTriangle, ArrowLeft, Plane, ShieldCheck } from "lucide-react";
import Link from "next/link";
import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";

import { cabinKeys, formatBookingDateTime, isLegacyCabin, statusKeys } from "./bookingPresentation";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/Dialog";
import { useLanguage } from "@/i18n/LanguageProvider";
import { formatStaffDate } from "@/i18n/formatters";
import type { TranslationKey } from "@/i18n/types";
import type { BookingDetail } from "@/lib/admin/bookingTypes";
import "./bookingOperations.css";

const DetailSection = ({ index, title, children }: { index: string; title: string; children: ReactNode }) => <section className="xbo-detail-section"><h2><span>{index} /</span>{title}</h2>{children}</section>;
const Fact = ({ label, children }: { label: string; children: ReactNode }) => <div className="xbo-fact"><span>{label}</span><strong>{children}</strong></div>;

const BookingDetailWorkspace = ({ bookingReference, canManage }: { bookingReference: string; canManage: boolean }) => {
  const { locale, t } = useLanguage();
  const [detail, setDetail] = useState<BookingDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [errorStatus, setErrorStatus] = useState<number | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  const [mutationMessage, setMutationMessage] = useState<"success" | "failed" | null>(null);
  const cancelTriggerRef = useRef<HTMLButtonElement | null>(null);
  const keepBookingRef = useRef<HTMLButtonElement | null>(null);

  const requestDetail = useCallback(async (signal?: AbortSignal) => {
    const response = await fetch(`/admin/api/bookings/${encodeURIComponent(bookingReference)}`, { credentials: "same-origin", cache: "no-store", signal });
    if (!response.ok) throw response.status;
    return response.json() as Promise<BookingDetail>;
  }, [bookingReference]);
  useEffect(() => {
    const controller = new AbortController();
    requestDetail(controller.signal)
      .then(setDetail)
      .catch((cause) => { if (!(cause instanceof DOMException && cause.name === "AbortError")) setErrorStatus(typeof cause === "number" ? cause : 503); })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, [requestDetail]);

  const refresh = async () => {
    setLoading(true); setErrorStatus(null);
    try { setDetail(await requestDetail()); }
    catch (cause) { setErrorStatus(typeof cause === "number" ? cause : 503); }
    finally { setLoading(false); }
  };

  const dateTime = (value: string | null) => value ? formatBookingDateTime(value, locale) : t("bookingManagement.unavailable");
  const money = (amount: number, currency: string) => new Intl.NumberFormat(locale === "th" ? "th-TH" : "en-GB", { style: "currency", currency, maximumFractionDigits: 0 }).format(amount);
  const passengerType = (value: BookingDetail["passengers"][number]["passengerType"]): TranslationKey => `bookingManagement.passenger.${value.toLowerCase() as "adult" | "child" | "infant"}`;
  const gender = (value: BookingDetail["passengers"][number]["gender"]): TranslationKey => `bookingManagement.passenger.${value === "UNSPECIFIED" ? "unspecified" : value.toLowerCase() as "male" | "female"}`;
  const cancel = async () => {
    setCancelling(true); setMutationMessage(null);
    try {
      const response = await fetch(`/admin/api/bookings/${encodeURIComponent(bookingReference)}/cancel`, { method: "POST", credentials: "same-origin", cache: "no-store", headers: { "x-x-fly-csrf": "1" } });
      if (!response.ok) { setMutationMessage("failed"); setDialogOpen(false); await refresh(); return; }
      setDetail(await response.json() as BookingDetail); setMutationMessage("success"); setDialogOpen(false);
    } catch { setMutationMessage("failed"); setDialogOpen(false); await refresh(); }
    finally { setCancelling(false); }
  };

  if (loading && !detail) return <p aria-live="polite" className="xbo-message">{t("bookingManagement.loading")}</p>;
  if (errorStatus || !detail) return <div className="xbo-error" role="alert"><p>{errorStatus === 404 ? t("bookingManagement.notFound") : errorStatus === 401 ? t("bookingManagement.sessionExpired") : errorStatus === 403 ? t("bookingManagement.forbidden") : t("bookingManagement.error")}</p><button onClick={() => void refresh()} type="button">{t("bookingManagement.retry")}</button></div>;

  const paymentMethod = detail.payment.method === "CARD" ? t("bookingManagement.paymentMethod.card") : t("bookingManagement.paymentMethod.bitcoin");
  const provider = detail.payment.provider === "STRIPE" ? t("bookingManagement.paymentMethod.stripe") : t("bookingManagement.paymentMethod.mockBitcoin");
  const canCancel = canManage && detail.bookingStatus === "CONFIRMED" && detail.cancellation.eligibility === "ELIGIBLE";
  return <article className="xbo-shell xbo-detail" aria-labelledby="booking-detail-title">
    <Link className="xbo-back" href="/admin/bookings"><ArrowLeft aria-hidden="true" />{t("bookingManagement.back")}</Link>
    <header className="xbo-detail-header"><div><p className="xbo-kicker">{t("bookingManagement.terminal")}</p><h1 id="booking-detail-title">{detail.bookingReference}</h1><div className="xbo-status-pair"><span>{t("bookingManagement.bookingStatus")} <b className="xbo-chip">{t(statusKeys[detail.bookingStatus])}</b></span><span>{t("bookingManagement.flightStatus")} <b className={`xbo-chip ${detail.journey.flightStatus === "CANCELLED" ? "is-alert" : ""}`}>{t(statusKeys[detail.journey.flightStatus])}</b></span></div></div><div className="xbo-route-mark"><span>{detail.journey.flightNumber}</span><strong>{detail.journey.originCode} → {detail.journey.destinationCode}</strong><time dateTime={detail.journey.travelDate}>{formatStaffDate(detail.journey.travelDate, locale)}</time></div></header>
    {detail.journey.flightStatus === "CANCELLED" ? <p className="xbo-flight-warning" role="status"><AlertTriangle aria-hidden="true" />{t("bookingManagement.flightCancelledNotice")}</p> : null}
    {mutationMessage ? <p aria-live="polite" className={`xbo-mutation-message ${mutationMessage === "failed" ? "is-error" : ""}`}>{t(mutationMessage === "success" ? "bookingManagement.cancelSuccess" : "bookingManagement.cancelFailed")}</p> : null}

    <div className="xbo-detail-grid">
      <DetailSection index="01" title={t("bookingManagement.booking")}><div className="xbo-facts"><Fact label={t("bookingManagement.bookingReference")}>{detail.bookingReference}</Fact><Fact label={t("bookingManagement.bookingStatus")}>{t(statusKeys[detail.bookingStatus])}</Fact><Fact label={t("bookingManagement.bookedAt")}>{dateTime(detail.createdAt)}</Fact></div></DetailSection>
      <DetailSection index="02" title={t("bookingManagement.journey")}><div className="xbo-facts"><Fact label={t("bookingManagement.flightNumber")}>{detail.journey.flightNumber}</Fact><Fact label={t("bookingManagement.route")}>{detail.journey.originCode} → {detail.journey.destinationCode}</Fact><Fact label={t("bookingManagement.departure")}>{formatStaffDate(detail.journey.travelDate, locale)} · {detail.journey.departureTime ?? t("bookingManagement.unavailable")} · {detail.journey.originTimeZone ?? t("bookingManagement.unavailable")}</Fact><Fact label={t("bookingManagement.aircraft")}>{detail.journey.aircraftCode}</Fact><Fact label={t("bookingManagement.cabin")}>{t(cabinKeys[detail.journey.cabin])}{isLegacyCabin(detail.journey.cabin) ? <small>{t("bookingManagement.historical")}</small> : null}</Fact><Fact label={t("bookingManagement.flightStatus")}>{t(statusKeys[detail.journey.flightStatus])}</Fact></div></DetailSection>
      <DetailSection index="03" title={t("bookingManagement.passengers")}><div className="xbo-passenger-list">{detail.passengers.map((passenger) => <div key={passenger.ordinal}><span>{String(passenger.ordinal).padStart(2,"0")}</span><strong>{passenger.displayName}</strong><small>{t(passengerType(passenger.passengerType))} · {t(gender(passenger.gender))}</small></div>)}</div><div className="xbo-seat-strip"><span>{t("bookingManagement.seats")}</span><strong>{detail.seats.join(" · ")}</strong><small>{t("bookingManagement.seatNote")}</small></div></DetailSection>
      <DetailSection index="04" title={t("bookingManagement.contact")}><div className="xbo-facts"><Fact label={t("bookingManagement.phone")}>{detail.contact ? `${detail.contact.phoneCountryCode} ${detail.contact.phoneNumber}` : t("bookingManagement.noContact")}</Fact></div></DetailSection>
      <DetailSection index="05" title={t("bookingManagement.payment")}><div className="xbo-facts"><Fact label={t("bookingManagement.method")}>{paymentMethod}</Fact><Fact label={t("bookingManagement.provider")}>{provider}</Fact><Fact label={t("bookingManagement.amountPaid")}>{money(detail.payment.amount.amount, detail.payment.amount.currencyCode)}</Fact><Fact label={t("bookingManagement.statusLabel")}>{t(statusKeys[detail.payment.status])}</Fact><Fact label={t("bookingManagement.succeededAt")}>{dateTime(detail.payment.succeededAt)}</Fact></div></DetailSection>
      <DetailSection index="06" title={t("bookingManagement.ticket")}><div className="xbo-facts"><Fact label={t("bookingManagement.ticketNumber")}>{detail.ticket.ticketNumber}</Fact><Fact label={t("bookingManagement.statusLabel")}>{t(statusKeys[detail.ticket.status])}</Fact><Fact label={t("bookingManagement.issuedAt")}>{dateTime(detail.ticket.issuedAt)}</Fact><Fact label={t("bookingManagement.cancelledAt")}>{dateTime(detail.ticket.cancelledAt)}</Fact></div></DetailSection>
      <DetailSection index="07" title={t("bookingManagement.cancellation")}><div className="xbo-facts"><Fact label={t("bookingManagement.eligibility")}>{t(detail.cancellation.eligibility === "ELIGIBLE" ? "bookingManagement.status.eligible" : "bookingManagement.status.ineligible")}</Fact><Fact label={t("bookingManagement.cutoff")}>{dateTime(detail.cancellation.cutoffAt)}</Fact><Fact label={t("bookingManagement.refundStatus")}>{detail.cancellation.refundStatus ? t(statusKeys[detail.cancellation.refundStatus]) : t("bookingManagement.unavailable")}</Fact><Fact label={t("bookingManagement.refundAmount")}>{detail.cancellation.refundAmount ? money(detail.cancellation.refundAmount.amount, detail.cancellation.refundAmount.currencyCode) : t("bookingManagement.unavailable")}</Fact><Fact label={t("bookingManagement.cancelledAt")}>{dateTime(detail.cancellation.cancelledAt)}</Fact><Fact label={t("bookingManagement.refundedAt")}>{dateTime(detail.cancellation.refundedAt)}</Fact></div>{canCancel ? <button className="xbo-danger" onClick={(event) => { cancelTriggerRef.current = event.currentTarget; setDialogOpen(true); }} type="button">{t("bookingManagement.cancelBooking")}</button> : null}</DetailSection>
      <DetailSection index="08" title={t("bookingManagement.audit")}>{detail.audit.length ? <ol className="xbo-audit">{detail.audit.map((entry) => <li key={`${entry.createdAt}-${entry.actorEmail}`}><ShieldCheck aria-hidden="true" /><div><strong>{t("bookingManagement.auditCancelled")}</strong><span>{entry.actorEmail}</span><time dateTime={entry.createdAt}>{dateTime(entry.createdAt)}</time></div></li>)}</ol> : <p className="xbo-muted">{t("bookingManagement.noAudit")}</p>}</DetailSection>
    </div>

    <Dialog onOpenChange={setDialogOpen} open={dialogOpen}><DialogContent className="xbo-dialog" onCloseAutoFocus={(event) => { event.preventDefault(); cancelTriggerRef.current?.focus(); }} onOpenAutoFocus={(event) => { event.preventDefault(); keepBookingRef.current?.focus(); }} showCloseButton={false}><DialogTitle>{t("bookingManagement.cancelTitle", { reference: detail.bookingReference })}</DialogTitle><DialogDescription>{t("bookingManagement.cancelDescription")}</DialogDescription><div className="xbo-confirm-context"><div><Plane aria-hidden="true" /><strong>{detail.journey.flightNumber} · {detail.journey.originCode} → {detail.journey.destinationCode}</strong><span>{formatStaffDate(detail.journey.travelDate, locale)} · {detail.passengers[0]?.displayName}</span></div><Fact label={t("bookingManagement.amountPaid")}>{money(detail.payment.amount.amount, detail.payment.amount.currencyCode)}</Fact><p>{t("bookingManagement.consequence")}</p><strong>{t("bookingManagement.fullRefund")} · {t("bookingManagement.noFee")}</strong></div><div className="xbo-dialog-actions"><button disabled={cancelling} onClick={() => setDialogOpen(false)} ref={keepBookingRef} type="button">{t("bookingManagement.keep")}</button><button className="xbo-danger" disabled={cancelling} onClick={() => void cancel()} type="button">{cancelling ? t("bookingManagement.cancelling") : t("bookingManagement.confirm")}</button></div></DialogContent></Dialog>
  </article>;
};

export { BookingDetailWorkspace };
