type ManagedCabin = { available: boolean; priceAmount: number | null; currencyCode: string | null; capacity: number };
type ManagedFlight = {
  id: string; publicId: string; flightNumber: string; originCode: string; destinationCode: string;
  originTimeZone: string; destinationTimeZone: string; operatingDate: string | null;
  departureTime: string | null; arrivalTime: string | null; arrivalDayOffset: number | null;
  aircraftCode: string; status: "SCHEDULED" | "CANCELLED"; business: ManagedCabin; first: ManagedCabin;
  version: number; updatedAt: string;
};
type FlightPage = { items: ManagedFlight[]; total: number; limit: number; offset: number };
type AirportReference = { code: string; name: string; city: string; countryCode: string; countryName: string; timeZone: string };
type FlightReferenceData = { airports: AirportReference[]; aircraft: string[] };
type FlightAuditEntry = { id: string; actorEmail: string; action: "FLIGHT_CREATED" | "FLIGHT_EDITED" | "FLIGHT_CANCELLED"; beforeState: unknown; afterState: unknown; createdAt: string };
type FlightDetail = ManagedFlight & { audit: FlightAuditEntry[] };

export type { AirportReference, FlightAuditEntry, FlightDetail, FlightPage, FlightReferenceData, ManagedCabin, ManagedFlight };
