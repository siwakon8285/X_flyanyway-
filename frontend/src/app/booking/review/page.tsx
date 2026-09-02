import { allowlistedRecoveryParams } from "@/components/booking/passengers/passengerRoute";
import { ReviewPage } from "@/components/booking/review/ReviewPage";

type ReviewRouteQuery = Record<string, string | string[] | undefined>;

export default async function ReviewRoute({ searchParams }: { searchParams: Promise<ReviewRouteQuery> }) {
  const query = await searchParams;
  const holdId = typeof query.holdId === "string" ? query.holdId : "";
  const raw = new URLSearchParams();
  Object.entries(query).forEach(([key, value]) => { if (typeof value === "string") raw.set(key, value); });
  return <ReviewPage backQuery={allowlistedRecoveryParams(raw.toString()).toString()} holdId={holdId} />;
}
