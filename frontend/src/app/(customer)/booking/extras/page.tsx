import { allowlistedRecoveryParams } from "@/components/booking/passengers/passengerRoute";
import { redirect } from "next/navigation";

type ExtrasRouteQuery = Record<string, string | string[] | undefined>;

export default async function TravelExtrasRoute({
  searchParams,
}: {
  searchParams: Promise<ExtrasRouteQuery>;
}) {
  const query = await searchParams;
  const holdId = typeof query.holdId === "string" ? query.holdId : "";
  const raw = new URLSearchParams();
  Object.entries(query).forEach(([key, value]) => {
    if (typeof value === "string") raw.set(key, value);
  });
  const recovery = allowlistedRecoveryParams(raw.toString());

  if (holdId) recovery.set("holdId", holdId);
  redirect(`/booking/review?${recovery.toString()}`);
}
