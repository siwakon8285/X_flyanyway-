import type { TranslationKey } from "@/i18n/types";
import type { Locale } from "@/i18n/config";
import { formatStaffDateTime } from "@/i18n/formatters";
import type { BookingStatus, Cabin, FlightStatus, PaymentStatus, RefundStatus, TicketStatus } from "@/lib/admin/bookingTypes";

const statusKeys: Record<BookingStatus | FlightStatus | PaymentStatus | RefundStatus | TicketStatus, TranslationKey> = {
  CONFIRMED: "bookingManagement.status.confirmed", CANCELLED: "bookingManagement.status.cancelled",
  SCHEDULED: "bookingManagement.status.scheduled", ISSUED: "bookingManagement.status.issued",
  SUCCEEDED: "bookingManagement.status.succeeded", PENDING: "bookingManagement.status.pending",
  IN_FLIGHT: "bookingManagement.status.inFlight", PROCESSING: "bookingManagement.status.processing",
  REQUIRES_ATTENTION: "bookingManagement.status.attention", CREATED: "bookingManagement.status.created",
  AWAITING_PAYMENT: "bookingManagement.status.awaitingPayment", FAILED: "bookingManagement.status.failed",
};

const cabinKeys: Record<Cabin, TranslationKey> = {
  business: "bookingManagement.cabinName.business", first: "bookingManagement.cabinName.first",
  economy: "bookingManagement.cabinName.economy", "premium-economy": "bookingManagement.cabinName.premiumEconomy",
};

const isLegacyCabin = (cabin: Cabin) => cabin === "economy" || cabin === "premium-economy";

const formatBookingDateTime = (value: string, locale: Locale) => formatStaffDateTime(value, locale);

const passengerCountKey = (count: number): TranslationKey => count === 1
  ? "bookingManagement.passengersCountOne"
  : "bookingManagement.passengersCount";

export { cabinKeys, formatBookingDateTime, isLegacyCabin, passengerCountKey, statusKeys };
