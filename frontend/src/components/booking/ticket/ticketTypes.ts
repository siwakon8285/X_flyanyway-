type TicketStatus = "ISSUED" | "CANCELLED";

type TicketJourney = {
  flightNumber: string;
  originCode: string;
  destinationCode: string;
  departureDate: string;
  departureTime: string | null;
  arrivalTime: string | null;
  arrivalDayOffset: number | null;
  cabin: string;
};

type TicketPassenger = {
  displayName: string;
};

type Ticket = {
  id: string;
  bookingReference: string;
  ticketNumber: string;
  status: TicketStatus;
  issuedAt: string;
  paymentStatus: string;
  amount: number;
  currencyCode: string;
  journey: TicketJourney;
  passengers: TicketPassenger[];
  seats: string[];
};

type TicketResponse = {
  ticket: Ticket;
  qrToken: string;
};

type TicketVerification = {
  valid: boolean;
  ticketStatus?: TicketStatus;
  flightNumber?: string;
  originCode?: string;
  destinationCode?: string;
  departureDate?: string;
  departureTime?: string;
  seats?: string[];
};

export type {
  Ticket,
  TicketJourney,
  TicketPassenger,
  TicketResponse,
  TicketStatus,
  TicketVerification,
};
