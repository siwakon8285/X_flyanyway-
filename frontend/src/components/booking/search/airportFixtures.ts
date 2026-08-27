import type { AirportOption } from "@/components/booking/search/searchTypes";

const AIRPORT_FIXTURES = [
  {
    airport: "Suvarnabhumi Airport",
    city: "Bangkok",
    code: "BKK",
    country: "Thailand",
  },
  {
    airport: "Haneda Airport",
    city: "Tokyo",
    code: "HND",
    country: "Japan",
  },
  {
    airport: "Heathrow Airport",
    city: "London",
    code: "LHR",
    country: "United Kingdom",
  },
  {
    airport: "Dubai International Airport",
    city: "Dubai",
    code: "DXB",
    country: "United Arab Emirates",
  },
  {
    airport: "John F. Kennedy International Airport",
    city: "New York",
    code: "JFK",
    country: "United States",
  },
] as const satisfies readonly AirportOption[];

export { AIRPORT_FIXTURES };
