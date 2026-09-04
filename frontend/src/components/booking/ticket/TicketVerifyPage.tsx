"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import {
  ArrowLeft,
  CheckCircle2,
  CircleX,
  Plane,
  ShieldCheck,
} from "lucide-react";

import { verifyTicket } from "@/components/booking/ticket/ticketClient";
import type { TicketVerification } from "@/components/booking/ticket/ticketTypes";
import { Container } from "@/components/layout/Container";
import { buttonVariants } from "@/components/ui/Button";
import { useLanguage } from "@/i18n/LanguageProvider";

type TicketVerifyPageProps = {
  token: string;
};

export function TicketVerifyPage({ token }: TicketVerifyPageProps) {
  const { t } = useLanguage();
  const [loading, setLoading] = useState(true);
  const [verification, setVerification] = useState<TicketVerification | null>(
    null,
  );
  const [fetchError, setFetchError] = useState(false);

  useEffect(() => {
    let active = true;

    const performVerification = async () => {
      setLoading(true);
      setFetchError(false);
      try {
        const result = await verifyTicket(token);
        if (active) {
          setVerification(result);
        }
      } catch {
        if (active) {
          setFetchError(true);
        }
      } finally {
        if (active) {
          setLoading(false);
        }
      }
    };

    void performVerification();

    return () => {
      active = false;
    };
  }, [token]);

  const isValid = !fetchError && verification?.valid === true;

  return (
    <main className="relative min-h-screen overflow-x-clip pb-section-md pt-[calc(var(--header-height)+clamp(1.5rem,4vw,3.5rem))]">
      {/* Background radial highlight */}
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-x-0 top-0 h-[40rem] bg-[radial-gradient(circle_at_70%_0%,rgba(255,212,0,0.07),transparent_36rem)]"
      />

      <Container className="relative max-w-2xl">
        {/* Navigation */}
        <div>
          <Link
            className="inline-flex min-h-11 items-center gap-2 text-sm text-muted-foreground transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus"
            href="/"
          >
            <ArrowLeft aria-hidden="true" className="size-4" />
            {t("ticket.back")}
          </Link>
        </div>

        {/* Verification Card */}
        <div className="mt-8 overflow-hidden rounded-surface border border-border/80 bg-surface/90 shadow-2xl backdrop-blur-md">
          {/* Top Brand Bar */}
          <div
            aria-hidden="true"
            className={`h-2.5 w-full ${
              loading
                ? "bg-muted"
                : isValid
                  ? "bg-brand"
                  : "bg-destructive"
            }`}
          />

          <div className="p-6 sm:p-10">
            {/* Header */}
            <div className="text-center">
              <p className="text-label text-brand">
                {t("ticket.verify.eyebrow")}
              </p>
              <h1 className="mt-2 text-h2 font-bold tracking-tight text-foreground">
                {t("ticket.verify.heading")}
              </h1>
            </div>

            {/* State Rendering */}
            {loading ? (
              <div
                aria-label={t("ticket.verify.verifying")}
                className="mt-10 flex flex-col items-center justify-center py-12"
                role="status"
              >
                <div className="size-12 animate-spin rounded-full border-4 border-brand/20 border-t-brand" />
                <p className="mt-4 text-sm text-muted-foreground">
                  {t("ticket.verify.verifying")}
                </p>
              </div>
            ) : isValid && verification ? (
              /* Valid State */
              <div className="mt-8 space-y-6" data-testid="verification-valid">
                <div className="flex flex-col items-center justify-center rounded-control border border-brand/40 bg-brand/10 p-6 text-center">
                  <CheckCircle2
                    aria-hidden="true"
                    className="size-12 text-brand"
                  />
                  <h2 className="mt-3 text-lg font-bold text-foreground">
                    {t("ticket.verify.validTitle")}
                  </h2>
                  <p className="mt-1 text-xs text-muted-foreground max-w-md">
                    {t("ticket.verify.validDescription")}
                  </p>
                </div>

                {/* Flight & Operational Details (No PII) */}
                <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                  {/* Flight Number */}
                  <div className="rounded-control border border-border/60 bg-background/50 p-4">
                    <span className="text-xs uppercase tracking-wider text-muted-foreground">
                      {t("ticket.verify.flightLabel")}
                    </span>
                    <div className="mt-1 flex items-center gap-2">
                      <Plane
                        aria-hidden="true"
                        className="size-4 text-brand"
                      />
                      <span
                        className="font-mono text-base font-bold text-foreground"
                        data-testid="verify-flight-number"
                      >
                        {verification.flightNumber || "XF 201"}
                      </span>
                    </div>
                  </div>

                  {/* Route */}
                  <div className="rounded-control border border-border/60 bg-background/50 p-4">
                    <span className="text-xs uppercase tracking-wider text-muted-foreground">
                      {t("ticket.verify.routeLabel")}
                    </span>
                    <p
                      className="mt-1 font-mono text-base font-bold text-foreground"
                      data-testid="verify-route"
                    >
                      {verification.originCode} → {verification.destinationCode}
                    </p>
                  </div>

                  {/* Departure Date & Time */}
                  <div className="rounded-control border border-border/60 bg-background/50 p-4">
                    <span className="text-xs uppercase tracking-wider text-muted-foreground">
                      {t("ticket.verify.dateLabel")}
                    </span>
                    <p
                      className="mt-1 text-sm font-semibold text-foreground"
                      data-testid="verify-date"
                    >
                      {verification.departureDate}
                      {verification.departureTime
                        ? ` · ${verification.departureTime}`
                        : ""}
                    </p>
                  </div>

                  {/* Status */}
                  <div className="rounded-control border border-border/60 bg-background/50 p-4">
                    <span className="text-xs uppercase tracking-wider text-muted-foreground">
                      {t("ticket.verify.statusLabel")}
                    </span>
                    <div className="mt-1 flex items-center gap-2">
                      <span
                        className="inline-flex items-center rounded-full bg-brand/10 px-2.5 py-0.5 text-xs font-semibold text-brand"
                        data-testid="verify-status"
                      >
                        {verification.ticketStatus || "ISSUED"}
                      </span>
                    </div>
                  </div>
                </div>

                {/* Assigned Seats */}
                {verification.seats?.length ? (
                  <div className="rounded-control border border-border/60 bg-background/50 p-4">
                    <span className="text-xs uppercase tracking-wider text-muted-foreground">
                      {t("ticket.verify.seatsLabel")}
                    </span>
                    <div
                      className="mt-2 flex flex-wrap gap-2"
                      data-testid="verify-seats"
                    >
                      {verification.seats.map((seat) => (
                        <span
                          className="rounded border border-brand/40 bg-brand/10 px-3 py-1 font-mono text-sm font-bold text-brand"
                          key={seat}
                        >
                          {seat}
                        </span>
                      ))}
                    </div>
                  </div>
                ) : null}

                {/* Security Footer Notice */}
                <div className="rounded-control border border-border/40 bg-background/30 p-3 text-center">
                  <p className="flex items-center justify-center gap-1.5 text-[11px] text-muted-foreground">
                    <ShieldCheck
                      aria-hidden="true"
                      className="size-3.5 text-brand"
                    />
                    {t("ticket.verify.securityNotice")}
                  </p>
                </div>
              </div>
            ) : (
              /* Invalid State */
              <div className="mt-8 space-y-6" data-testid="verification-invalid">
                <div className="flex flex-col items-center justify-center rounded-control border border-destructive/40 bg-destructive/10 p-6 text-center">
                  <CircleX
                    aria-hidden="true"
                    className="size-12 text-destructive"
                  />
                  <h2 className="mt-3 text-lg font-bold text-destructive">
                    {t("ticket.verify.invalidTitle")}
                  </h2>
                  <p className="mt-1 text-xs text-muted-foreground max-w-md">
                    {t("ticket.verify.invalidDescription")}
                  </p>
                </div>
              </div>
            )}

            {/* Actions */}
            <div className="mt-8 flex justify-center">
              <Link
                className={buttonVariants({ variant: "outline" })}
                href="/"
              >
                {t("ticket.verify.returnHome")}
              </Link>
            </div>
          </div>
        </div>
      </Container>
    </main>
  );
}
