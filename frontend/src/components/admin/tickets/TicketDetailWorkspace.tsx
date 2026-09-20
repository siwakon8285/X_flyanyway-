"use client";

import Link from "next/link";
import { ArrowLeft, Printer, ShieldCheck } from "lucide-react";
import { QRCodeSVG } from "qrcode.react";
import { useCallback, useEffect, useState, type ReactNode } from "react";

import { BoardingPassView } from "@/components/boarding-pass/BoardingPassView";
import { buildTicketVerificationUrl } from "@/components/booking/ticket/ticketClient";
import { Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/Dialog";
import { useLanguage } from "@/i18n/LanguageProvider";
import { formatStaffDate } from "@/i18n/formatters";
import { formatBookingDateTime } from "@/components/admin/bookings/bookingPresentation";
import { cabinKey, statusKey } from "@/components/admin/tickets/ticketOperationsPresentation";
import type { BoardingPassDocument, CheckInUnavailableReason, PrintableTicketOperationsDetail, TicketOperationsDetail } from "@/lib/admin/ticketOperationsTypes";

import "./ticketOperations.css";

const Section = ({ n, title, children }: { n: string; title: string; children: ReactNode }) => (
  <section className="xto-detail-section">
    <h2><span>{n} /</span>{title}</h2>
    {children}
  </section>
);

const Fact = ({ label, children, className }: { label: string; children: ReactNode; className?: string }) => (
  <div className={`xto-fact${className ? ` ${className}` : ""}`}><span>{label}</span><strong>{children}</strong></div>
);

const unavailableReasonKey: Record<CheckInUnavailableReason, "boardingPass.tooEarly" | "boardingPass.alreadyDeparted" | "boardingPass.ticketCancelled" | "boardingPass.flightCancelled" | "boardingPass.noSeatAssignment" | "boardingPass.bookingInvalid"> = {
  TOO_EARLY: "boardingPass.tooEarly",
  ALREADY_DEPARTED: "boardingPass.alreadyDeparted",
  TICKET_CANCELLED: "boardingPass.ticketCancelled",
  FLIGHT_CANCELLED: "boardingPass.flightCancelled",
  NO_SEAT_ASSIGNMENT: "boardingPass.noSeatAssignment",
  BOOKING_INVALID: "boardingPass.bookingInvalid",
};

export function TicketDetailWorkspace({
  ticketNumber,
  canPrint,
  canIssue = false,
}: {
  ticketNumber: string;
  canPrint: boolean;
  canIssue?: boolean;
}) {
  const { locale, t } = useLanguage();
  const [detail, setDetail] = useState<TicketOperationsDetail | null>(null);
  const [printData, setPrintData] = useState<PrintableTicketOperationsDetail | null>(null);
  const [boardingPass, setBoardingPass] = useState<BoardingPassDocument | null>(null);
  const [boardingPassViewOpen, setBoardingPassViewOpen] = useState(false);
  const [confirming, setConfirming] = useState<TicketOperationsDetail["passengers"][number] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<number | null>(null);
  const [printing, setPrinting] = useState(false);
  const [boardingPassLoading, setBoardingPassLoading] = useState(false);
  const [printError, setPrintError] = useState(false);
  const [boardingPassError, setBoardingPassError] = useState(false);
  const [printMode, setPrintMode] = useState<"ticket" | "boarding-pass" | null>(null);

  const load = useCallback(async (signal?: AbortSignal) => {
    const response = await fetch(`/admin/api/tickets/${encodeURIComponent(ticketNumber)}`, {
      credentials: "same-origin",
      cache: "no-store",
      signal,
    });
    if (!response.ok) throw response.status;
    return response.json() as Promise<TicketOperationsDetail>;
  }, [ticketNumber]);

  useEffect(() => {
    const controller = new AbortController();
    load(controller.signal)
      .then(setDetail)
      .catch((reason) => {
        if (!(reason instanceof DOMException && reason.name === "AbortError")) setError(typeof reason === "number" ? reason : 503);
      })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, [load]);

  useEffect(() => {
    const handleAfterPrint = () => setPrintMode(null);
    window.addEventListener("afterprint", handleAfterPrint);
    return () => window.removeEventListener("afterprint", handleAfterPrint);
  }, []);

  const fetchBoardingPass = async (ordinal: number, shouldPrint: boolean) => {
    setBoardingPassLoading(true);
    setBoardingPassError(false);
    try {
      const response = await fetch(`/admin/api/tickets/${encodeURIComponent(ticketNumber)}/passengers/${ordinal}/boarding-pass`, {
        credentials: "same-origin",
        cache: "no-store",
      });
      if (!response.ok) throw new Error();
      const document = await response.json() as BoardingPassDocument;
      setBoardingPass(document);
      setBoardingPassViewOpen(!shouldPrint);
      setPrintMode(shouldPrint ? "boarding-pass" : null);
      if (shouldPrint) window.setTimeout(() => window.print(), 50);
    } catch {
      setBoardingPassError(true);
    } finally {
      setBoardingPassLoading(false);
    }
  };

  const issueBoardingPass = async () => {
    if (!confirming) return;
    const ordinal = confirming.ordinal;
    setBoardingPassLoading(true);
    setBoardingPassError(false);
    try {
      const response = await fetch(`/admin/api/tickets/${encodeURIComponent(ticketNumber)}/passengers/${ordinal}/boarding-pass`, {
        method: "POST",
        credentials: "same-origin",
        cache: "no-store",
        headers: { "content-type": "application/json", "x-x-fly-csrf": "1" },
        body: "{}",
      });
      if (!response.ok) throw new Error();
      const document = await response.json() as BoardingPassDocument;
      setBoardingPass(document);
      setBoardingPassViewOpen(false);
      setConfirming(null);
      try {
        setDetail(await load());
      } catch {
        // The authoritative issuance response is still shown if the refresh is briefly unavailable.
      }
    } catch {
      setBoardingPassError(true);
    } finally {
      setBoardingPassLoading(false);
    }
  };

  const printTicket = async () => {
    setPrinting(true);
    setPrintError(false);
    try {
      const response = await fetch(`/admin/api/tickets/${encodeURIComponent(ticketNumber)}/print`, { credentials: "same-origin", cache: "no-store" });
      if (!response.ok) throw new Error();
      const data = await response.json() as PrintableTicketOperationsDetail;
      setPrintData(data);
      setPrintMode("ticket");
      window.setTimeout(() => window.print(), 50);
    } catch {
      setPrintError(true);
    } finally {
      setPrinting(false);
    }
  };

  if (loading) return <p className="xto-state" role="status">{t("ticketOperations.loading")}</p>;
  if (error || !detail) return <div className="xto-state is-error" role="alert"><p>{t(error === 404 ? "ticketOperations.notFound" : error === 401 ? "ticketOperations.sessionExpired" : error === 403 ? "ticketOperations.forbidden" : "ticketOperations.error")}</p><button onClick={() => location.reload()}>{t("ticketOperations.retry")}</button></div>;

  const fmt = (value: string | null) => value ? formatBookingDateTime(value, locale) : t("ticketOperations.unavailable");
  const cancelled = detail.ticketStatus === "CANCELLED";
  const flightCancelled = detail.journey.flightStatus === "CANCELLED";

  return (
    <article className={`xto-shell xto-detail ${cancelled ? "is-cancelled" : ""} ${printMode === "boarding-pass" ? "is-printing-boarding-pass" : ""} ${printMode === "ticket" ? "is-printing-ticket" : ""}`}>
      <Link className="xto-back" href="/admin/tickets"><ArrowLeft aria-hidden="true" />{t("ticketOperations.back")}</Link>
      <header className="xto-ticket-head"><div><p>{t("ticketOperations.terminal")}</p><h1>{detail.ticketNumber}</h1><span className={`xto-chip ${cancelled ? "is-cancelled" : ""}`}>{t(statusKey(detail.ticketStatus))}</span></div><div><strong>{detail.journey.flightNumber}</strong><span>{detail.journey.originCode} → {detail.journey.destinationCode}</span><time dateTime={detail.journey.travelDate}>{formatStaffDate(detail.journey.travelDate, locale)}</time></div></header>
      {cancelled ? <div className="xto-cancelled-banner" role="status">{t("ticketOperations.cancelledNotice")}</div> : null}
      {flightCancelled ? <aside className="xto-flight-cancelled" role="alert"><strong>{t("ticketOperations.flightCancelledTitle")}</strong><span>{t("ticketOperations.flightCancelledMessage")}</span></aside> : null}
      <div className="xto-detail-grid">
        <Section n="01" title={t("ticketOperations.ticket")}><div className="xto-facts"><Fact label={t("ticketOperations.ticketNumber")}>{detail.ticketNumber}</Fact><Fact label={t("ticketOperations.ticketStatus")}>{t(statusKey(detail.ticketStatus))}</Fact><Fact label={t("ticketOperations.issuedAt")}>{fmt(detail.issuedAt)}</Fact><Fact label={t("ticketOperations.cancelledAt")}>{fmt(detail.cancelledAt)}</Fact></div></Section>
        <Section n="02" title={t("ticketOperations.passengers")}>
          <div className="xto-passengers">
            {detail.passengers.map((passenger) => {
              const checkIn = passenger.checkIn;
              const hasBoardingPass = passenger.boardingPass !== null;
              const checkInLabel = hasBoardingPass || checkIn.status === "CHECKED_IN" ? t("boardingPass.checkedIn") : checkIn.status === "READY" ? t("boardingPass.ready") : t("boardingPass.unavailable");
              return <div className="xto-passenger-row" key={passenger.ordinal}>
                <span>{String(passenger.ordinal).padStart(2, "0")}</span>
                <strong>{passenger.displayName}</strong>
                <small>{t(`ticketOperations.passengerTypeName.${passenger.passengerType.toLowerCase() as "adult" | "child" | "infant"}`)} · {t(`ticketOperations.genderName.${passenger.gender.toLowerCase() as "male" | "female" | "unspecified"}`)}</small>
                <b>{passenger.seat ?? t("ticketOperations.noSeat")}</b>
                <div className="xto-checkin-state"><span>{t("boardingPass.checkIn")}</span><strong className={checkIn.status === "READY" ? "is-ready" : checkIn.status === "CHECKED_IN" ? "is-checked-in" : "is-unavailable"}>{checkInLabel}</strong>{checkIn.reason ? <small>{t(unavailableReasonKey[checkIn.reason])}</small> : null}</div>
                <div className="xto-passenger-actions">
                  {checkIn.status === "READY" && canIssue ? <button className="xto-boarding-action" disabled={boardingPassLoading} onClick={() => setConfirming(passenger)} type="button">{t("boardingPass.issue")}</button> : null}
                  {hasBoardingPass ? <><button className="xto-secondary-action" disabled={boardingPassLoading} onClick={() => void fetchBoardingPass(passenger.ordinal, false)} type="button">{t("boardingPass.view")}</button>{canPrint ? <button className="xto-secondary-action" disabled={boardingPassLoading} onClick={() => void fetchBoardingPass(passenger.ordinal, true)} type="button"><Printer aria-hidden="true" />{t("boardingPass.print")}</button> : null}</> : null}
                </div>
              </div>;
            })}
          </div>
          {boardingPassError ? <p className="xto-print-warning" role="alert">{t("boardingPass.issueFailed")}</p> : null}
        </Section>
        <Section n="03" title={t("ticketOperations.journey")}><div className="xto-facts"><Fact label={t("ticketOperations.flightNumber")}>{detail.journey.flightNumber}</Fact><Fact label={t("ticketOperations.route")}>{detail.journey.originCode} → {detail.journey.destinationCode}</Fact><Fact label={t("ticketOperations.departure")}>{formatStaffDate(detail.journey.travelDate, locale)} · {detail.journey.departureTime ?? t("ticketOperations.unavailable")}</Fact><Fact label={t("ticketOperations.timeZone")}>{detail.journey.originTimeZone ?? t("ticketOperations.unavailable")}</Fact><Fact label={t("ticketOperations.flightStatus")}>{t(statusKey(detail.journey.flightStatus))}</Fact></div></Section>
        <Section n="04" title={t("ticketOperations.travelProduct")}><div className="xto-facts"><Fact label={t("ticketOperations.cabin")}>{t(cabinKey(detail.journey.cabin))}</Fact><Fact label={t("ticketOperations.seat")}>{detail.passengers.map((passenger) => passenger.seat).filter(Boolean).join(" · ")}</Fact></div></Section>
        <Section n="05" title={t("ticketOperations.booking")}><div className="xto-facts"><Fact label={t("ticketOperations.bookingReference")}>{detail.bookingReference}</Fact><Fact label={t("ticketOperations.bookingStatus")}>{t(statusKey(detail.bookingStatus))}</Fact><Fact label={t("ticketOperations.paymentStatus")}>{t(statusKey(detail.paymentStatus))}</Fact><Fact label={t("ticketOperations.refundStatus")}>{detail.refundStatus ? t(statusKey(detail.refundStatus)) : t("ticketOperations.unavailable")}</Fact></div></Section>
        <Section n="06" title={t("ticketOperations.document")}><div className="xto-document"><div><ShieldCheck aria-hidden="true" /><strong>{t("ticketOperations.documentTitle")}</strong><span>{t("ticketOperations.documentCopy")}</span></div>{printData ? <div className="xto-qr" role="img" aria-label={t("ticketOperations.qrLabel")}><QRCodeSVG value={buildTicketVerificationUrl(printData.qrToken)} size={152} title={t("ticketOperations.qrLabel")} /><small>{t("ticketOperations.qrContext")}</small></div> : null}</div>{cancelled ? <p className="xto-print-warning">{t("ticketOperations.cancelledPrint")}</p> : canPrint ? <button className="xto-print" disabled={printing} onClick={() => void printTicket()}><Printer aria-hidden="true" />{printing ? t("ticketOperations.preparingPrint") : t("ticketOperations.print")}</button> : <p className="xto-print-warning">{t("ticketOperations.printDenied")}</p>}{printError ? <p className="xto-print-warning" role="alert">{t("ticketOperations.printFailed")}</p> : null}</Section>
      </div>
      {boardingPass && !boardingPassViewOpen ? <div className="xto-boarding-pass-panel"><BoardingPassView document={boardingPass} /></div> : null}
      <Dialog open={boardingPassViewOpen} onOpenChange={setBoardingPassViewOpen}>
        <DialogContent className="xto-boarding-view-dialog" showCloseButton={false}>
          <DialogHeader className="sr-only">
            <DialogTitle>{t("boardingPass.view")}</DialogTitle>
            <DialogDescription>{t("boardingPass.demoNotice")}</DialogDescription>
          </DialogHeader>
          {boardingPass ? <BoardingPassView document={boardingPass} /> : null}
          <DialogFooter className="xto-boarding-view-footer">
            <DialogClose asChild>
              <button type="button">{t("boardingPass.cancel")}</button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <Dialog open={confirming !== null} onOpenChange={(open) => { if (!open && !boardingPassLoading) setConfirming(null); }}>
        <DialogContent className="xto-checkin-dialog" onOpenAutoFocus={(event) => event.preventDefault()} showCloseButton={false}>
          <DialogHeader><DialogTitle>{t("boardingPass.confirmTitle")}</DialogTitle><DialogDescription>{t("boardingPass.confirmDescription")}</DialogDescription></DialogHeader>
          {confirming ? <div className="xto-confirm-context"><Fact className="xto-confirm-fact" label={t("boardingPass.passenger")}>{confirming.displayName}</Fact><Fact className="xto-confirm-fact" label={t("boardingPass.flight")}>{detail.journey.flightNumber}</Fact><Fact className="xto-confirm-fact" label={t("boardingPass.fromTo")}>{detail.journey.originCode} → {detail.journey.destinationCode}</Fact><Fact className="xto-confirm-fact" label={t("boardingPass.seat")}>{confirming.seat ?? t("ticketOperations.noSeat")}</Fact><Fact className="xto-confirm-fact" label={t("boardingPass.cabin")}>{t(cabinKey(detail.journey.cabin))}</Fact></div> : null}
          {boardingPassError ? <p className="xto-print-warning" role="alert">{t("boardingPass.issueFailed")}</p> : null}
          <DialogFooter><DialogClose asChild><button disabled={boardingPassLoading} type="button">{t("boardingPass.cancel")}</button></DialogClose><button className="xto-boarding-action" disabled={boardingPassLoading} onClick={() => void issueBoardingPass()} type="button">{boardingPassLoading ? t("boardingPass.issuing") : t("boardingPass.issue")}</button></DialogFooter>
        </DialogContent>
      </Dialog>
    </article>
  );
}
