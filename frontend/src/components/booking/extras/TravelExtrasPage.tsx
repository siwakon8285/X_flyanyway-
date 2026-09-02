"use client";

import { ArrowLeft, Check } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useRef, useState } from "react";

import {
  BookingApiError,
  getExtrasContext,
  saveExtras,
} from "@/components/booking/extras/extrasClient";
import { ExtrasSummary } from "@/components/booking/extras/ExtrasSummary";
import { IncludedBenefits } from "@/components/booking/extras/IncludedBenefits";
import { PassengerExtras } from "@/components/booking/extras/PassengerExtras";
import type {
  ExtraSelectionInput,
  ExtrasContext,
} from "@/components/booking/extras/extrasTypes";
import {
  allowlistedRecoveryParams,
  buildReviewHandoffHref,
} from "@/components/booking/passengers/passengerRoute";
import { getRemainingHoldMilliseconds } from "@/components/booking/seats/seatHoldClient";
import { Container } from "@/components/layout/Container";
import { Button } from "@/components/ui/Button";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";
import { motionDurations } from "@/lib/motion/durations";
import { gsapEasings } from "@/lib/motion/easing";
import { gsap, useGSAP } from "@/lib/motion/gsap";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { cn } from "@/lib/utils/cn";

const lifecycleMessage = (error: BookingApiError): TranslationKey | null => {
  if (error.code === "HOLD_EXPIRED") return "travelExtras.state.expired";
  if (error.code === "HOLD_RELEASED") return "travelExtras.state.released";
  if (error.code === "PASSENGERS_NOT_READY") return "travelExtras.state.passengersNotReady";
  if (error.code === "SEAT_COUNT_MISMATCH") return "travelExtras.state.seatsIncomplete";
  if (["HOLD_UNAUTHORIZED", "HOLD_NOT_FOUND"].includes(error.code)) {
    return "travelExtras.state.unauthorized";
  }
  if (error.code === "HOLD_CONSUMED") return "travelExtras.state.consumed";
  return null;
};

const buildPassengerHref = (backQuery: string, holdId: string) => {
  const params = allowlistedRecoveryParams(backQuery);
  params.set("holdId", holdId);
  return `/booking/passengers?${params.toString()}`;
};

const buildSeatRecoveryHref = (
  backQuery: string,
  context: ExtrasContext | null,
) => {
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
  return flightId
    ? `/flights/${encodeURIComponent(flightId)}/seats?${params.toString()}`
    : `/flights?${params.toString()}`;
};

const TravelExtrasPage = ({
  backQuery,
  holdId,
}: {
  backQuery: string;
  holdId: string;
}) => {
  const { t } = useLanguage();
  const router = useRouter();
  const page = useRef<HTMLDivElement>(null);
  const reducedMotion = useReducedMotion();
  const [context, setContext] = useState<ExtrasContext | null>(null);
  const [selections, setSelections] = useState<ExtraSelectionInput[]>([]);
  const [activeIndex, setActiveIndex] = useState(0);
  const [loading, setLoading] = useState(Boolean(holdId));
  const [saving, setSaving] = useState(false);
  const [recentlySaved, setRecentlySaved] = useState(false);
  const [loadFailed, setLoadFailed] = useState(false);
  const [loadAttempt, setLoadAttempt] = useState(0);
  const [saveFailed, setSaveFailed] = useState(false);
  const [stateMessage, setStateMessage] = useState<TranslationKey | null>(
    holdId ? null : "travelExtras.state.invalid",
  );
  const [remainingMilliseconds, setRemainingMilliseconds] = useState(0);
  const [holdReceivedAt, setHoldReceivedAt] = useState(0);

  const applyContext = useCallback((next: ExtrasContext) => {
    const receivedAt = Date.now();
    setContext(next);
    setSelections(
      next.selections.map(({ passengerOrdinal, productCode, quantity }) => ({
        passengerOrdinal,
        productCode,
        quantity,
      })),
    );
    setHoldReceivedAt(receivedAt);
    setRemainingMilliseconds(
      getRemainingHoldMilliseconds({
        clientNow: receivedAt,
        expiresAt: next.hold.expiresAt,
        serverTime: next.hold.serverTime,
        serverTimeReceivedAt: receivedAt,
      }),
    );
  }, []);

  useEffect(() => {
    let active = true;
    if (!holdId) return;
    const load = async () => {
      setLoading(true);
      setLoadFailed(false);
      try {
        const next = await getExtrasContext(holdId);
        if (active) {
          applyContext(next);
          setStateMessage(null);
        }
      } catch (error) {
        if (!active) return;
        const terminal =
          error instanceof BookingApiError ? lifecycleMessage(error) : null;
        if (terminal) setStateMessage(terminal);
        else setLoadFailed(true);
      } finally {
        if (active) setLoading(false);
      }
    };
    void load();
    return () => {
      active = false;
    };
  }, [applyContext, holdId, loadAttempt]);

  useEffect(() => {
    if (!context || holdReceivedAt === 0 || stateMessage) return;
    const update = () => {
      const remaining = getRemainingHoldMilliseconds({
        clientNow: Date.now(),
        expiresAt: context.hold.expiresAt,
        serverTime: context.hold.serverTime,
        serverTimeReceivedAt: holdReceivedAt,
      });
      setRemainingMilliseconds(remaining);
      if (remaining === 0) setStateMessage("travelExtras.state.expired");
    };
    const interval = window.setInterval(update, 1_000);
    return () => window.clearInterval(interval);
  }, [context, holdReceivedAt, stateMessage]);

  const revealReady = Boolean(
    context && !loading && !loadFailed && !stateMessage,
  );

  useGSAP(
    () => {
      if (!revealReady || reducedMotion) return;
      const targets = page.current?.querySelectorAll<HTMLElement>(
        "[data-extras-reveal]",
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

  const changeSelections = (next: ExtraSelectionInput[]) => {
    setSelections(next);
    setRecentlySaved(false);
    setSaveFailed(false);
    setContext((current) =>
      current ? { ...current, readyToContinue: false, savedAt: null } : current,
    );
  };

  const handleSave = async () => {
    if (!context || stateMessage || remainingMilliseconds <= 0) return;
    setSaving(true);
    setSaveFailed(false);
    try {
      const saved = await saveExtras(holdId, selections);
      applyContext(saved);
      setRecentlySaved(true);
      const receivedAt = Date.now();
      const savedRemaining = getRemainingHoldMilliseconds({
        clientNow: receivedAt,
        expiresAt: saved.hold.expiresAt,
        serverTime: saved.hold.serverTime,
        serverTimeReceivedAt: receivedAt,
      });
      if (saved.readyToContinue && savedRemaining > 0) {
        router.push(buildReviewHandoffHref({ holdId, query: backQuery }));
      }
    } catch (error) {
      const terminal =
        error instanceof BookingApiError ? lifecycleMessage(error) : null;
      if (terminal) setStateMessage(terminal);
      else setSaveFailed(true);
    } finally {
      setSaving(false);
    }
  };

  const totalAmount = context
    ? selections.reduce((total, selection) => {
        const product = context.catalog.products.find(
          (item) => item.code === selection.productCode,
        );
        return total + (product?.unitPrice.amount ?? 0) * selection.quantity;
      }, 0)
    : 0;
  const passenger = context?.passengers[activeIndex];
  const seatRecoveryHref = buildSeatRecoveryHref(backQuery, context);

  return (
    <main className="relative min-h-screen overflow-x-clip pb-section-md pt-[calc(var(--header-height)+clamp(2rem,5vw,4rem))]">
      <div aria-hidden="true" className="pointer-events-none absolute inset-x-0 top-0 h-[34rem] bg-[radial-gradient(circle_at_45%_0%,rgba(255,212,0,0.06),transparent_32rem)]" />
      <Container className="relative" ref={page}>
        <div data-extras-reveal>
          <Link className="group/back inline-flex min-h-11 items-center gap-2 text-sm text-muted-foreground transition-colors duration-200 hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus motion-reduce:transition-none" href={buildPassengerHref(backQuery, holdId)}>
            <ArrowLeft aria-hidden="true" className="transition-transform duration-200 motion-safe:group-hover/back:-translate-x-0.5 motion-reduce:transition-none" />
            {t("travelExtras.back")}
          </Link>
          <ol className="mt-8 flex flex-wrap items-center gap-3 text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground" aria-label={t("travelExtras.progress.label")}> 
            <li className="flex items-center gap-2 text-foreground"><Check aria-hidden="true" className="size-4 text-brand" />{t("travelExtras.progress.seat")}</li>
            <li aria-hidden="true">—</li>
            <li className="flex items-center gap-2 text-foreground"><Check aria-hidden="true" className="size-4 text-brand" />{t("travelExtras.progress.passenger")}</li>
            <li aria-hidden="true">—</li>
            <li aria-current="step" className="text-brand">{t("travelExtras.progress.extras")}</li>
            <li aria-hidden="true">—</li>
            <li>{t("travelExtras.progress.review")}</li>
          </ol>
        </div>
        <div data-extras-reveal>
          <p className="mt-10 text-label text-brand">{t("travelExtras.eyebrow")}</p>
          <h1 className="mt-3 text-h1">{t("travelExtras.heading")}</h1>
          <p className="mt-5 max-w-2xl text-body-lg text-muted-foreground">{t("travelExtras.intro")}</p>
        </div>

        {loading ? (
          <div aria-label={t("travelExtras.loading")} className="mt-12 min-h-56 animate-pulse rounded-surface border border-border bg-surface/60 motion-reduce:animate-none" role="status" />
        ) : stateMessage ? (
          <div className="mt-10 rounded-surface border border-destructive/45 bg-destructive/10 p-7 motion-safe:animate-[extras-panel-in_320ms_ease-out_1] motion-reduce:animate-none">
            <p className="text-body text-foreground" role="alert">{t(stateMessage)}</p>
            <Link className="mt-5 inline-flex min-h-11 items-center gap-2 font-medium text-brand" href={seatRecoveryHref}>
              <ArrowLeft aria-hidden="true" />{t("travelExtras.returnSeats")}
            </Link>
          </div>
        ) : loadFailed ? (
          <div className="mt-10 rounded-surface border border-destructive/45 bg-destructive/10 p-7 motion-safe:animate-[extras-panel-in_320ms_ease-out_1] motion-reduce:animate-none">
            <p className="text-body text-foreground" role="alert">{t("travelExtras.state.unavailable")}</p>
            <Button className="mt-5" onClick={() => setLoadAttempt((attempt) => attempt + 1)}>
              {t("travelExtras.retry")}
            </Button>
          </div>
        ) : context && passenger ? (
          <div className="mt-10 grid gap-8 lg:grid-cols-[minmax(0,1fr)_20rem] lg:items-start">
            <div className="min-w-0">
              <IncludedBenefits catalog={context.catalog} />
              <div data-extras-reveal>
              <nav aria-label={t("travelExtras.passengerNav")} className="mt-8 flex gap-2 overflow-x-auto pb-2">
                {context.passengers.map((item, index) => {
                  const type = t(
                    item.passengerType === "ADULT"
                      ? "passengerInformation.passenger.adult"
                      : item.passengerType === "CHILD"
                        ? "passengerInformation.passenger.child"
                        : "passengerInformation.passenger.infant",
                  );
                  const label = t("passengerInformation.passenger.label", { ordinal: item.ordinal, type });
                  return (
                    <button
                      aria-current={index === activeIndex ? "step" : undefined}
                      className={cn(
                        "min-h-11 shrink-0 rounded-control border px-4 text-sm font-medium outline-none transition-[border-color,background-color,color,transform] duration-200 focus-visible:ring-2 focus-visible:ring-focus motion-safe:hover:-translate-y-0.5 motion-reduce:transition-none",
                        index === activeIndex
                          ? "border-brand bg-brand/10 text-brand"
                          : "border-border bg-surface text-muted-foreground",
                      )}
                      key={item.ordinal}
                      onClick={() => setActiveIndex(index)}
                      type="button"
                    >
                      {label}
                    </button>
                  );
                })}
              </nav>
              <PassengerExtras
                onChange={changeSelections}
                passenger={passenger}
                products={context.catalog.products}
                selections={selections}
              />
              {saveFailed ? (
                <p className="mt-6 rounded-control border border-destructive/45 bg-destructive/10 p-4 text-sm text-foreground" role="alert">
                  {t("travelExtras.saveFailed")}
                </p>
              ) : null}
              {recentlySaved ? (
                <p className="mt-6 flex items-center gap-2 rounded-control border border-brand/40 bg-brand/10 p-4 text-sm font-medium text-brand motion-safe:animate-[extras-panel-in_320ms_ease-out_1] motion-reduce:animate-none" role="status">
                  <Check aria-hidden="true" className="motion-safe:animate-[extras-check-in_220ms_ease-out_1] motion-reduce:animate-none" />{t("travelExtras.savedReady")}
                </p>
              ) : null}
              <Button
                className="mt-6 w-full transition-[background-color,box-shadow,transform] duration-200 data-[save-state=saved]:shadow-[0_10px_28px_rgb(255_212_0/0.16)] motion-safe:hover:-translate-y-0.5 motion-safe:active:translate-y-0 motion-safe:active:scale-[0.99] motion-reduce:transition-none sm:w-auto"
                data-save-state={recentlySaved ? "saved" : "dirty"}
                loading={saving}
                onClick={() => void handleSave()}
                size="lg"
              >
                {recentlySaved && !saving ? (
                  <Check aria-hidden="true" className="motion-safe:animate-[extras-check-in_220ms_ease-out_1] motion-reduce:animate-none" />
                ) : null}
                {saving
                  ? t("travelExtras.saving")
                  : recentlySaved
                    ? t("travelExtras.saved")
                    : t("travelExtras.save")}
              </Button>
              </div>
            </div>
            <ExtrasSummary
              context={context}
              remainingMilliseconds={remainingMilliseconds}
              selections={selections}
              totalAmount={totalAmount}
            />
          </div>
        ) : null}
      </Container>
    </main>
  );
};

export { TravelExtrasPage };
