"use client";

import { ArrowLeft, Check } from "lucide-react";
import Link from "next/link";
import { useCallback, useEffect, useRef, useState } from "react";

import { buildExtrasHandoffHref, buildPaymentHandoffHref, allowlistedRecoveryParams } from "@/components/booking/passengers/passengerRoute";
import { FareSummary } from "@/components/booking/review/FareSummary";
import { BookingApiError, getReviewContext } from "@/components/booking/review/reviewClient";
import { ReviewDetails } from "@/components/booking/review/ReviewDetails";
import type { ReviewContext } from "@/components/booking/review/reviewTypes";
import { getRemainingHoldMilliseconds } from "@/components/booking/seats/seatHoldClient";
import { Container } from "@/components/layout/Container";
import { Button } from "@/components/ui/Button";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";
import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";
import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";

type RecoveryKind = "extras" | "passengers" | "retry" | "seats";
type ReviewState = { action: RecoveryKind; message: TranslationKey };

const stateForError = (error: BookingApiError): ReviewState => {
  if (error.code === "PASSENGERS_NOT_READY") return { action: "passengers", message: "review.state.passengers" };
  if (error.code === "EXTRAS_NOT_READY") return { action: "extras", message: "review.state.extras" };
  if (error.code === "HOLD_EXPIRED") return { action: "seats", message: "review.state.expired" };
  if (error.code === "HOLD_RELEASED") return { action: "seats", message: "review.state.released" };
  if (error.code === "SEATS_NOT_READY") return { action: "seats", message: "review.state.seats" };
  if (["HOLD_UNAUTHORIZED", "HOLD_NOT_FOUND"].includes(error.code)) return { action: "seats", message: "review.state.unauthorized" };
  if (error.code === "HOLD_CONSUMED") return { action: "seats", message: "review.state.consumed" };
  if (error.code === "REVIEW_PRICING_UNAVAILABLE") return { action: "retry", message: "review.state.pricing" };
  return { action: "retry", message: "review.state.unavailable" };
};

const passengerHref = (backQuery: string, holdId: string) => {
  const params = allowlistedRecoveryParams(backQuery);
  params.set("holdId", holdId);
  return `/booking/passengers?${params.toString()}`;
};

const seatHref = (backQuery: string, context: ReviewContext | null) => {
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

const ReviewPage = ({ backQuery, holdId }: { backQuery: string; holdId: string }) => {
  const { t } = useLanguage();
  const page = useRef<HTMLDivElement>(null);
  const reducedMotion = useReducedMotion();
  const [context, setContext] = useState<ReviewContext | null>(null);
  const [loading, setLoading] = useState(Boolean(holdId));
  const [attempt, setAttempt] = useState(0);
  const [reviewState, setReviewState] = useState<ReviewState | null>(
    holdId ? null : { action: "seats", message: "review.state.invalid" },
  );
  const [receivedAt, setReceivedAt] = useState(0);
  const [remainingMilliseconds, setRemainingMilliseconds] = useState(0);

  const applyContext = useCallback((next: ReviewContext) => {
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
        const next = await getReviewContext(holdId);
        if (active) {
          applyContext(next);
          setReviewState(null);
        }
      } catch (error) {
        if (active) setReviewState(error instanceof BookingApiError ? stateForError(error) : { action: "retry", message: "review.state.unavailable" });
      } finally {
        if (active) setLoading(false);
      }
    };
    void load();
    return () => { active = false; };
  }, [applyContext, attempt, holdId]);

  useEffect(() => {
    if (!context || !receivedAt || reviewState) return;
    const update = () => {
      const remaining = getRemainingHoldMilliseconds({ clientNow: Date.now(), expiresAt: context.hold.expiresAt, serverTime: context.hold.serverTime, serverTimeReceivedAt: receivedAt });
      setRemainingMilliseconds(remaining);
      if (remaining === 0) setReviewState({ action: "seats", message: "review.state.expired" });
    };
    const interval = window.setInterval(update, 1_000);
    return () => window.clearInterval(interval);
  }, [context, receivedAt, reviewState]);

  const revealReady = Boolean(context && !loading && !reviewState);

  useGSAP(
    () => {
      if (!revealReady || reducedMotion) return;
      const targets = page.current?.querySelectorAll<HTMLElement>(
        "[data-review-reveal]",
      );
      if (!targets?.length) return;

      gsap.fromTo(
        targets,
        { autoAlpha: 0, y: 10 },
        {
          autoAlpha: 1,
          clearProps: "opacity,transform,visibility",
          duration: motionDurations.ui,
          ease: gsapEasings.enter,
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

  const extrasHref = buildExtrasHandoffHref({ holdId, query: backQuery });
  const paymentHref = buildPaymentHandoffHref({ holdId, query: backQuery });
  const actionHref = reviewState?.action === "passengers"
    ? passengerHref(backQuery, holdId)
    : reviewState?.action === "extras"
      ? extrasHref
      : seatHref(backQuery, context);
  const actionLabel = reviewState?.action === "passengers" ? "review.action.passengers" : reviewState?.action === "extras" ? "review.action.extras" : "review.action.seats";

  return (
    <main className="relative min-h-screen overflow-x-clip pb-section-md pt-[calc(var(--header-height)+clamp(2rem,5vw,4rem))]">
      <div aria-hidden="true" className="pointer-events-none absolute inset-x-0 top-0 h-[36rem] bg-[radial-gradient(circle_at_70%_0%,rgba(255,212,0,0.07),transparent_32rem)]" />
      <Container className="relative" ref={page}>
        <div data-review-reveal="navigation">
          <Link className="inline-flex min-h-11 items-center gap-2 text-sm text-muted-foreground transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus motion-reduce:transition-none" href={extrasHref}><ArrowLeft aria-hidden="true" />{t("review.back")}</Link>
          <ol aria-label={t("review.progress.label")} className="mt-8 flex flex-wrap items-center gap-3 text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground">
            {["seat", "passenger", "extras"].map((step) => <li className="flex items-center gap-2 text-foreground" key={step}><Check aria-hidden="true" className="size-4 text-brand" />{t(`review.progress.${step}` as TranslationKey)}</li>)}
            <li aria-current="step" className="text-brand">{t("review.progress.review")}</li><li>{t("review.progress.payment")}</li>
          </ol>
        </div>
        <div data-review-reveal="heading">
          <p className="mt-10 text-label text-brand">{t("review.eyebrow")}</p>
          <h1 className="mt-3 text-h1">{t("review.heading")}</h1>
          <p className="mt-5 max-w-2xl text-body-lg text-muted-foreground">{t("review.intro")}</p>
        </div>

        {loading ? <div aria-label={t("review.loading")} className="mt-10 min-h-72 animate-pulse rounded-surface border border-border bg-surface/60 motion-reduce:animate-none" role="status" /> : reviewState ? <div className="review-recovery-panel mt-10 rounded-surface border border-destructive/45 bg-destructive/10 p-7" data-review-recovery={reviewState.action}><p role="alert">{t(reviewState.message)}</p>{reviewState.action === "retry" ? <Button className="mt-5" onClick={() => setAttempt((value) => value + 1)}>{t("review.action.retry")}</Button> : <Link className="mt-5 inline-flex min-h-11 items-center gap-2 font-medium text-brand focus-visible:ring-2 focus-visible:ring-focus" href={actionHref}><ArrowLeft aria-hidden="true" />{t(actionLabel)}</Link>}</div> : context ? <div className="mt-10 grid min-w-0 gap-8 lg:grid-cols-[minmax(0,1fr)_20rem] lg:items-start" data-testid="review-layout"><div data-review-reveal="details"><ReviewDetails context={context} /></div><FareSummary context={context} paymentHref={paymentHref} remainingMilliseconds={remainingMilliseconds} /></div> : null}
      </Container>
    </main>
  );
};

export { ReviewPage };
