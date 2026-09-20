"use client";

import { ArrowRight } from "lucide-react";
import { QRCodeSVG } from "qrcode.react";

import { buildBoardingPassVerificationUrl } from "@/components/boarding-pass/boardingPassClient";
import { cabinKey } from "@/components/admin/tickets/ticketOperationsPresentation";
import { formatStaffDateTime } from "@/i18n/formatters";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { BoardingPassDocument } from "@/lib/admin/ticketOperationsTypes";

import "./boardingPass.css";

const Field = ({ label, value, className = "" }: { label: string; value: string; className?: string }) => (
  <div className={`xbp-field${className ? ` ${className}` : ""}`}>
    <span>{label}</span>
    <strong>{value}</strong>
  </div>
);

export function BoardingPassView({ document }: { document: BoardingPassDocument }) {
  const { locale, t } = useLanguage();
  const statusLabel = document.validForTravel
    ? t("boardingPass.validForTravel")
    : t("boardingPass.notValidForTravel");

  return (
    <section className="xbp-print-region" aria-labelledby="boarding-pass-title">
      <div aria-hidden="true" className="xbp-brand-bar" />
      <div className="xbp-document">
        <div className="xbp-body">
          <header className="xbp-header">
            <div>
              <p className="xbp-eyebrow">X-FLY ANYWAY</p>
              <h2 id="boarding-pass-title">{t("boardingPass.boardingPass")}</h2>
            </div>
            <span className={`xbp-status ${document.validForTravel ? "is-valid" : "is-invalid"}`}>
              {statusLabel}
            </span>
          </header>

          <section aria-label={t("boardingPass.fromTo")} className="xbp-route">
            <div className="xbp-route-heading">{t("boardingPass.fromTo")}</div>
            <div className="xbp-route-points">
              <strong>{document.originCode}</strong>
              <div aria-hidden="true" className="xbp-route-connector">
                <span />
                <ArrowRight />
                <span />
              </div>
              <strong>{document.destinationCode}</strong>
            </div>
          </section>

          <div className="xbp-flight-row">
            <Field className="xbp-flight-field" label={t("boardingPass.flight")} value={document.flightNumber} />
            <Field
              className="xbp-departure-field"
              label={t("boardingPass.departure")}
              value={`${formatStaffDateTime(document.departureAt, locale, document.originTimeZone)} · ${document.originTimeZone}`}
            />
          </div>

          <div className="xbp-passenger-row">
            <Field className="xbp-passenger-field" label={t("boardingPass.passenger")} value={document.passengerName} />
            <Field className="xbp-seat-field" label={t("boardingPass.seat")} value={document.seat} />
            <Field
              className="xbp-cabin-field"
              label={t("boardingPass.cabin")}
              value={t(cabinKey(document.cabin as "business" | "first" | "economy" | "premium-economy"))}
            />
          </div>

          <div className="xbp-details">
            <Field label={t("boardingPass.bookingReference")} value={document.bookingReference} />
            <Field label={t("boardingPass.ticketNumber")} value={document.ticketNumber} />
            <Field label={t("boardingPass.reference")} value={document.boardingPassId} />
            <Field label={t("boardingPass.checkedInAt")} value={formatStaffDateTime(document.checkedInAt, locale)} />
            <Field label={t("boardingPass.issuedAt")} value={formatStaffDateTime(document.issuedAt, locale)} />
          </div>

          <p className="xbp-document-note">{t("boardingPass.demoNotice")}</p>
        </div>

        <aside aria-label={t("boardingPass.qrLabel")} className="xbp-stub">
          <div className="xbp-stub-header">
            <span className="xbp-stub-brand">X-FLY</span>
            <span>{t("boardingPass.boardingPass")}</span>
          </div>
          <div className="xbp-qr">
            <QRCodeSVG
              value={buildBoardingPassVerificationUrl(document.qrToken)}
              size={184}
              title={t("boardingPass.qrLabel")}
            />
          </div>
          <p className="xbp-qr-copy">{t("boardingPass.qrContext")}</p>
        </aside>
      </div>
    </section>
  );
}
