"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import {
  Armchair,
  ArrowLeft,
  Check,
  CheckCircle2,
  CircleX,
  Clock,
  Copy,
  CreditCard,
  Plane,
  Printer,
  User,
} from "lucide-react";
import { useGSAP } from "@gsap/react";
import gsap from "gsap";

import {
  BookingApiError,
  getTicket,
} from "@/components/booking/ticket/ticketClient";
import type { TicketResponse } from "@/components/booking/ticket/ticketTypes";
import { Container } from "@/components/layout/Container";
import { Button, buttonVariants } from "@/components/ui/Button";
import { formatDate, formatPrice } from "@/i18n/formatters";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";
import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";

type TicketPageProps = {
  holdId: string;
  attemptId: string;
  backQuery?: string;
};

const cabinKeyMap: Record<string, TranslationKey> = {
  business: "common.cabins.business",
  economy: "common.cabins.economy",
  first: "common.cabins.first",
  "premium-economy": "common.cabins.premiumEconomy",
};

const TicketPage = ({
  holdId,
  attemptId,
}: TicketPageProps) => {
  const { locale, t } = useLanguage();
  const [data, setData] = useState<TicketResponse | null>(null);
  const canLoad = Boolean(holdId && attemptId);
  const [loading, setLoading] = useState(canLoad);
  const [error, setError] = useState<{
    code: string;
    message: string;
  } | null>(
    !canLoad
      ? { code: "INVALID_PARAMS", message: "ticket.error.notFound" }
      : null,
  );
  const [copiedKey, setCopiedKey] = useState<string | null>(null);

  const containerRef = useRef<HTMLDivElement>(null);
  const hasRevealed = useRef(false);

  useEffect(() => {
    if (!canLoad) return;
    let active = true;

    const loadTicket = async () => {
      try {
        const res = await getTicket(holdId, attemptId);
        if (active) {
          setData(res);
          setError(null);
        }
      } catch (err) {
        if (!active) return;
        if (err instanceof BookingApiError) {
          if (err.code === "TICKET_PAYMENT_INCOMPLETE") {
            setError({
              code: err.code,
              message: "ticket.error.paymentIncomplete",
            });
          } else if (err.status === 401) {
            setError({
              code: "UNAUTHORIZED",
              message: "ticket.error.unauthorized",
            });
          } else {
            setError({
              code: err.code,
              message: "ticket.error.notFound",
            });
          }
        } else {
          setError({
            code: "NETWORK_ERROR",
            message: "ticket.error.generic",
          });
        }
      } finally {
        if (active) {
          setLoading(false);
        }
      }
    };

    void loadTicket();
    return () => {
      active = false;
    };
  }, [attemptId, canLoad, holdId]);

  const copyToClipboard = async (text: string, key: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedKey(key);
      window.setTimeout(() => setCopiedKey(null), 2500);
    } catch {
      // Ignore clipboard write failure
    }
  };

  const handlePrint = () => {
    if (typeof window !== "undefined") {
      window.print();
    }
  };

  useGSAP(
    () => {
      if (loading || !data || hasRevealed.current) return;
      hasRevealed.current = true;
      const targets = containerRef.current?.querySelectorAll<HTMLElement>(
        "[data-ticket-reveal]",
      );
      if (!targets?.length) return;

      gsap.fromTo(
        targets,
        { opacity: 0, y: 14 },
        {
          clearProps: "opacity,transform",
          duration: motionDurations.ui,
          ease: gsapEasings.enter,
          opacity: 1,
          stagger: 0.06,
          y: 0,
        },
      );
    },
    {
      dependencies: [loading, data],
      scope: containerRef,
    },
  );

  const arrivalDate = data
    ? new Date(
        Date.parse(`${data.ticket.journey.departureDate}T00:00:00Z`) +
          (data.ticket.journey.arrivalDayOffset ?? 0) * 86_400_000,
      ).toISOString().slice(0, 10)
    : null;

  return (
    <main
      className="relative min-h-screen overflow-x-clip pb-section-md pt-[calc(var(--header-height)+clamp(1.5rem,4vw,3.5rem))] print:p-0"
      data-ticket-page
    >
      {/* Background radial highlight */}
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-x-0 top-0 h-[40rem] bg-[radial-gradient(circle_at_70%_0%,rgba(255,212,0,0.07),transparent_36rem)] print:hidden"
      />

      <Container className="relative print:max-w-none print:p-0" ref={containerRef}>
        {/* Navigation & Header (Hidden during print) */}
        <div className="print:hidden" data-ticket-reveal="nav">
          <Link
            className="inline-flex min-h-11 items-center gap-2 text-sm text-muted-foreground transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus motion-reduce:transition-none"
            href="/"
          >
            <ArrowLeft aria-hidden="true" className="size-4" />
            {t("ticket.back")}
          </Link>
        </div>

        {/* Loading State */}
        {loading ? (
          <div
            aria-label={t("ticket.loading")}
            className="mt-10 min-h-[32rem] animate-pulse rounded-surface border border-border bg-surface/60 motion-reduce:animate-none"
            role="status"
          />
        ) : error ? (
          /* Error State */
          <section
            aria-live="polite"
            className="mt-10 rounded-surface border border-destructive/45 bg-destructive/10 p-7 text-center"
            role="alert"
          >
            <CircleX
              aria-hidden="true"
              className="mx-auto size-12 text-destructive"
            />
            <h1 className="mt-4 text-h3">{t("ticket.heading")}</h1>
            <p className="mx-auto mt-3 max-w-xl text-muted-foreground">
              {t(error.message as TranslationKey)}
            </p>
            <div className="mt-7 flex flex-wrap justify-center gap-4">
              <Link
                className={buttonVariants({ variant: "outline" })}
                href="/"
              >
                {t("ticket.error.returnHome")}
              </Link>
              {holdId ? (
                <Link
                  className={buttonVariants({ variant: "primary" })}
                  href={`/booking/payment?holdId=${encodeURIComponent(holdId)}`}
                >
                  {t("ticket.error.backToPayment")}
                </Link>
              ) : null}
            </div>
          </section>
        ) : data ? (
          /* Authoritative Ticket Card */
          <article
            aria-labelledby="ticket-main-heading"
            className="mt-8"
            data-ticket-content="true"
          >
            {/* Header / Intro */}
            <div className="print:text-black" data-ticket-reveal="heading">
              <div className="flex flex-wrap items-center justify-between gap-4">
                <div>
                  <p className="text-label text-brand">{t("ticket.eyebrow")}</p>
                  <h1 className="mt-2 text-h2" id="ticket-main-heading">
                    {t("ticket.heading")}
                  </h1>
                </div>
                <div
                  className={`inline-flex items-center gap-2 rounded-full border px-4 py-1.5 text-sm font-semibold tracking-wide ${
                    data.ticket.status === "ISSUED"
                      ? "border-brand/50 bg-brand/10 text-brand"
                      : "border-destructive/50 bg-destructive/10 text-destructive"
                  }`}
                  data-ticket-status={data.ticket.status}
                >
                  {data.ticket.status === "ISSUED" ? (
                    <CheckCircle2 aria-hidden="true" className="size-4" />
                  ) : (
                    <CircleX aria-hidden="true" className="size-4" />
                  )}
                  <span>
                    {t(
                      data.ticket.status === "ISSUED"
                        ? "ticket.status.issued"
                        : "ticket.status.cancelled",
                    )}
                  </span>
                </div>
              </div>
              <p className="mt-3 max-w-2xl text-body text-muted-foreground">
                {t("ticket.intro")}
              </p>
            </div>

            {/* Cinematic Airline Ticket Container */}
            <div
              className="mt-8 overflow-hidden rounded-surface border border-brand/35 bg-surface/90 shadow-2xl backdrop-blur-md print:border-neutral-800 print:bg-white print:text-black print:shadow-none"
              data-ticket-reveal="card"
            >
              {/* Gold brand bar */}
              <div
                aria-hidden="true"
                className="h-2 w-full bg-gradient-to-r from-brand via-[#FFE566] to-brand print:bg-black"
              />

              {/* Main Ticket Interior */}
              <div className="grid gap-8 p-6 sm:p-10">
                {/* Left Side: Journey, Passengers, Flight Details */}
                <div className="space-y-8">
                  {/* Branding & Flight Number */}
                  <div className="flex items-center justify-between border-b border-border/70 pb-5 print:border-neutral-300">
                    <div>
                      <span className="font-display text-lg font-extrabold tracking-widest text-brand print:text-black">
                        X-FLY ANYWAY
                      </span>
                      <p className="text-xs uppercase tracking-[0.14em] text-muted-foreground print:text-neutral-600">
                        Global Aviation
                      </p>
                    </div>
                    <div className="text-right">
                      <span className="text-xs uppercase tracking-wider text-muted-foreground print:text-neutral-600">
                        {t("ticket.flight")}
                      </span>
                      <p
                        className="font-mono text-xl font-bold tracking-tight text-foreground print:text-black"
                        data-testid="ticket-flight-number"
                      >
                        {data.ticket.journey.flightNumber}
                      </p>
                    </div>
                  </div>

                  {/* Route Visual Line */}
                  <div className="rounded-control border border-border/60 bg-background/50 p-6 print:border-neutral-300 print:bg-neutral-50">
                    <div className="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
                      <div>
                        <p className="text-xs uppercase tracking-widest text-muted-foreground print:text-neutral-600">
                          {t("ticket.departure")}
                        </p>
                        <p
                          className="mt-1 font-display text-3xl font-black text-foreground sm:text-4xl print:text-black"
                          data-testid="ticket-origin"
                        >
                          {data.ticket.journey.originCode}
                        </p>
                        {data.ticket.journey.departureTime ? (
                          <p className="mt-1 flex items-center gap-1.5 font-mono text-sm text-brand print:text-neutral-800">
                            <Clock aria-hidden="true" className="size-3.5" />
                            {data.ticket.journey.departureTime}
                          </p>
                        ) : null}
                      </div>

                      {/* Flight direction illustration */}
                      <div className="flex flex-1 flex-col items-center px-4 sm:px-8">
                        <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground print:text-neutral-600">
                          <span>
                            {formatDate(data.ticket.journey.departureDate, locale)}
                          </span>
                        </div>
                        <div className="relative mt-2 flex w-full max-w-[14rem] items-center justify-center">
                          <div className="h-[2px] w-full bg-border print:bg-neutral-400" />
                          <div className="absolute rounded-full border border-brand/40 bg-surface p-1.5 text-brand print:border-black print:text-black">
                            <Plane aria-hidden="true" className="size-4 rotate-90" />
                          </div>
                        </div>
                      </div>

                      <div className="text-right">
                        <p className="text-xs uppercase tracking-widest text-muted-foreground print:text-neutral-600">
                          {t("ticket.arrival")}
                        </p>
                        <p
                          className="mt-1 font-display text-3xl font-black text-foreground sm:text-4xl print:text-black"
                          data-testid="ticket-destination"
                        >
                          {data.ticket.journey.destinationCode}
                        </p>
                        {arrivalDate ? (
                          <p className="mt-1 text-sm text-muted-foreground print:text-neutral-600">
                            {formatDate(arrivalDate, locale)}
                          </p>
                        ) : null}
                        {data.ticket.journey.arrivalTime ? (
                          <p className="mt-1 flex items-center justify-end gap-1.5 font-mono text-sm text-brand print:text-neutral-800">
                            <Clock aria-hidden="true" className="size-3.5" />
                            {data.ticket.journey.arrivalTime}
                            {data.ticket.journey.arrivalDayOffset ? (
                              <span className="text-xs text-muted-foreground">
                                (+{data.ticket.journey.arrivalDayOffset})
                              </span>
                            ) : null}
                          </p>
                        ) : null}
                      </div>
                    </div>
                  </div>

                  {/* Passenger & Seats Grid */}
                  <div className="grid gap-4 sm:grid-cols-2">
                    {/* Passenger names */}
                    <div className="rounded-control border border-border/60 bg-background/30 p-5 print:border-neutral-300 print:bg-neutral-50">
                      <p className="flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-muted-foreground print:text-neutral-600">
                        <User aria-hidden="true" className="size-4" />
                        {t("ticket.passengers")}
                      </p>
                      <ul
                        aria-label={t("ticket.passengers")}
                        className="mt-3 space-y-1 text-base font-semibold text-foreground print:text-black"
                        data-testid="ticket-passengers"
                      >
                        {data.ticket.passengers.map((p, idx) => (
                          <li key={idx}>{p.displayName}</li>
                        ))}
                      </ul>
                    </div>

                    {/* Booked seats & Cabin */}
                    <div className="rounded-control border border-border/60 bg-background/30 p-5 print:border-neutral-300 print:bg-neutral-50">
                      <div className="flex items-center justify-between">
                        <p className="flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-muted-foreground print:text-neutral-600">
                          <Armchair aria-hidden="true" className="size-4" />
                          {t("ticket.seats")}
                        </p>
                        <span className="rounded-full border border-border px-2.5 py-0.5 text-xs font-semibold uppercase text-brand print:border-neutral-400 print:text-black">
                          {cabinKeyMap[data.ticket.journey.cabin]
                            ? t(cabinKeyMap[data.ticket.journey.cabin])
                            : data.ticket.journey.cabin}
                        </span>
                      </div>
                      <div
                        className="mt-3 flex flex-wrap gap-2"
                        data-testid="ticket-seats"
                      >
                        {data.ticket.seats.map((seat) => (
                          <span
                            className="inline-flex min-w-10 items-center justify-center rounded-control border border-brand/40 bg-brand/10 px-3 py-1 font-mono text-lg font-bold text-foreground print:border-black print:text-black"
                            key={seat}
                          >
                            {seat}
                          </span>
                        ))}
                      </div>
                    </div>
                  </div>

                  {/* Reference numbers & Amount paid */}
                  <div className="grid gap-4 rounded-control border border-border/60 bg-background/30 p-5 sm:grid-cols-3 print:border-neutral-300 print:bg-neutral-50">
                    {/* Booking Reference */}
                    <div>
                      <span className="text-xs uppercase tracking-wider text-muted-foreground print:text-neutral-600">
                        {t("ticket.bookingReference")}
                      </span>
                      <div className="mt-1 flex items-center gap-2">
                        <span
                          className="font-mono text-lg font-black tracking-widest text-brand print:text-black"
                          data-testid="ticket-booking-reference"
                        >
                          {data.ticket.bookingReference}
                        </span>
                        <button
                          aria-label={t("ticket.copyReference")}
                          className="inline-flex size-7 items-center justify-center rounded-control text-muted-foreground transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus print:hidden"
                          onClick={() =>
                            copyToClipboard(
                              data.ticket.bookingReference,
                              "bookingReference",
                            )
                          }
                          type="button"
                        >
                          {copiedKey === "bookingReference" ? (
                            <Check aria-hidden="true" className="size-3.5 text-brand" />
                          ) : (
                            <Copy aria-hidden="true" className="size-3.5" />
                          )}
                        </button>
                      </div>
                    </div>

                    {/* Ticket Number */}
                    <div>
                      <span className="text-xs uppercase tracking-wider text-muted-foreground print:text-neutral-600">
                        {t("ticket.ticketNumber")}
                      </span>
                      <div className="mt-1 flex items-center gap-2">
                        <span
                          className="font-mono text-sm font-semibold tracking-wider text-foreground print:text-black"
                          data-testid="ticket-number"
                        >
                          {data.ticket.ticketNumber}
                        </span>
                        <button
                          aria-label={t("ticket.copyReference")}
                          className="inline-flex size-7 items-center justify-center rounded-control text-muted-foreground transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus print:hidden"
                          onClick={() =>
                            copyToClipboard(data.ticket.ticketNumber, "ticketNumber")
                          }
                          type="button"
                        >
                          {copiedKey === "ticketNumber" ? (
                            <Check aria-hidden="true" className="size-3.5 text-brand" />
                          ) : (
                            <Copy aria-hidden="true" className="size-3.5" />
                          )}
                        </button>
                      </div>
                    </div>

                    {/* Amount Paid */}
                    <div>
                      <span className="text-xs uppercase tracking-wider text-muted-foreground print:text-neutral-600">
                        {t("ticket.amountPaid")}
                      </span>
                      <p
                        className="mt-1 font-mono text-base font-bold text-foreground print:text-black"
                        data-testid="ticket-amount"
                      >
                        {formatPrice(
                          data.ticket.amount,
                          locale,
                          data.ticket.currencyCode,
                        )}
                      </p>
                      <span className="flex items-center gap-1 text-[11px] text-brand print:text-neutral-700">
                        <CreditCard aria-hidden="true" className="size-3" />
                        {t("ticket.paymentSucceeded")}
                      </span>
                    </div>
                  </div>
                </div>

              </div>
            </div>

            {/* Final summary actions */}
            <div
              className="mt-8 flex items-center print:hidden"
              data-ticket-reveal="actions"
            >
              <Button
                className="gap-2"
                onClick={handlePrint}
                type="button"
                variant="outline"
              >
                <Printer aria-hidden="true" className="size-4" />
                {t("ticket.actions.print")}
              </Button>

            </div>
          </article>
        ) : null}
      </Container>
    </main>
  );
};

export { TicketPage };
