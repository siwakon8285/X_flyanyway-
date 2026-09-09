type TicketStatus = "ISSUED" | "CANCELLED";
type Cabin = "business" | "first" | "economy" | "premium-economy";
type TicketOperationsListItem = {
  ticketNumber:string; bookingReference:string; passengerNames:string[]; flightNumber:string;
  originCode:string; destinationCode:string; travelDate:string; cabin:Cabin; seats:string[]; ticketStatus:TicketStatus;
};
type TicketOperationsPage = { items:TicketOperationsListItem[]; nextOffset:number|null };
type TicketOperationsDetail = {
  ticketNumber:string; ticketStatus:TicketStatus; issuedAt:string; cancelledAt:string|null;
  bookingReference:string; bookingStatus:"CONFIRMED"|"CANCELLED";
  paymentStatus:"CREATED"|"PROCESSING"|"AWAITING_PAYMENT"|"SUCCEEDED"|"FAILED"|"CANCELLED";
  refundStatus:"PENDING"|"IN_FLIGHT"|"PROCESSING"|"SUCCEEDED"|"REQUIRES_ATTENTION"|null;
  journey:{ flightNumber:string; originCode:string; destinationCode:string; travelDate:string; departureAt:string|null; departureTime:string|null; originTimeZone:string|null; cabin:Cabin; flightStatus:"SCHEDULED"|"CANCELLED" };
  passengers:Array<{ ordinal:number; displayName:string; passengerType:"ADULT"|"CHILD"|"INFANT"; gender:"MALE"|"FEMALE"|"UNSPECIFIED"; seat:string|null }>;
};
type PrintableTicketOperationsDetail = TicketOperationsDetail & { qrToken:string; printable:boolean };
export type { Cabin, PrintableTicketOperationsDetail, TicketOperationsDetail, TicketOperationsListItem, TicketOperationsPage, TicketStatus };
