"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { ArrowLeft, CheckCircle2, CircleX, Plane } from "lucide-react";

import { verifyBoardingPass } from "@/components/boarding-pass/boardingPassClient";
import type { BoardingPassVerification } from "@/lib/admin/ticketOperationsTypes";
import { Container } from "@/components/layout/Container";
import { useLanguage } from "@/i18n/LanguageProvider";

export function BoardingPassVerifyPage({ token }: { token: string }) {
  const { t } = useLanguage();
  const [loading, setLoading] = useState(true);
  const [fetchError, setFetchError] = useState(false);
  const [verification, setVerification] = useState<BoardingPassVerification | null>(null);

  useEffect(() => {
    let active = true;
    void verifyBoardingPass(token)
      .then((result) => {
        if (active) setVerification(result);
      })
      .catch(() => {
        if (active) setFetchError(true);
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [token]);

  const valid = !fetchError && verification?.valid === true;
  const expired = verification?.invalidReason === "EXPIRED";

  return (
    <div className="relative min-h-screen overflow-x-clip pb-section-md pt-[calc(var(--header-height)+clamp(1.5rem,4vw,3.5rem))]">
      <Container className="relative max-w-2xl">
        <Link className="inline-flex min-h-11 items-center gap-2 text-sm text-muted-foreground focus-visible:ring-2 focus-visible:ring-focus" href="/">
          <ArrowLeft aria-hidden="true" className="size-4" />
          {t("boardingPass.verificationReturnHome")}
        </Link>
        <div className="mt-8 overflow-hidden rounded-surface border border-border/80 bg-surface/90 shadow-2xl">
          <div className={`h-2.5 w-full ${loading ? "bg-muted" : valid ? "bg-brand" : "bg-destructive"}`} />
          <div className="p-6 sm:p-10">
            <p className="text-label text-brand">{t("boardingPass.verificationEyebrow")}</p>
            <h1 className="mt-2 text-h2 font-bold tracking-tight text-foreground">{t("boardingPass.verificationHeading")}</h1>
            <p className="mt-3 text-sm text-muted-foreground">{t("boardingPass.verificationSecurityNotice")}</p>
            {loading ? (
              <div className="mt-10 flex flex-col items-center py-12" role="status">
                <div className="size-12 animate-spin rounded-full border-4 border-brand/20 border-t-brand" />
                <p className="mt-4 text-sm text-muted-foreground">{t("boardingPass.verificationChecking")}</p>
              </div>
            ) : (
              <div className="mt-8 space-y-6" data-testid={valid ? "boarding-pass-valid" : "boarding-pass-invalid"}>
                <div className={`flex flex-col items-center rounded-control border p-6 text-center ${valid ? "border-brand/40 bg-brand/10" : "border-destructive/40 bg-destructive/10"}`}>
                  {valid ? <CheckCircle2 aria-hidden="true" className="size-12 text-brand" /> : <CircleX aria-hidden="true" className="size-12 text-destructive" />}
                  <h2 className="mt-3 text-lg font-bold text-foreground">
                    {valid ? t("boardingPass.verificationValidTitle") : t("boardingPass.verificationInvalidTitle")}
                  </h2>
                  <p className="mt-1 max-w-md text-xs text-muted-foreground">
                    {fetchError
                      ? t("boardingPass.verificationUnknownDescription")
                      : expired
                        ? t("boardingPass.verificationExpiredDescription")
                        : valid
                          ? t("boardingPass.verificationValidDescription")
                          : t("boardingPass.verificationInvalidDescription")}
                  </p>
                </div>
                {verification?.flightNumber ? (
                  <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                    <div className="rounded-control border border-border/60 bg-background/50 p-4">
                      <span className="text-xs uppercase tracking-wider text-muted-foreground">{t("boardingPass.flight")}</span>
                      <div className="mt-1 flex items-center gap-2"><Plane aria-hidden="true" className="size-4 text-brand" /><span className="font-mono text-base font-bold">{verification.flightNumber}</span></div>
                    </div>
                    <div className="rounded-control border border-border/60 bg-background/50 p-4">
                      <span className="text-xs uppercase tracking-wider text-muted-foreground">{t("boardingPass.fromTo")}</span>
                      <p className="mt-1 font-mono text-base font-bold">{verification.originCode} → {verification.destinationCode}</p>
                    </div>
                    {verification.departureAt ? <div className="rounded-control border border-border/60 bg-background/50 p-4"><span className="text-xs uppercase tracking-wider text-muted-foreground">{t("boardingPass.departure")}</span><p className="mt-1 font-mono text-sm font-bold">{new Intl.DateTimeFormat(localeForDateTime(), { dateStyle: "medium", timeStyle: "short", ...(verification.originTimeZone ? { timeZone: verification.originTimeZone } : {}) }).format(new Date(verification.departureAt))}</p></div> : null}
                    {verification.seat ? <div className="rounded-control border border-border/60 bg-background/50 p-4"><span className="text-xs uppercase tracking-wider text-muted-foreground">{t("boardingPass.seat")}</span><p className="mt-1 font-mono text-base font-bold">{verification.seat}</p></div> : null}
                  </div>
                ) : null}
              </div>
            )}
          </div>
        </div>
      </Container>
    </div>
  );

  function localeForDateTime() {
    return typeof document !== "undefined" && document.documentElement.lang === "th" ? "th-TH-u-ca-buddhist-nu-latn" : "en-GB-u-ca-gregory-nu-latn";
  }
}
