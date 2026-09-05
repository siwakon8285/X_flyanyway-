type BookingStatus = "CONFIRMED" | "CANCELLED";
type PaymentStatus = "CREATED" | "PROCESSING" | "AWAITING_PAYMENT" | "SUCCEEDED" | "FAILED" | "CANCELLED";
type TicketStatus = "ISSUED" | "CANCELLED";
type TravelDocumentStatus = "COMPLETE" | "INCOMPLETE";
type CancellationEligibility = "ELIGIBLE" | "UNAVAILABLE";
type ExtraCategory = "BAGGAGE" | "MEAL" | "ASSISTANCE";

type ManageBooking = {
  bookingReference: string;
  status: BookingStatus;
  journey: {
    flightNumber: string;
    originCode: string;
    destinationCode: string;
    departureDate: string;
    departureTime: string | null;
    departureTimeZone: string | null;
    departureAt: string | null;
    arrivalDate: string | null;
    arrivalTime: string | null;
    cabin: string;
  };
  passengers: Array<{
    ordinal: number;
    displayName: string;
    travelDocumentStatus: TravelDocumentStatus;
  }>;
  seats: string[];
  extras: Array<{
    passengerOrdinal: number;
    productCode: string;
    category: ExtraCategory;
    quantity: number;
  }>;
  payment: {
    status: PaymentStatus;
    amount: { amount: number; currencyCode: string };
  };
  ticket: { ticketNumber: string; status: TicketStatus; issuedAt: string };
  cancellation: { eligibility: CancellationEligibility; cutoffAt: string | null };
};

type ManageBookingLookupInput = {
  bookingReference: string;
  lastName: string;
};

type ManageBookingDetails = ManageBooking & { qrToken: string };

export type {
  ManageBookingDetails,
  BookingStatus,
  CancellationEligibility,
  ExtraCategory,
  ManageBooking,
  ManageBookingLookupInput,
  PaymentStatus,
  TicketStatus,
  TravelDocumentStatus,
};
