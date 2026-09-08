import { HomePage } from "@/components/home/HomePage";
import { fetchPublicAirports } from "@/lib/flights/publicFlightBackend";

const HomeRoute = async () => (
  <HomePage airports={(await fetchPublicAirports()) ?? []} />
);

export default HomeRoute;
