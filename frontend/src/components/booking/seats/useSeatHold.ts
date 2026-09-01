"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useRouter } from "next/navigation";

import type { SeatSelectionRequest } from "@/components/booking/detail/flightDetailUtils";
import type { CabinSeatMap } from "@/components/booking/seats/seatMapTypes";
import {
  createSeatHold,
  getRemainingHoldMilliseconds,
  getSeatHold,
  getSeatInventory,
  releaseSeatHold,
  replaceSeatHoldSeats,
  SeatHoldApiError,
  validateSeatHoldForContinue,
  type SeatHold,
} from "@/components/booking/seats/seatHoldClient";
import { applySeatInventory, getHeldByMeSeats } from "@/components/booking/seats/seatHoldState";
import { buildPassengerInformationHref } from "@/components/booking/passengers/passengerRoute";
import { toggleSeatSelection } from "@/components/booking/seats/seatMapUtils";
import type { TranslationKey } from "@/i18n/types";

const POLL_INTERVAL_MS = 10_000;

const isExpiredError = (error: unknown) =>
  error instanceof SeatHoldApiError &&
  ["HOLD_EXPIRED", "HOLD_RELEASED", "HOLD_NOT_FOUND"].includes(error.code);

const useSeatHold = ({
  initialMap,
  request,
  requiredSeatCount,
}: {
  initialMap: CabinSeatMap;
  request: SeatSelectionRequest;
  requiredSeatCount: number;
}) => {
  const router = useRouter();
  const storageKey = useMemo(
    () =>
      `x-fly:seat-hold:${request.flight.id}:${request.criteria.departure}:${request.selectedCabin}`,
    [request.criteria.departure, request.flight.id, request.selectedCabin],
  );
  const [map, setMap] = useState(initialMap);
  const [selectedSeatIds, setSelectedSeatIds] = useState<Set<string>>(() => new Set());
  const [pendingSeatIds, setPendingSeatIds] = useState<Set<string>>(() => new Set());
  const [hold, setHold] = useState<SeatHold | null>(null);
  const [holdReceivedAt, setHoldReceivedAt] = useState<number | null>(null);
  const [remainingMilliseconds, setRemainingMilliseconds] = useState<number | null>(null);
  const [limitReached, setLimitReached] = useState(false);
  const [mutationPending, setMutationPending] = useState(false);
  const [continuePending, setContinuePending] = useState(false);
  const [messageKey, setMessageKey] = useState<TranslationKey | null>(null);
  const holdIdRef = useRef<string | null>(null);

  const clearHold = useCallback(() => {
    holdIdRef.current = null;
    setHold(null);
    setHoldReceivedAt(null);
    setRemainingMilliseconds(null);
    setSelectedSeatIds(new Set());
    window.sessionStorage.removeItem(storageKey);
  }, [storageKey]);

  const applyHold = useCallback(
    (nextHold: SeatHold) => {
      const receivedAt = Date.now();
      holdIdRef.current = nextHold.id;
      setHold(nextHold);
      setHoldReceivedAt(receivedAt);
      setRemainingMilliseconds(
        getRemainingHoldMilliseconds({
          clientNow: receivedAt,
          expiresAt: nextHold.expiresAt,
          serverTime: nextHold.serverTime,
          serverTimeReceivedAt: receivedAt,
        }),
      );
      setSelectedSeatIds(new Set(nextHold.seats));
      window.sessionStorage.setItem(storageKey, nextHold.id);
    },
    [storageKey],
  );

  const refreshInventory = useCallback(
    async (ownerHoldId: string | null = holdIdRef.current) => {
      if (typeof fetch !== "function") return;
      try {
        const inventory = await getSeatInventory({
          cabin: request.selectedCabin,
          departureDate: request.criteria.departure,
          flightId: request.flight.id,
          holdId: ownerHoldId,
        });
        setMap(applySeatInventory(initialMap, inventory));
        if (ownerHoldId) {
          const ownedSeats = getHeldByMeSeats(inventory);
          if (ownedSeats.length > 0) setSelectedSeatIds(new Set(ownedSeats));
        }
      } catch (error) {
        if (isExpiredError(error)) {
          clearHold();
          setMessageKey("seatMap.holdExpired");
          const inventory = await getSeatInventory({
            cabin: request.selectedCabin,
            departureDate: request.criteria.departure,
            flightId: request.flight.id,
          });
          setMap(applySeatInventory(initialMap, inventory));
        } else if (!(error instanceof DOMException && error.name === "AbortError")) {
          setMessageKey("seatMap.holdServiceUnavailable");
        }
      }
    },
    [clearHold, initialMap, request.criteria.departure, request.flight.id, request.selectedCabin],
  );

  useEffect(() => {
    let active = true;
    const restore = async () => {
      const savedHoldId = window.sessionStorage.getItem(storageKey);
      if (!savedHoldId) {
        await refreshInventory(null);
        return;
      }

      try {
        const restored = await getSeatHold(savedHoldId);
        if (!active) return;
        const matchesSelection =
          restored.flightId === request.flight.id &&
          restored.departureDate === request.criteria.departure &&
          restored.cabin === request.selectedCabin;
        if (!matchesSelection) {
          clearHold();
          await refreshInventory(null);
          return;
        }
        applyHold(restored);
        await refreshInventory(restored.id);
      } catch (error) {
        if (!active) return;
        clearHold();
        if (isExpiredError(error)) setMessageKey("seatMap.holdExpired");
        await refreshInventory(null);
      }
    };
    void restore();

    return () => {
      active = false;
    };
  }, [applyHold, clearHold, refreshInventory, request.criteria.departure, request.flight.id, request.selectedCabin, storageKey]);

  useEffect(() => {
    const poll = () => void refreshInventory();
    const interval = window.setInterval(poll, POLL_INTERVAL_MS);
    window.addEventListener("focus", poll);
    return () => {
      window.clearInterval(interval);
      window.removeEventListener("focus", poll);
    };
  }, [refreshInventory]);

  useEffect(() => {
    if (!hold || holdReceivedAt === null) return;
    const updateCountdown = () => {
      const remaining = getRemainingHoldMilliseconds({
        clientNow: Date.now(),
        expiresAt: hold.expiresAt,
        serverTime: hold.serverTime,
        serverTimeReceivedAt: holdReceivedAt,
      });
      setRemainingMilliseconds(remaining);
      if (remaining === 0) {
        clearHold();
        setMessageKey("seatMap.holdExpired");
        void refreshInventory(null);
      }
    };
    updateCountdown();
    const interval = window.setInterval(updateCountdown, 1_000);
    return () => window.clearInterval(interval);
  }, [clearHold, hold, holdReceivedAt, refreshInventory]);

  const handleToggle = useCallback(
    async (seatId: string) => {
      if (mutationPending) return;
      const previous = new Set(selectedSeatIds);
      const result = toggleSeatSelection({ requiredSeatCount, seatId, selectedSeatIds });
      setLimitReached(result.limitReached);
      if (result.limitReached) return;

      const desiredSeats = [...result.selectedSeatIds];
      setMessageKey(null);
      setSelectedSeatIds(result.selectedSeatIds);
      setPendingSeatIds(new Set([seatId]));
      setMutationPending(true);

      try {
        const activeHoldId = holdIdRef.current;
        if (desiredSeats.length === 0 && activeHoldId) {
          await releaseSeatHold(activeHoldId);
          clearHold();
          await refreshInventory(null);
        } else {
          const nextHold = activeHoldId
            ? await replaceSeatHoldSeats(activeHoldId, desiredSeats)
            : await createSeatHold({
                cabin: request.selectedCabin,
                departureDate: request.criteria.departure,
                flightId: request.flight.id,
                passengers: request.criteria.passengers,
                seats: desiredSeats,
              });
          applyHold(nextHold);
          setMessageKey("seatMap.holdConfirmed");
          await refreshInventory(nextHold.id);
        }
      } catch (error) {
        setSelectedSeatIds(previous);
        if (error instanceof SeatHoldApiError && error.code === "SEAT_UNAVAILABLE") {
          setMessageKey("seatMap.holdConflict");
          await refreshInventory(holdIdRef.current);
        } else if (isExpiredError(error)) {
          clearHold();
          setMessageKey("seatMap.holdExpired");
          await refreshInventory(null);
        } else {
          setMessageKey("seatMap.holdServiceUnavailable");
        }
      } finally {
        setPendingSeatIds(new Set());
        setMutationPending(false);
      }
    },
    [applyHold, clearHold, mutationPending, refreshInventory, request.criteria.departure, request.criteria.passengers, request.flight.id, request.selectedCabin, requiredSeatCount, selectedSeatIds],
  );

  const handleContinue = useCallback(async () => {
    const activeHoldId = holdIdRef.current;
    if (!activeHoldId || selectedSeatIds.size !== requiredSeatCount) return;
    setContinuePending(true);
    setMessageKey(null);
    try {
      const validated = await validateSeatHoldForContinue(activeHoldId);
      if (validated.seats.length !== requiredSeatCount) {
        applyHold(validated);
        setMessageKey("seatMap.holdIncomplete");
        return;
      }
      applyHold(validated);
      router.push(
        buildPassengerInformationHref({
          flightId: request.flight.id,
          holdId: validated.id,
          query: request.query,
          selectedCabin: request.selectedCabin,
        }),
      );
    } catch (error) {
      if (isExpiredError(error)) {
        clearHold();
        setMessageKey("seatMap.holdExpired");
        await refreshInventory(null);
      } else {
        setMessageKey("seatMap.holdServiceUnavailable");
      }
    } finally {
      setContinuePending(false);
    }
  }, [applyHold, clearHold, refreshInventory, request.flight.id, request.query, request.selectedCabin, requiredSeatCount, router, selectedSeatIds.size]);

  return {
    continuePending,
    handleContinue,
    handleToggle,
    hold,
    limitReached,
    map,
    messageKey,
    mutationPending,
    pendingSeatIds,
    remainingMilliseconds,
    selectedSeatIds,
  };
};

export { useSeatHold };
