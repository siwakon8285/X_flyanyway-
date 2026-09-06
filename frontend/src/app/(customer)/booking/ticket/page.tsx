import { allowlistedRecoveryParams } from "@/components/booking/passengers/passengerRoute";
import { TicketPage } from "@/components/booking/ticket/TicketPage";

type TicketRouteQuery = Record<string, string | string[] | undefined>;

export default async function TicketRoute({
  searchParams,
}: {
  searchParams: Promise<TicketRouteQuery>;
}) {
  const query = await searchParams;
  const holdId = typeof query.holdId === "string" ? query.holdId : "";
  const attemptId = typeof query.attemptId === "string" ? query.attemptId : "";
  const raw = new URLSearchParams();
  Object.entries(query).forEach(([key, value]) => {
    if (typeof value === "string") raw.set(key, value);
  });
  return (
    <TicketPage
      attemptId={attemptId}
      backQuery={allowlistedRecoveryParams(raw.toString()).toString()}
      holdId={holdId}
    />
  );
}
