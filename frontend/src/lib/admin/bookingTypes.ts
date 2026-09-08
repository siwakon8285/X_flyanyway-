type BookingStatus = "CONFIRMED" | "CANCELLED";
type FlightStatus = "SCHEDULED" | "CANCELLED";
type PaymentStatus = "CREATED" | "PROCESSING" | "AWAITING_PAYMENT" | "SUCCEEDED" | "FAILED" | "CANCELLED";
type TicketStatus = "ISSUED" | "CANCELLED";
type RefundStatus = "PENDING" | "IN_FLIGHT" | "PROCESSING" | "SUCCEEDED" | "REQUIRES_ATTENTION";
type Cabin = "business" | "first" | "economy" | "premium-economy";

type BookingListItem = {
  bookingReference: string; leadPassengerName: string; passengerCount: number;
  flightNumber: string; originCode: string; destinationCode: string; travelDate: string;
  departureAt: string | null; cabin: Cabin; bookingStatus: BookingStatus;
  flightStatus: FlightStatus; paymentStatus: PaymentStatus; ticketStatus: TicketStatus;
  refundStatus: RefundStatus | null; bookedAt: string;
};
type BookingListPage = { items: BookingListItem[]; nextOffset: number | null };
type Money = { amount: number; currencyCode: string };
type BookingDetail = {
  bookingReference: string; bookingStatus: BookingStatus; createdAt: string;
  journey: {
    flightNumber: string; originCode: string; destinationCode: string; travelDate: string;
    departureAt: string | null; departureTime: string | null; arrivalDate: string | null;
    arrivalTime: string | null; originTimeZone: string | null; aircraftCode: string;
    cabin: Cabin; flightStatus: FlightStatus;
  };
  passengers: Array<{ ordinal: number; passengerType: "ADULT" | "CHILD" | "INFANT"; displayName: string; gender: "MALE" | "FEMALE" | "UNSPECIFIED" }>;
  seats: string[];
  contact: { phoneCountryCode: string; phoneNumber: string } | null;
  payment: { method: "CARD" | "BITCOIN"; provider: "STRIPE" | "MOCK_BITCOIN"; status: PaymentStatus; amount: Money; succeededAt: string | null };
  ticket: { ticketNumber: string; status: TicketStatus; issuedAt: string; cancelledAt: string | null };
  cancellation: { eligibility: "ELIGIBLE" | "UNAVAILABLE"; cutoffAt: string | null; cancelledAt: string | null; refundStatus: RefundStatus | null; refundAmount: Money | null; refundedAt: string | null };
  audit: Array<{ action: "STAFF_BOOKING_CANCELLED"; actorEmail: string; createdAt: string }>;
};

export type { BookingDetail, BookingListItem, BookingListPage, BookingStatus, Cabin, FlightStatus, PaymentStatus, RefundStatus, TicketStatus };
