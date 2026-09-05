"use client";

import { ArrowLeft, Check, ShieldCheck } from "lucide-react";
import Link from "next/link";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { allowlistedRecoveryParams, buildExtrasHandoffHref, buildReviewHandoffHref } from "@/components/booking/passengers/passengerRoute";
import { BitcoinPaymentPanel } from "@/components/booking/payment/BitcoinPaymentPanel";
import { CardPaymentForm } from "@/components/booking/payment/CardPaymentForm";
import { PaymentMethodSelector } from "@/components/booking/payment/PaymentMethodSelector";
import { PaymentStatusPanel } from "@/components/booking/payment/PaymentStatusPanel";
import { PaymentSummary } from "@/components/booking/payment/PaymentSummary";
import { BookingApiError, createPaymentAttempt, getPaymentContext, simulateBitcoinPayment } from "@/components/booking/payment/paymentClient";
import type { BitcoinSimulationOutcome, PaymentAttempt, PaymentContext, PaymentMethod } from "@/components/booking/payment/paymentTypes";
import { getRemainingHoldMilliseconds } from "@/components/booking/seats/seatHoldClient";
import { Container } from "@/components/layout/Container";
import { Button } from "@/components/ui/Button";
import { formatPrice } from "@/i18n/formatters";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";
import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";
import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";

type RecoveryAction = "extras" | "passengers" | "retry" | "review" | "seats";
type RecoveryState = { action: RecoveryAction; message: TranslationKey };

const recoveryForError = (error: BookingApiError): RecoveryState => {
  if (error.code === "PASSENGERS_NOT_READY") return { action: "passengers", message: "payment.recovery.passengers" };
  if (error.code === "EXTRAS_NOT_READY") return { action: "extras", message: "payment.recovery.extras" };
  if (["REVIEW_NOT_READY", "REVIEW_PRICING_UNAVAILABLE"].includes(error.code)) return { action: "review", message: "payment.recovery.review" };
  if (error.code === "HOLD_EXPIRED") return { action: "seats", message: "payment.recovery.expired" };
  if (error.code === "HOLD_RELEASED") return { action: "seats", message: "payment.recovery.released" };
  if (error.code === "HOLD_CONSUMED") return { action: "seats", message: "payment.recovery.consumed" };
  if (error.code === "SEATS_NOT_READY") return { action: "seats", message: "payment.recovery.seats" };
  if (["HOLD_UNAUTHORIZED", "HOLD_NOT_FOUND"].includes(error.code)) return { action: "seats", message: "payment.recovery.unauthorized" };
  return { action: "retry", message: "payment.recovery.unavailable" };
};

const createRequestId = () => {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) return crypto.randomUUID();
  return `00000000-0000-4000-8000-${Date.now().toString(16).padStart(12, "0").slice(-12)}`;
};

const seatHref = (backQuery: string, context: PaymentContext | null) => {
  const params = allowlistedRecoveryParams(backQuery);
  const flightId = context?.hold.flightId ?? params.get("flightId");
  params.delete("flightId");
  if (context) {
    params.set("departure", context.hold.departureDate);
    params.set("adults", String(context.hold.passengers.adults));
    params.set("children", String(context.hold.passengers.children));
    params.set("infants", String(context.hold.passengers.infants));
    params.set("selectedCabin", context.hold.cabin);
  }
  return flightId ? `/flights/${encodeURIComponent(flightId)}/seats?${params.toString()}` : `/flights?${params.toString()}`;
};

const PaymentPage = ({ backQuery, holdId }: { backQuery: string; holdId: string }) => {
  const { locale, t } = useLanguage();
  const page = useRef<HTMLDivElement>(null);
  const hasRevealed = useRef(false);
  const reducedMotion = useReducedMotion();
  const [context, setContext] = useState<PaymentContext | null>(null);
  const [loading, setLoading] = useState(Boolean(holdId));
  const [loadAttempt, setLoadAttempt] = useState(0);
  const [recovery, setRecovery] = useState<RecoveryState | null>(holdId ? null : { action: "seats", message: "payment.recovery.invalid" });
  const [method, setMethod] = useState<PaymentMethod>("CARD");
  const [busy, setBusy] = useState(false);
  const [receivedAt, setReceivedAt] = useState(0);
  const [remainingMilliseconds, setRemainingMilliseconds] = useState(0);
  const [cardSession, setCardSession] = useState<string | undefined>();
  const [cardConfirming, setCardConfirming] = useState(false);
  const [stillConfirming, setStillConfirming] = useState(false);
  const pollCount = useRef(0);

  const applyContext = useCallback((next: PaymentContext) => {
    const now = Date.now();
    setContext(next);
    setReceivedAt(now);
    setRemainingMilliseconds(getRemainingHoldMilliseconds({ clientNow: now, expiresAt: next.hold.expiresAt, serverTime: next.hold.serverTime, serverTimeReceivedAt: now }));
  }, []);

  useEffect(() => {
    if (!holdId) return;
    let active = true;
    const load = async () => {
      setLoading(true);
      try {
        const next = await getPaymentContext(holdId);
        if (active) { applyContext(next); setRecovery(null); }
      } catch (error) {
        if (active) setRecovery(error instanceof BookingApiError ? recoveryForError(error) : { action: "retry", message: "payment.recovery.unavailable" });
      } finally { if (active) setLoading(false); }
    };
    void load();
    return () => { active = false; };
  }, [applyContext, holdId, loadAttempt]);

  const latestAttempt = context?.attempts[0] ?? null;
  const succeeded = latestAttempt?.status === "SUCCEEDED";
  const cardAttemptOpen = latestAttempt?.provider === "STRIPE" && ["CREATED", "PROCESSING", "AWAITING_PAYMENT"].includes(latestAttempt.status);

  const checkPaymentStatus = useCallback(async () => {
    if (!holdId) return;
    try {
      const next = await getPaymentContext(holdId);
      applyContext(next);
      const current = next.attempts[0];
      if (["SUCCEEDED", "FAILED", "CANCELLED"].includes(current?.status ?? "")) {
        setCardConfirming(false);
        setStillConfirming(false);
      }
    } catch {
      // A provider or transport outage is not a payment failure. Keep the
      // current protected attempt visibly confirming and let the customer retry.
    }
  }, [applyContext, holdId]);

  useEffect(() => {
    if (!cardConfirming || !cardAttemptOpen || stillConfirming) return;
    if (pollCount.current >= 15) { setStillConfirming(true); return; }
    const timeout = window.setTimeout(() => {
      pollCount.current += 1;
      void checkPaymentStatus();
    }, 2_000);
    return () => window.clearTimeout(timeout);
  }, [cardAttemptOpen, cardConfirming, checkPaymentStatus, stillConfirming, latestAttempt?.updatedAt]);

  useEffect(() => {
    if (!context || !receivedAt || recovery || succeeded) return;
    const update = () => {
      const remaining = getRemainingHoldMilliseconds({ clientNow: Date.now(), expiresAt: context.hold.expiresAt, serverTime: context.hold.serverTime, serverTimeReceivedAt: receivedAt });
      setRemainingMilliseconds(remaining);
      if (remaining === 0) setRecovery({ action: "seats", message: "payment.recovery.expired" });
    };
    const interval = window.setInterval(update, 1_000);
    return () => window.clearInterval(interval);
  }, [context, receivedAt, recovery, succeeded]);

  const updateAttempt = (attempt: PaymentAttempt) => setContext((current) => current ? { ...current, attempts: [attempt, ...current.attempts.filter((item) => item.id !== attempt.id)], readyForPayment: attempt.status !== "SUCCEEDED" } : current);
  const preferredLocale = locale.toUpperCase() as "EN" | "TH";
  const run = async (operation: () => Promise<PaymentAttempt>) => {
    setBusy(true);
    try { updateAttempt(await operation()); }
    catch (error) {
      if (error instanceof BookingApiError && error.code === "PAYMENT_ALREADY_SUCCEEDED") setLoadAttempt((value) => value + 1);
      else setRecovery(error instanceof BookingApiError ? recoveryForError(error) : { action: "retry", message: "payment.recovery.unavailable" });
    } finally { setBusy(false); }
  };
  const submitCard = async () => {
    setBusy(true);
    try { const attempt = await createPaymentAttempt(holdId, { method: "CARD", preferredLocale, requestId: createRequestId() }); updateAttempt(attempt); setCardSession(attempt.clientPaymentSession); setCardConfirming(false); setStillConfirming(false); }
    catch (error) { setRecovery(error instanceof BookingApiError ? recoveryForError(error) : { action: "retry", message: "payment.recovery.unavailable" }); }
    finally { setBusy(false); }
  };
  const createBitcoin = () => run(() => createPaymentAttempt(holdId, { method: "BITCOIN", preferredLocale, requestId: createRequestId() }));
  const simulateBitcoin = (outcome: BitcoinSimulationOutcome) => latestAttempt ? run(() => simulateBitcoinPayment(holdId, latestAttempt.id, outcome)) : Promise.resolve();
  const amount = useMemo(() => context ? formatPrice(context.pricing.grandTotal.amount, locale, context.pricing.currencyCode) : "", [context, locale]);
  const reviewHref = buildReviewHandoffHref({ holdId, query: backQuery });
  const extrasHref = buildExtrasHandoffHref({ holdId, query: backQuery });
  const passengerParams = allowlistedRecoveryParams(backQuery);
  passengerParams.set("holdId", holdId);
  const recoveryHref = recovery?.action === "passengers" ? `/booking/passengers?${passengerParams.toString()}` : recovery?.action === "extras" ? extrasHref : recovery?.action === "review" ? reviewHref : seatHref(backQuery, context);
  const recoveryLabel = recovery?.action === "passengers" ? "payment.recovery.passengersAction" : recovery?.action === "extras" ? "payment.recovery.extrasAction" : recovery?.action === "review" ? "payment.recovery.reviewAction" : "payment.recovery.seatsAction";
  const open = ["CREATED", "PROCESSING", "AWAITING_PAYMENT"].includes(latestAttempt?.status ?? "");
  const revealReady = Boolean(context && !loading && !recovery);

  useGSAP(
    () => {
      if (!revealReady || hasRevealed.current) return;
      hasRevealed.current = true;
      if (reducedMotion) return;
      const targets = page.current?.querySelectorAll<HTMLElement>(
        "[data-payment-reveal]",
      );
      if (!targets?.length) return;

      gsap.fromTo(
        targets,
        { opacity: 0, y: 10 },
        {
          clearProps: "opacity,transform",
          duration: motionDurations.ui,
          ease: gsapEasings.enter,
          opacity: 1,
          stagger: 0.055,
          y: 0,
        },
      );
    },
    {
      dependencies: [reducedMotion, revealReady],
      scope: page,
    },
  );

  return (
    <main className="relative min-h-screen overflow-x-clip pb-section-md pt-[calc(var(--header-height)+clamp(2rem,5vw,4rem))]" data-reduced-motion={reducedMotion || undefined}>
      <div aria-hidden="true" className="pointer-events-none absolute inset-x-0 top-0 h-[36rem] bg-[radial-gradient(circle_at_72%_0%,rgba(255,212,0,0.08),transparent_32rem)]" />
      <Container className="relative" ref={page}>
        <div data-payment-reveal="navigation">
          <Link className="inline-flex min-h-11 items-center gap-2 text-sm text-muted-foreground transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus motion-reduce:transition-none" href={reviewHref}><ArrowLeft aria-hidden="true" />{t("payment.back")}</Link>
          <ol aria-label={t("payment.progress.label")} className="mt-8 flex flex-wrap items-center gap-3 text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground">{["seat", "passenger", "extras", "review"].map((step) => <li className="flex items-center gap-2 text-foreground" key={step}><Check aria-hidden="true" className="size-4 text-brand" />{t(`payment.progress.${step}` as TranslationKey)}</li>)}<li aria-current="step" className="text-brand">{t("payment.progress.payment")}</li></ol>
        </div>
        <div className="mt-10" data-payment-reveal="heading"><p className="text-label text-brand">{t("payment.eyebrow")}</p><h1 className="mt-3 text-h1">{t("payment.heading")}</h1><p className="mt-5 max-w-2xl text-body-lg text-muted-foreground">{t("payment.intro")}</p></div>
        {loading ? <div aria-label={t("payment.loading")} className="mt-10 min-h-72 animate-pulse rounded-surface border border-border bg-surface/60 motion-reduce:animate-none" role="status" /> : recovery ? <section className="payment-failure-in mt-10 rounded-surface border border-destructive/45 bg-destructive/10 p-7" data-payment-failure="true"><p role="alert">{t(recovery.message)}</p>{recovery.action === "retry" ? <Button className="mt-5" onClick={() => setLoadAttempt((value) => value + 1)}>{t("payment.recovery.retry")}</Button> : <Link className="mt-5 inline-flex min-h-11 items-center gap-2 font-medium text-brand focus-visible:ring-2 focus-visible:ring-focus" href={recoveryHref}><ArrowLeft aria-hidden="true" />{t(recoveryLabel)}</Link>}</section> : context ? <div className="mt-10 grid min-w-0 gap-8 lg:grid-cols-[minmax(0,1fr)_20rem] lg:items-start" data-payment-layout><div className="min-w-0"><div className="rounded-control border border-brand/35 bg-brand/5 p-4" data-payment-reveal="notice"><p className="flex items-center gap-2 text-sm font-semibold text-brand"><ShieldCheck aria-hidden="true" className="size-5" />{t("payment.demo.badge")}</p><p className="mt-2 text-sm text-muted-foreground">{t("payment.demo.notice")}</p></div>          <div className="md:min-h-[31rem]" data-payment-reveal="checkout">
            {succeeded && latestAttempt ? (
              <div className="mt-7">
                <PaymentStatusPanel
                  attempt={latestAttempt}
                  backQuery={backQuery}
                  holdId={holdId}
                />
              </div>
            ) : (
              <>
                <div className="mt-7">
                  <PaymentMethodSelector
                    disabled={busy || open}
                    onChange={setMethod}
                    value={method}
                  />
                </div>
                {latestAttempt &&
                ["FAILED", "CANCELLED"].includes(latestAttempt.status) ? (
                  <div className="mt-6">
                    <PaymentStatusPanel
                      attempt={latestAttempt}
                      backQuery={backQuery}
                      holdId={holdId}
                    />
                  </div>
                ) : null}
                {stillConfirming ? (
                  <section
                    aria-live="polite"
                    className="mt-6 rounded-control border border-border bg-surface p-5"
                    role="status"
                  >
                    <p>{t("payment.card.stillConfirming")}</p>
                    <Button
                      className="mt-4"
                      onClick={() => {
                        pollCount.current = 0;
                        setStillConfirming(false);
                        void checkPaymentStatus();
                      }}
                    >
                      {t("payment.card.checkStatus")}
                    </Button>
                  </section>
                ) : null}
                {method === "CARD" ? (
                  <CardPaymentForm
                    amount={amount}
                    clientSecret={cardSession}
                    confirming={cardConfirming}
                    disabled={
                      busy ||
                      remainingMilliseconds === 0 ||
                      (open && !cardSession)
                    }
                    onConfirmed={() => {
                      pollCount.current = 0;
                      setStillConfirming(false);
                      setCardConfirming(true);
                    }}
                    onStart={submitCard}
                  />
                ) : (
                  <BitcoinPaymentPanel
                    attempt={
                      latestAttempt?.paymentMethod === "BITCOIN"
                        ? latestAttempt
                        : null
                    }
                    busy={busy}
                    onCreate={createBitcoin}
                    onSimulate={simulateBitcoin}
                    sourceAmount={amount}
                  />
                )}
              </>
            )}
          </div>
        </div>
        <PaymentSummary
          context={context}
          remainingMilliseconds={remainingMilliseconds}
        />
      </div>
    : null}
      </Container>
    </main>
  );
};

export { PaymentPage };
