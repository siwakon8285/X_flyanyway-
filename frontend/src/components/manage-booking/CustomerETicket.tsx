"use client";

import { ArrowRight, CheckCircle2, CircleX, Clock3, Printer } from "lucide-react";
import type { ReactNode } from "react";

import { cabinLabelKeys } from "@/components/booking/cabin/cabinPresentation";
import type { ManageBookingDetails } from "@/components/manage-booking/manageBookingTypes";
import { TicketVerificationQr } from "@/components/manage-booking/TicketVerificationQr";
import { Button } from "@/components/ui/Button";
import { formatDate } from "@/i18n/formatters";
import { useLanguage } from "@/i18n/LanguageProvider";
import { cn } from "@/lib/utils/cn";

type CustomerETicketProps = {
  booking: ManageBookingDetails;
  onPrint: () => void;
};

type TicketFieldProps = {
  children: ReactNode;
  label: string;
  labelClassName?: string;
  testId?: string;
  valueClassName?: string;
};

const displayValue = (value: string | null | undefined, fallback: string) => {
  const normalized = value?.trim();
  return normalized || fallback;
};

const TicketField = ({
  children,
  label,
  labelClassName,
  testId,
  valueClassName,
}: TicketFieldProps) => (
  <div className="min-w-0">
    <dt className={cn("text-label text-black/55 print:text-neutral-600", labelClassName)}>{label}</dt>
    <dd
      className={cn(
        "mt-2 min-w-0 break-words font-medium text-[#151515] print:text-black",
        valueClassName,
      )}
      data-testid={testId}
    >
      {children}
    </dd>
  </div>
);

const CustomerETicket = ({ booking, onPrint }: CustomerETicketProps) => {
  const { locale, t } = useLanguage();
  const cabinKey = cabinLabelKeys[booking.journey.cabin as keyof typeof cabinLabelKeys];
  const cabin = cabinKey
    ? t(cabinKey)
    : displayValue(booking.journey.cabin, t("ticket.notAvailable"));
  const departureTime = booking.journey.departureTime?.trim();
  const arrivalDate = booking.journey.arrivalDate?.trim();
  const arrivalTime = booking.journey.arrivalTime?.trim();
  const hasMultipleSeatRecords = booking.passengers.length > 1 || booking.seats.length > 1;
  const hasSeats = booking.seats.length > 0;
  const ticketStatusLabel = t(
    booking.ticket.status === "ISSUED"
      ? "ticket.status.issued"
      : "ticket.status.cancelled",
  );

  return (
    <article
      aria-labelledby="customer-e-ticket-heading"
      className="overflow-hidden rounded-[1.25rem] border border-brand/45 bg-[#fcfbf7] text-[#151515] shadow-[0_22px_70px_rgb(0_0_0/0.18)] print:break-inside-avoid print:border-neutral-800 print:bg-white print:shadow-none"
      data-ticket-content="true"
      data-testid="customer-e-ticket"
    >
      <div aria-hidden="true" className="h-2 bg-brand print:bg-black" />

      <div className="grid grid-cols-1 md:grid-cols-[minmax(0,1fr)_17rem]" data-testid="customer-e-ticket-layout">
        <div className="min-w-0 p-6 sm:p-8 lg:p-10">
          <header className="flex items-start justify-between gap-5 border-b border-black/10 pb-7 print:border-neutral-300">
            <div className="min-w-0">
              <p className="font-display text-lg font-black tracking-[0.2em] text-[#151515]">
                X-FLY ANYWAY
              </p>
              <h2 className="mt-5 text-3xl font-semibold tracking-[-0.04em] sm:text-4xl" id="customer-e-ticket-heading">
                {t("ticket.eTicket")}
              </h2>
              <p className="mt-2 max-w-md text-sm leading-6 text-black/60 print:text-neutral-700">
                {t("ticket.eTicketSubtitle")}
              </p>
            </div>
            <div
              className={cn(
                "inline-flex shrink-0 items-center gap-2 rounded-full border px-3 py-1.5 text-xs font-bold uppercase tracking-[0.08em] print:text-black",
                booking.ticket.status === "ISSUED"
                  ? "border-brand bg-brand text-brand-foreground print:border-black print:bg-white"
                  : "border-destructive/50 bg-destructive/10 text-destructive print:border-black print:bg-white",
              )}
              data-testid="e-ticket-status"
            >
              {booking.ticket.status === "ISSUED" ? (
                <CheckCircle2 aria-hidden="true" className="size-4" />
              ) : (
                <CircleX aria-hidden="true" className="size-4" />
              )}
              <span>{ticketStatusLabel}</span>
            </div>
          </header>

          <section aria-labelledby="e-ticket-route-heading" className="pt-8" data-testid="e-ticket-route">
            <h3 className="sr-only" id="e-ticket-route-heading">
              {t("ticket.route")}
            </h3>
            <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-end gap-3 sm:gap-6">
              <div className="min-w-0">
                <p className="text-label text-black/55 print:text-neutral-600">{t("ticket.departure")}</p>
                <p className="mt-2 truncate font-display text-4xl font-black tracking-[-0.04em] sm:text-6xl" data-testid="e-ticket-origin">
                  {displayValue(booking.journey.originCode, t("ticket.notAvailable"))}
                </p>
                {departureTime ? (
                  <p className="mt-3 flex items-center gap-2 font-mono text-base font-bold text-[#151515] print:text-black" data-testid="e-ticket-departure-time">
                    <Clock3 aria-hidden="true" className="size-4 text-brand print:text-black" />
                    {departureTime}
                  </p>
                ) : null}
              </div>

              <div aria-hidden="true" className="mb-3 flex items-center gap-2 text-brand print:text-black">
                <span className="hidden h-px w-8 bg-black/20 sm:block" />
                <ArrowRight className="size-6" />
                <span className="hidden h-px w-8 bg-black/20 sm:block" />
              </div>

              <div className="min-w-0 text-right">
                <p className="text-label text-black/55 print:text-neutral-600">{t("ticket.arrival")}</p>
                <p className="mt-2 truncate font-display text-4xl font-black tracking-[-0.04em] sm:text-6xl" data-testid="e-ticket-destination">
                  {displayValue(booking.journey.destinationCode, t("ticket.notAvailable"))}
                </p>
                {arrivalTime ? (
                  <p className="mt-3 font-mono text-base font-bold text-[#151515] print:text-black" data-testid="e-ticket-arrival-time">
                    {arrivalTime}
                  </p>
                ) : null}
              </div>
            </div>
          </section>

          <dl className="mt-9 grid gap-x-6 gap-y-7 border-t border-black/10 pt-7 sm:grid-cols-2 xl:grid-cols-4 print:border-neutral-300">
            <div className="min-w-0 sm:col-span-2 xl:col-span-2">
              <dt className="text-label text-black/55 print:text-neutral-600">{t("ticket.passengers")}</dt>
              <dd className="mt-2 min-w-0" data-testid="e-ticket-passengers">
                <ul className="space-y-1 break-words text-base font-semibold text-[#151515] print:text-black">
                  {booking.passengers.length > 0 ? (
                    booking.passengers.map((passenger) => (
                      <li key={passenger.ordinal}>
                        {displayValue(passenger.displayName, t("ticket.notAvailable"))}
                      </li>
                    ))
                  ) : (
                    <li>{t("ticket.notAvailable")}</li>
                  )}
                </ul>
              </dd>
            </div>

            <TicketField label={t("ticket.flight")} testId="e-ticket-flight-number">
              {displayValue(booking.journey.flightNumber, t("ticket.notAvailable"))}
            </TicketField>

            <TicketField label={t("ticket.date")} testId="e-ticket-departure-date">
              {formatDate(booking.journey.departureDate, locale)}
            </TicketField>

            {departureTime ? (
              <TicketField label={t("ticket.departure")} testId="e-ticket-departure-time-field">
                {departureTime}
              </TicketField>
            ) : null}

            <TicketField label={t("ticket.cabin")} testId="e-ticket-cabin">
              {cabin}
            </TicketField>

            <TicketField label={t("ticket.ticketNumber")} testId="e-ticket-ticket-number" valueClassName="font-mono text-sm font-bold tracking-[0.04em]">
              {displayValue(booking.ticket.ticketNumber, t("ticket.notAvailable"))}
            </TicketField>

            {arrivalDate ? (
              <TicketField label={t("ticket.arrival")} testId="e-ticket-arrival-date">
                {formatDate(arrivalDate, locale)}
              </TicketField>
            ) : null}
          </dl>
        </div>

        <aside
          aria-label={t("ticket.qr.heading")}
          className="flex min-w-0 flex-col border-t border-dashed border-black/25 bg-[#111111] p-6 text-white md:border-l md:border-t-0 sm:p-8 print:border-black print:bg-white print:text-black"
        >
          <div className="flex items-center justify-between gap-3 border-b border-white/15 pb-5 print:border-neutral-300">
            <span className="font-display text-sm font-black tracking-[0.18em]">X-FLY</span>
            <span className="text-label text-white/60 print:text-neutral-600">{t("ticket.eTicket")}</span>
          </div>

          <div className="mt-6">
            <TicketVerificationQr
              className="mt-0 border-0 p-0"
              supportingTextClassName="text-white/70"
              token={booking.qrToken}
            />
          </div>

          <dl className="mt-7 grid gap-6 border-t border-white/15 pt-6 print:border-neutral-300">
            <TicketField
              label={t("ticket.seats")}
              labelClassName="text-white/70"
              testId="e-ticket-seats"
              valueClassName="flex flex-wrap gap-2"
            >
              {hasSeats ? (
                booking.seats.map((seat) => (
                  <span
                    className="inline-flex min-w-14 items-center justify-center rounded-control border border-brand bg-brand px-3 py-2 font-mono text-xl font-black text-brand-foreground print:border-black print:bg-white print:text-black"
                    key={seat}
                  >
                    {displayValue(seat, t("ticket.notAvailable"))}
                  </span>
                ))
              ) : (
                t("ticket.notAvailable")
              )}
            </TicketField>

            <TicketField
              label={t("ticket.bookingReference")}
              labelClassName="text-white/70"
              testId="e-ticket-booking-reference"
              valueClassName="font-mono text-lg font-black tracking-[0.12em] text-white print:text-black"
            >
              {displayValue(booking.bookingReference, t("ticket.notAvailable"))}
            </TicketField>

            <TicketField
              label={t("ticket.flight")}
              labelClassName="text-white/70"
              valueClassName="font-mono text-base font-bold text-white print:text-black"
            >
              {displayValue(booking.journey.flightNumber, t("ticket.notAvailable"))}
            </TicketField>
          </dl>

          {hasMultipleSeatRecords ? (
            <p className="mt-7 text-xs leading-5 text-white/60 print:text-neutral-700">
              {t("manageBooking.seatsSeparate")}
            </p>
          ) : null}
        </aside>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-4 border-t border-black/10 px-6 py-5 sm:px-8 lg:px-10 print:hidden" data-testid="e-ticket-print-action">
        <p className="text-sm text-black/60">{t("ticket.qr.secureNotice")}</p>
        <Button className="gap-2 border-black/25 text-[#151515] hover:border-black/50 hover:bg-black/5 focus-visible:ring-offset-[#fcfbf7]" onClick={onPrint} type="button" variant="outline">
          <Printer aria-hidden="true" className="size-4" />
          {t("ticket.actions.printETicket")}
        </Button>
      </div>
    </article>
  );
};

export { CustomerETicket };
