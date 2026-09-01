import {
  BookingApiError,
  requestJson,
} from "@/components/booking/api/bookingApiClient";
import type {
  Passenger,
  PassengerContext,
} from "@/components/booking/passengers/passengerTypes";

const passengerPath = (holdId: string) =>
  `/seat-holds/${encodeURIComponent(holdId)}/passengers`;

const getPassengerContext = (holdId: string) =>
  requestJson<PassengerContext>(passengerPath(holdId));

const savePassengerDraft = (holdId: string, passengers: Passenger[]) =>
  requestJson<PassengerContext>(passengerPath(holdId), {
    body: JSON.stringify({ passengers }),
    method: "PUT",
  });

export { BookingApiError, getPassengerContext, savePassengerDraft };
