import {
  BookingApiError,
  requestJson,
} from "@/components/booking/api/bookingApiClient";
import type { ReviewContext } from "@/components/booking/review/reviewTypes";

const reviewPath = (holdId: string) =>
  `/seat-holds/${encodeURIComponent(holdId)}/review`;

const getReviewContext = (holdId: string) =>
  requestJson<ReviewContext>(reviewPath(holdId));

export { BookingApiError, getReviewContext };
