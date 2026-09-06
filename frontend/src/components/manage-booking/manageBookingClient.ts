import { requestJson } from "@/components/booking/api/bookingApiClient";
import type {
  ManageBooking,
  ManageBookingDetails,
  ManageBookingLookupInput,
} from "@/components/manage-booking/manageBookingTypes";
import type { TicketResponse } from "@/components/booking/ticket/ticketTypes";

const getCurrentManageBooking = () =>
  requestJson<ManageBookingDetails>("/manage-booking/current");

const lookupManageBooking = (input: ManageBookingLookupInput) =>
  requestJson<ManageBooking>("/manage-booking/lookup", {
    method: "POST",
    body: JSON.stringify(input),
  });

const getCurrentManageBookingTicket = () =>
  requestJson<TicketResponse>("/manage-booking/current/ticket");

const cancelCurrentManageBooking = () =>
  requestJson<{ refundStatus: string }>("/manage-booking/current/cancel", {
    method: "POST",
    headers: { "X-X-Fly-CSRF": "1" },
  });

export {
  getCurrentManageBooking,
  getCurrentManageBookingTicket,
  lookupManageBooking,
  cancelCurrentManageBooking,
};
