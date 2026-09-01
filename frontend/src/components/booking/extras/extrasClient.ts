import {
  BookingApiError,
  requestJson,
} from "@/components/booking/api/bookingApiClient";
import type {
  ExtraSelectionInput,
  ExtrasContext,
} from "@/components/booking/extras/extrasTypes";

const extrasPath = (holdId: string) =>
  `/seat-holds/${encodeURIComponent(holdId)}/extras`;

const getExtrasContext = (holdId: string) =>
  requestJson<ExtrasContext>(extrasPath(holdId));

const saveExtras = (holdId: string, selections: ExtraSelectionInput[]) =>
  requestJson<ExtrasContext>(extrasPath(holdId), {
    body: JSON.stringify({ selections }),
    method: "PUT",
  });

export { BookingApiError, getExtrasContext, saveExtras };
