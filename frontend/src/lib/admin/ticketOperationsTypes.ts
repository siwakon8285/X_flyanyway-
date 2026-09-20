type TicketStatus = "ISSUED" | "CANCELLED";
type Cabin = "business" | "first" | "economy" | "premium-economy";
type TicketOperationsListItem = {
  ticketNumber:string; bookingReference:string; passengerNames:string[]; flightNumber:string;
  originCode:string; destinationCode:string; travelDate:string; cabin:Cabin; seats:string[]; ticketStatus:TicketStatus;
};
type TicketOperationsPage = { items:TicketOperationsListItem[]; nextOffset:number|null };
type CheckInStatus = "READY"|"CHECKED_IN"|"UNAVAILABLE";
type CheckInUnavailableReason = "TOO_EARLY"|"ALREADY_DEPARTED"|"TICKET_CANCELLED"|"FLIGHT_CANCELLED"|"NO_SEAT_ASSIGNMENT"|"BOOKING_INVALID";
type BoardingPassSummary = { id:string; seat:string; cabin:string; checkedInAt:string; issuedAt:string; validForTravel:boolean };
type TicketOperationsDetail = {
  ticketNumber:string; ticketStatus:TicketStatus; issuedAt:string; cancelledAt:string|null;
  bookingReference:string; bookingStatus:"CONFIRMED"|"CANCELLED";
  paymentStatus:"CREATED"|"PROCESSING"|"AWAITING_PAYMENT"|"SUCCEEDED"|"FAILED"|"CANCELLED";
  refundStatus:"PENDING"|"IN_FLIGHT"|"PROCESSING"|"SUCCEEDED"|"REQUIRES_ATTENTION"|null;
  journey:{ flightNumber:string; originCode:string; destinationCode:string; travelDate:string; departureAt:string|null; departureTime:string|null; originTimeZone:string|null; cabin:Cabin; flightStatus:"SCHEDULED"|"CANCELLED" };
  passengers:Array<{ ordinal:number; displayName:string; passengerType:"ADULT"|"CHILD"|"INFANT"; gender:"MALE"|"FEMALE"|"UNSPECIFIED"; seat:string|null; checkIn:{status:CheckInStatus; reason?:CheckInUnavailableReason}; boardingPass:BoardingPassSummary|null }>;
};
type PrintableTicketOperationsDetail = TicketOperationsDetail & { qrToken:string; printable:boolean };
type BoardingPassDocument = { boardingPassId:string; ticketNumber:string; bookingReference:string; passengerOrdinal:number; passengerName:string; flightNumber:string; originCode:string; destinationCode:string; departureAt:string; departureTime:string; originTimeZone:string; seat:string; cabin:string; checkedInAt:string; issuedAt:string; validForTravel:boolean; invalidReason?:"EXPIRED"|"TICKET_CANCELLED"|"FLIGHT_CANCELLED"|"BOOKING_INVALID"; qrToken:string };
type BoardingPassVerification = { valid:boolean; boardingPassId?:string; invalidReason?:"EXPIRED"|"TICKET_CANCELLED"|"FLIGHT_CANCELLED"|"BOOKING_INVALID"; flightNumber?:string; originCode?:string; destinationCode?:string; departureAt?:string; originTimeZone?:string; seat?:string; cabin?:string };
export type { BoardingPassDocument, BoardingPassSummary, BoardingPassVerification, Cabin, CheckInStatus, CheckInUnavailableReason, PrintableTicketOperationsDetail, TicketOperationsDetail, TicketOperationsListItem, TicketOperationsPage, TicketStatus };
