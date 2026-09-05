"use client";

import { ArrowLeft, ArrowRight, CheckCircle2, Plane, TicketCheck, Users } from "lucide-react";
import Link from "next/link";
import { useEffect, useState } from "react";
import { BookingApiError } from "@/components/booking/api/bookingApiClient";
import { cabinLabelKeys } from "@/components/booking/cabin/cabinPresentation";
import { productLabelKey } from "@/components/booking/extras/extrasPresentation";
import { getCurrentManageBooking } from "@/components/manage-booking/manageBookingClient";
import type { ManageBookingDetails } from "@/components/manage-booking/manageBookingTypes";
import { TicketVerificationQr } from "@/components/manage-booking/TicketVerificationQr";
import { Badge } from "@/components/ui/Badge";
import { buttonVariants } from "@/components/ui/Button";
import { Card } from "@/components/ui/Card";
import { Container } from "@/components/layout/Container";
import { useLanguage } from "@/i18n/LanguageProvider";
import { formatDate, formatPrice } from "@/i18n/formatters";
import type { TranslationKey } from "@/i18n/types";

const statusKeys = {
  booking: { CONFIRMED: "manageBooking.status.confirmed", CANCELLED: "manageBooking.status.cancelled" },
  payment: {
    CREATED: "payment.status.created", PROCESSING: "payment.status.processing",
    AWAITING_PAYMENT: "payment.status.awaiting", SUCCEEDED: "payment.status.succeeded",
    FAILED: "payment.status.failed", CANCELLED: "payment.status.cancelled",
  },
  ticket: { ISSUED: "ticket.status.issued", CANCELLED: "ticket.status.cancelled" },
} as const satisfies Record<string, Record<string, TranslationKey>>;


const ManageBookingDetailsPage = () => {
  const { locale, t } = useLanguage();
  const [booking, setBooking] = useState<ManageBookingDetails | null>(null);
  const [failure, setFailure] = useState<"authorization" | "unavailable" | null>(null);
  useEffect(() => {
    let active = true;
    getCurrentManageBooking()
      .then((current) => { if (active) setBooking(current); })
      .catch((error: unknown) => {
        if (active) setFailure(error instanceof BookingApiError && error.status === 401 ? "authorization" : "unavailable");
      });
    return () => { active = false; };
  }, []);
  if (!booking) {
    return <main className="min-h-screen bg-background pb-24 pt-28"><Container>
      {failure ? <section role="alert"><h1 className="text-display-sm">{t("manageBooking.yourBooking")}</h1><p className="mt-4">{t(failure === "authorization" ? "manageBooking.error.authorization" : "manageBooking.error.unavailable")}</p><Link className={buttonVariants({ variant: "primary" })} href="/manage-booking">{t("manageBooking.find")}</Link></section>
        : <div aria-label={t("manageBooking.loading")} role="status" />}
    </Container></main>;
  }
    const money = formatPrice(booking.payment.amount.amount, locale, booking.payment.amount.currencyCode);
    const cutoff = booking.cancellation.cutoffAt
      ? new Intl.DateTimeFormat(locale === "th" ? "th-TH" : "en-GB", { dateStyle: "medium", timeStyle: "short", timeZone: booking.journey.departureTimeZone ?? undefined }).format(new Date(booking.cancellation.cutoffAt))
      : null;
    return (
      <main className="min-h-screen bg-background pb-24 pt-28">
        <Container>
          <Link
            className="inline-flex min-h-11 items-center gap-2 text-sm text-muted-foreground transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus motion-reduce:transition-none"
            href="/manage-booking"
          >
            <ArrowLeft aria-hidden="true" className="size-4" />
            {t("manageBooking.findAnother")}
          </Link>
          <header className="border-b border-border pb-8">
            <p className="text-label text-brand">{t("manageBooking.eyebrow")}</p>
            <div className="mt-3 flex flex-col justify-between gap-5 md:flex-row md:items-end">
              <div><h1 className="text-display-sm">{t("manageBooking.yourBooking")}</h1><p className="mt-3 font-mono text-xl tracking-[0.18em] text-muted-foreground">{booking.bookingReference}</p></div>
              <Badge>{t(statusKeys.booking[booking.status])}</Badge>
            </div>
          </header>

          <section aria-labelledby="journey-heading" className="py-10">
            <div className="flex items-center gap-3"><Plane aria-hidden="true" className="text-brand" /><h2 className="text-h2" id="journey-heading">{t("manageBooking.flight")}</h2></div>
            <Card className="mt-6 p-6 sm:p-8">
              <div className="grid gap-7 md:grid-cols-[1fr_auto_1fr] md:items-center">
                <div><p className="text-display-sm">{booking.journey.originCode}</p><p className="mt-2 text-muted-foreground">{formatDate(booking.journey.departureDate, locale)} · {booking.journey.departureTime ?? "—"}</p></div>
                <ArrowRight aria-hidden="true" className="text-brand" />
                <div className="md:text-right"><p className="text-display-sm">{booking.journey.destinationCode}</p><p className="mt-2 text-muted-foreground">{formatDate(booking.journey.arrivalDate ?? booking.journey.departureDate, locale)} · {booking.journey.arrivalTime ?? "—"}</p></div>
              </div>
              <div className="mt-7 grid gap-4 border-t border-border pt-6 sm:grid-cols-2"><p><span className="text-muted-foreground">{t("manageBooking.flightNumber")}</span><br /><strong>{booking.journey.flightNumber}</strong></p><p><span className="text-muted-foreground">{t("manageBooking.cabin")}</span><br /><strong>{t(cabinLabelKeys[booking.journey.cabin as keyof typeof cabinLabelKeys] ?? "cabin.economy")}</strong></p></div>
            </Card>
          </section>

          <div className="grid gap-6 lg:grid-cols-2">
            <Card className="p-6"><h2 className="flex items-center gap-3 text-h3"><Users aria-hidden="true" className="text-brand" />{t("manageBooking.passengers")}</h2><ul className="mt-5 space-y-4">{booking.passengers.map((passenger) => <li className="flex justify-between gap-4 border-b border-border pb-4 last:border-0" key={passenger.ordinal}><span>{passenger.displayName}</span><span className="text-sm text-muted-foreground">{t(passenger.travelDocumentStatus === "COMPLETE" ? "manageBooking.documentComplete" : "manageBooking.documentIncomplete")}</span></li>)}</ul></Card>
            <Card className="p-6"><h2 className="text-h3">{t("manageBooking.seats")}</h2><div className="mt-5 flex flex-wrap gap-3">{booking.seats.map((seat) => <Badge key={seat}>{seat}</Badge>)}</div><p className="mt-4 text-sm text-muted-foreground">{t(booking.status === "CANCELLED" ? "manageBooking.cancelledSeatsNotice" : "manageBooking.seatsSeparate")}</p></Card>
            <Card className="p-6"><h2 className="text-h3">{t("manageBooking.extras")}</h2>{booking.extras.length ? <ul className="mt-5 space-y-3">{booking.extras.map((extra) => <li key={`${extra.passengerOrdinal}-${extra.productCode}`}>{t(productLabelKey(extra.productCode))} <span className="text-muted-foreground">· {t("manageBooking.passengerOrdinal", { ordinal: extra.passengerOrdinal })}</span></li>)}</ul> : <p className="mt-5 text-muted-foreground">{t("manageBooking.noExtras")}</p>}</Card>
            <Card className="p-6"><h2 className="text-h3">{t("manageBooking.payment")}</h2><p className="mt-5 text-2xl font-semibold">{money}</p><p className="mt-2 text-muted-foreground">{t(statusKeys.payment[booking.payment.status])}</p></Card>
            <Card className="p-6"><h2 className="flex items-center gap-3 text-h3"><TicketCheck aria-hidden="true" className="text-brand" />{t("manageBooking.ticket")}</h2><p className="mt-5 font-mono tracking-wider">{booking.ticket.ticketNumber}</p><p className="mt-2 text-muted-foreground">{t(statusKeys.ticket[booking.ticket.status])}</p><TicketVerificationQr token={booking.qrToken} /></Card>
            <Card className="p-6"><h2 className="flex items-center gap-3 text-h3"><CheckCircle2 aria-hidden="true" className="text-brand" />{t("manageBooking.cancellation")}</h2><p className="mt-5 font-medium">{t(booking.cancellation.eligibility === "ELIGIBLE" ? "manageBooking.cancellationEligible" : "manageBooking.cancellationUnavailable")}</p>{booking.cancellation.eligibility === "ELIGIBLE" && cutoff ? <p className="mt-2 text-sm text-muted-foreground">{t("manageBooking.freeCancellationUntil", { date: cutoff })}</p> : null}<p className="mt-4 text-sm text-muted-foreground">{t("manageBooking.cancellationFuture")}</p></Card>
          </div>
        </Container>
      </main>
    );
};

export { ManageBookingDetailsPage };
