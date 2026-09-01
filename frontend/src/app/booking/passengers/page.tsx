import { PassengerInformationPage } from "@/components/booking/passengers/PassengerInformationPage";

type PassengerRouteQuery = Record<string, string | string[] | undefined>;

const recoveryKeys = new Set([
  "adults",
  "cabin",
  "children",
  "departure",
  "flightId",
  "from",
  "infants",
  "return",
  "selectedCabin",
  "to",
  "trip",
]);

export default async function PassengerInformationRoute({
  searchParams,
}: {
  searchParams: Promise<PassengerRouteQuery>;
}) {
  const query = await searchParams;
  const holdId = typeof query.holdId === "string" ? query.holdId : "";
  const backParams = new URLSearchParams();

  Object.entries(query).forEach(([key, value]) => {
    if (recoveryKeys.has(key) && typeof value === "string") {
      backParams.set(key, value);
    }
  });

  return (
    <PassengerInformationPage
      backQuery={backParams.toString()}
      holdId={holdId}
    />
  );
}
