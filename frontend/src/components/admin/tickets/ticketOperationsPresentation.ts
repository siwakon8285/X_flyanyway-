import type { TranslationKey } from "@/i18n/types";
import type { ticketOperationsEn } from "@/i18n/locales/ticketOperations";
import type { TicketOperationsDetail } from "@/lib/admin/ticketOperationsTypes";

type TicketOperationsStatus =
  | TicketOperationsDetail["ticketStatus"]
  | TicketOperationsDetail["bookingStatus"]
  | TicketOperationsDetail["paymentStatus"]
  | NonNullable<TicketOperationsDetail["refundStatus"]>
  | TicketOperationsDetail["journey"]["flightStatus"];

const statusNameByValue: Record<TicketOperationsStatus, keyof typeof ticketOperationsEn.status> = {
  ISSUED: "issued",
  CANCELLED: "cancelled",
  CONFIRMED: "confirmed",
  SCHEDULED: "scheduled",
  SUCCEEDED: "succeeded",
  CREATED: "created",
  PROCESSING: "processing",
  AWAITING_PAYMENT: "awaitingPayment",
  FAILED: "failed",
  PENDING: "pending",
  IN_FLIGHT: "inFlight",
  REQUIRES_ATTENTION: "attention",
};

const statusKey = (status: TicketOperationsStatus): TranslationKey =>
  `ticketOperations.status.${statusNameByValue[status]}` as TranslationKey;

const cabinKey = (cabin: TicketOperationsDetail["journey"]["cabin"]): TranslationKey =>
  `ticketOperations.cabinName.${cabin === "premium-economy" ? "premiumEconomy" : cabin}` as TranslationKey;

export { cabinKey, statusKey };
