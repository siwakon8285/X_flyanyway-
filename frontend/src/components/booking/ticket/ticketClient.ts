import {
  BookingApiError,
  requestJson,
} from "@/components/booking/api/bookingApiClient";
import type {
  TicketResponse,
  TicketVerification,
} from "@/components/booking/ticket/ticketTypes";

const ticketPath = (holdId: string, attemptId: string) =>
  `/seat-holds/${encodeURIComponent(holdId)}/payment-attempts/${encodeURIComponent(attemptId)}/ticket`;

const verifyPath = (token: string) =>
  `/tickets/verify/${encodeURIComponent(token)}`;

const getTicket = (holdId: string, attemptId: string) =>
  requestJson<TicketResponse>(ticketPath(holdId, attemptId));

const verifyTicket = (token: string) =>
  requestJson<TicketVerification>(verifyPath(token));

export {
  BookingApiError,
  getTicket,
  verifyTicket,
};
export {
  buildTicketVerificationUrl,
  resolveFrontendOrigin,
} from "@/components/booking/ticket/ticketVerificationUrl";
