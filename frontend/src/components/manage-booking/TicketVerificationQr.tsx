"use client";

import { QRCodeSVG } from "qrcode.react";
import { buildTicketVerificationUrl } from "@/components/booking/ticket/ticketClient";
import { useLanguage } from "@/i18n/LanguageProvider";

export function TicketVerificationQr({ token }: { token: string }) {
  const { t } = useLanguage();
  const url = buildTicketVerificationUrl(token);
  return (
    <section className="mt-6 rounded-control border border-border p-5 text-center">
      <h3 className="text-label text-brand">{t("ticket.qr.heading")}</h3>
      <p className="mt-2 text-sm text-muted-foreground">{t("ticket.qr.scanInstruction")}</p>
      <div aria-label={t("ticket.qr.ariaLabel")} className="mx-auto mt-5 flex size-48 items-center justify-center rounded-2xl bg-white p-3" data-testid="ticket-qr-container" data-verification-url={url} role="img">
        <QRCodeSVG bgColor="#FFFFFF" fgColor="#000000" level="M" size={168} title={t("ticket.qr.ariaLabel")} value={url} />
      </div>
      <p className="mt-4 text-xs text-muted-foreground">{t("ticket.qr.secureNotice")}</p>
    </section>
  );
}
