import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import type { AirportOption } from "@/components/booking/search/searchTypes";
import type { FlightResult } from "@/components/booking/results/flightResultTypes";

const API_URL=process.env.X_FLY_INTERNAL_API_URL??process.env.NEXT_PUBLIC_X_FLY_API_URL??"http://localhost:8080/api/v1";
const paramsFor=(criteria:FlightSearchFormValues)=>new URLSearchParams({origin:criteria.from!.code,destination:criteria.to!.code,departure:criteria.departure,cabin:criteria.cabin});

type PublicAirport = {
  code: string;
  name: string;
  city: string;
  countryCode: string;
  countryName: string;
  timeZone: string;
};

async function fetchPublicAirports(): Promise<AirportOption[] | null> {
  try {
    const response = await fetch(`${API_URL}/airports`, {
      cache: "no-store",
      signal: AbortSignal.timeout(15_000),
    });
    if (!response.ok) return null;
    const airports = (await response.json()) as PublicAirport[];
    return airports.map((airport) => ({
      airport: airport.name,
      city: airport.city,
      code: airport.code,
      country: airport.countryName,
    }));
  } catch {
    return null;
  }
}

async function fetchPublicFlights(criteria:FlightSearchFormValues):Promise<FlightResult[]|null>{
  try{const response=await fetch(`${API_URL}/flights?${paramsFor(criteria)}`,{cache:"no-store",signal:AbortSignal.timeout(15_000)});if(!response.ok)return response.status===422?[]:null;return await response.json() as FlightResult[];}catch{return null;}
}
async function fetchPublicFlightDetail(id:string,criteria:FlightSearchFormValues):Promise<FlightResult|null>{
  try{const query=new URLSearchParams({departure:criteria.departure,cabin:criteria.cabin});const response=await fetch(`${API_URL}/flights/${encodeURIComponent(id)}?${query}`,{cache:"no-store",signal:AbortSignal.timeout(15_000)});if(!response.ok)return null;return await response.json() as FlightResult;}catch{return null;}
}
export { fetchPublicAirports, fetchPublicFlightDetail, fetchPublicFlights };
