"use client";

import { ArrowLeft, Check } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useState } from "react";

import {
  BookingApiError,
  getPassengerContext,
  savePassengerDraft,
} from "@/components/booking/passengers/passengerClient";
import { PassengerForm } from "@/components/booking/passengers/PassengerForm";
import { PassengerSummary } from "@/components/booking/passengers/PassengerSummary";
import { buildExtrasHandoffHref } from "@/components/booking/passengers/passengerRoute";
import { validationKeys } from "@/components/booking/passengers/passengerPresentation";
import type {
  PassengerContext,
  PassengerFieldName,
  PassengerFormValue,
  PassengerValidationCode,
  PassengerValidationError,
} from "@/components/booking/passengers/passengerTypes";
import {
  createEmptyPassenger,
  normalizePassengerDraft,
  validatePassengerDraft,
} from "@/components/booking/passengers/passengerValidation";
import { getRemainingHoldMilliseconds } from "@/components/booking/seats/seatHoldClient";
import { getTodayDateInputValue } from "@/components/booking/search/searchState";
import { Container } from "@/components/layout/Container";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { TranslationKey } from "@/i18n/types";

const passengerFields = new Set<string>([
  "dateOfBirth",
  "email",
  "emergencyContact",
  "familyName",
  "gender",
  "givenName",
  "middleName",
  "nationalityCode",
  "passportIssuingCountryCode",
  "passportNumber",
  "phoneCountryCode",
  "phoneNumber",
  "title",
]);

const toFormValues = (context: PassengerContext): PassengerFormValue[] =>
  context.expectedPassengers.map((slot) => {
    const saved = context.passengers.find((passenger) => passenger.ordinal === slot.ordinal);
    return saved
      ? {
          ...saved,
          middleName: saved.middleName ?? "",
        }
      : createEmptyPassenger(slot);
  });

const lifecycleMessage = (error: BookingApiError): TranslationKey => {
  if (error.code === "HOLD_EXPIRED") return "passengerInformation.state.expired";
  if (error.code === "HOLD_RELEASED") return "passengerInformation.state.released";
  if (["HOLD_UNAUTHORIZED", "HOLD_NOT_FOUND"].includes(error.code)) {
    return "passengerInformation.state.unauthorized";
  }
  return "passengerInformation.state.unavailable";
};

const buildBackHref = (backQuery: string, context: PassengerContext | null) => {
  const params = new URLSearchParams(backQuery);
  const flightId = context?.hold.flightId ?? params.get("flightId");
  params.delete("flightId");
  params.delete("holdId");
  params.delete("seats");
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

const PassengerInformationPage = ({
  backQuery,
  holdId,
}: {
  backQuery: string;
  holdId: string;
}) => {
  const { t } = useLanguage();
  const router = useRouter();
  const [context, setContext] = useState<PassengerContext | null>(null);
  const [values, setValues] = useState<PassengerFormValue[]>([]);
  const [errors, setErrors] = useState<PassengerValidationError[]>([]);
  const [loading, setLoading] = useState(Boolean(holdId));
  const [saving, setSaving] = useState(false);
  const [ready, setReady] = useState(false);
  const [recentlySaved, setRecentlySaved] = useState(false);
  const [stateMessage, setStateMessage] = useState<TranslationKey | null>(
    holdId ? null : "passengerInformation.state.invalid",
  );
  const [remainingMilliseconds, setRemainingMilliseconds] = useState(0);
  const [holdReceivedAt, setHoldReceivedAt] = useState(0);

  const applyContext = useCallback((next: PassengerContext, restoreValues: boolean) => {
    const receivedAt = Date.now();
    setContext(next);
    setHoldReceivedAt(receivedAt);
    setRemainingMilliseconds(
      getRemainingHoldMilliseconds({
        clientNow: receivedAt,
        expiresAt: next.hold.expiresAt,
        serverTime: next.hold.serverTime,
        serverTimeReceivedAt: receivedAt,
      }),
    );
    if (restoreValues) {
      setValues(toFormValues(next));
      setReady(next.readyToContinue);
    }
  }, []);

  useEffect(() => {
    let active = true;
    if (!holdId) return;
    const load = async () => {
      try {
        const next = await getPassengerContext(holdId);
        if (!active) return;
        applyContext(next, true);
      } catch (error) {
        if (!active) return;
        setStateMessage(
          error instanceof BookingApiError
            ? lifecycleMessage(error)
            : "passengerInformation.state.unavailable",
        );
      } finally {
        if (active) setLoading(false);
      }
    };
    void load();
    return () => {
      active = false;
    };
  }, [applyContext, holdId]);

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
      if (remaining === 0) setStateMessage("passengerInformation.state.expired");
    };
    const interval = window.setInterval(update, 1_000);
    return () => window.clearInterval(interval);
  }, [context, holdReceivedAt, stateMessage]);

  useEffect(() => {
    if (!context || stateMessage) return;
    const revalidate = async () => {
      try {
        const next = await getPassengerContext(holdId);
        applyContext(next, false);
      } catch (error) {
        setStateMessage(
          error instanceof BookingApiError
            ? lifecycleMessage(error)
            : "passengerInformation.state.unavailable",
        );
      }
    };
    window.addEventListener("focus", revalidate);
    return () => window.removeEventListener("focus", revalidate);
  }, [applyContext, context, holdId, stateMessage]);

  const handleSave = async () => {
    if (!context || stateMessage || remainingMilliseconds <= 0) return;
    const nextErrors = validatePassengerDraft(
      values,
      context.hold.departureDate,
      getTodayDateInputValue(),
    );
    setErrors(nextErrors);
    if (nextErrors.length > 0) return;

    setSaving(true);
    try {
      const saved = await savePassengerDraft(holdId, normalizePassengerDraft(values));
      applyContext(saved, true);
      setErrors([]);
      setReady(true);
      setRecentlySaved(true);
      router.push(buildExtrasHandoffHref({ holdId, query: backQuery }));
    } catch (error) {
      if (error instanceof BookingApiError) {
        if (["HOLD_EXPIRED", "HOLD_RELEASED", "HOLD_NOT_FOUND", "HOLD_UNAUTHORIZED"].includes(error.code)) {
          setStateMessage(lifecycleMessage(error));
        } else if (error.fieldErrors.length > 0) {
          const mapped = error.fieldErrors.flatMap((fieldError) => {
            if (
              !(fieldError.code in validationKeys) ||
              !passengerFields.has(fieldError.field)
            ) {
              return [];
            }
            return [{
              code: fieldError.code as PassengerValidationCode,
              field: fieldError.field as PassengerFieldName,
              passenger: fieldError.passenger,
            }];
          });
          setErrors(mapped);
        } else {
          setStateMessage("passengerInformation.state.unavailable");
        }
      } else {
        setStateMessage("passengerInformation.state.unavailable");
      }
    } finally {
      setSaving(false);
    }
  };

  const backHref = buildBackHref(backQuery, context);

  return (
    <section className="relative min-h-screen overflow-x-clip pb-section-md pt-[calc(var(--header-height)+clamp(2rem,5vw,4rem))]">
      <div aria-hidden="true" className="pointer-events-none absolute inset-x-0 top-0 h-[34rem] bg-[radial-gradient(circle_at_45%_0%,rgba(255,212,0,0.06),transparent_32rem)]" />
      <Container className="relative">
        <Link className="inline-flex min-h-11 items-center gap-2 text-sm text-muted-foreground hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus" href={backHref}>
          <ArrowLeft aria-hidden="true" />{t("passengerInformation.back")}
        </Link>
        <ol className="mt-8 flex items-center gap-3 text-xs font-semibold uppercase tracking-[0.08em] text-muted-foreground" aria-label={t("passengerInformation.eyebrow")}>
          <li className="flex items-center gap-2 text-foreground"><Check aria-hidden="true" className="size-4 text-brand" />{t("passengerInformation.progress.seat")}</li>
          <li aria-hidden="true">—</li>
          <li aria-current="step" className="text-brand">{t("passengerInformation.progress.passenger")}</li>
          <li aria-hidden="true">—</li>
          <li>{t("passengerInformation.progress.extras")}</li>
        </ol>
        <p className="mt-10 text-label text-brand">{t("passengerInformation.eyebrow")}</p>
        <h1 className="mt-3 text-h1">{t("passengerInformation.heading")}</h1>
        <p className="mt-5 max-w-2xl text-body-lg text-muted-foreground">{t("passengerInformation.intro")}</p>

        {loading ? (
          <div aria-label={t("passengerInformation.loading")} className="mt-12 min-h-56 animate-pulse rounded-surface border border-border bg-surface/60" role="status" />
        ) : stateMessage ? (
          <div className="mt-10 rounded-surface border border-destructive/45 bg-destructive/10 p-7">
            <p className="text-body text-foreground" role="alert">{t(stateMessage)}</p>
            <Link className="mt-5 inline-flex min-h-11 items-center gap-2 font-medium text-brand" href={backHref}>
              <ArrowLeft aria-hidden="true" />{t("passengerInformation.returnSeats")}
            </Link>
          </div>
        ) : context ? (
          <div className="mt-10 grid gap-8 lg:grid-cols-[minmax(0,1fr)_20rem] lg:items-start">
            <PassengerForm
              errors={errors}
              onSave={() => void handleSave()}
              onValuesChange={(next) => {
                setValues(next);
                setErrors([]);
                setReady(false);
                setRecentlySaved(false);
              }}
              ready={ready}
              recentlySaved={recentlySaved}
              saving={saving}
              values={values}
            />
            <PassengerSummary hold={context.hold} remainingMilliseconds={remainingMilliseconds} />
          </div>
        ) : null}
      </Container>
    </section>
  );
};

export { PassengerInformationPage };
