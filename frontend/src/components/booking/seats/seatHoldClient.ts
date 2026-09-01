import type { CabinClass } from "@/components/booking/search/searchTypes";
import {
  API_BASE_URL,
  BookingApiError,
  BookingApiError as SeatHoldApiError,
  requestJson,
  type ApiErrorResponse,
} from "@/components/booking/api/bookingApiClient";

type PassengerCounts = {
  adults: number;
  children: number;
  infants: number;
};

type SeatHold = {
  cabin: CabinClass;
  departureDate: string;
  expiresAt: string;
  flightId: string;
  id: string;
  passengers: PassengerCounts;
  seats: string[];
  serverTime: string;
};

type SeatInventoryStatus =
  | "AVAILABLE"
  | "BOOKED"
  | "HELD_BY_ME"
  | "UNAVAILABLE";

type SeatInventoryItem = {
  columnCode: string;
  position: "aisle" | "middle" | "window";
  rowNumber: number;
  seatNumber: string;
  status: SeatInventoryStatus;
};

type SeatInventory = {
  cabin: CabinClass;
  departureDate: string;
  flightId: string;
  seats: SeatInventoryItem[];
  serverTime: string;
};

type CreateSeatHoldRequest = {
  cabin: CabinClass;
  departureDate: string;
  flightId: string;
  passengers: PassengerCounts;
  seats: string[];
};

const getSeatInventory = ({
  cabin,
  departureDate,
  flightId,
  holdId,
  signal,
}: {
  cabin: CabinClass;
  departureDate: string;
  flightId: string;
  holdId?: string | null;
  signal?: AbortSignal;
}) => {
  const query = new URLSearchParams({ cabin, departure: departureDate });
  if (holdId) query.set("holdId", holdId);
  return requestJson<SeatInventory>(
    `/flights/${encodeURIComponent(flightId)}/seats?${query.toString()}`,
    { signal },
  );
};

const createSeatHold = (request: CreateSeatHoldRequest) =>
  requestJson<SeatHold>("/seat-holds", {
    body: JSON.stringify(request),
    method: "POST",
  });

const replaceSeatHoldSeats = (holdId: string, seats: readonly string[]) =>
  requestJson<SeatHold>(`/seat-holds/${encodeURIComponent(holdId)}`, {
    body: JSON.stringify({ seats }),
    method: "PUT",
  });

const getSeatHold = (holdId: string) =>
  requestJson<SeatHold>(`/seat-holds/${encodeURIComponent(holdId)}`);

const validateSeatHoldForContinue = (holdId: string) =>
  requestJson<SeatHold>(
    `/seat-holds/${encodeURIComponent(holdId)}/validation`,
    { method: "POST" },
  );

const releaseSeatHold = async (holdId: string) => {
  const response = await fetch(
    `${API_BASE_URL}/seat-holds/${encodeURIComponent(holdId)}`,
    { credentials: "include", method: "DELETE" },
  );
  if (!response.ok) {
    const payload = (await response.json().catch(() => ({}))) as ApiErrorResponse;
    throw new BookingApiError({
      code: payload.error?.code ?? "SEAT_HOLD_REQUEST_FAILED",
      message: payload.error?.message ?? "The seat hold request failed.",
      status: response.status,
    });
  }
};

const getRemainingHoldMilliseconds = ({
  clientNow,
  expiresAt,
  serverTime,
  serverTimeReceivedAt = clientNow,
}: {
  clientNow: number;
  expiresAt: string;
  serverTime: string;
  serverTimeReceivedAt?: number;
}) => {
  const serverOffset = Date.parse(serverTime) - serverTimeReceivedAt;
  return Math.max(0, Date.parse(expiresAt) - (clientNow + serverOffset));
};

export {
  createSeatHold,
  getRemainingHoldMilliseconds,
  getSeatHold,
  getSeatInventory,
  releaseSeatHold,
  replaceSeatHoldSeats,
  SeatHoldApiError,
  validateSeatHoldForContinue,
};
export type {
  CreateSeatHoldRequest,
  PassengerCounts,
  SeatHold,
  SeatInventory,
  SeatInventoryItem,
  SeatInventoryStatus,
};
