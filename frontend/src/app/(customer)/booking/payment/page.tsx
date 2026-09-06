import { allowlistedRecoveryParams } from "@/components/booking/passengers/passengerRoute";
import { PaymentPage } from "@/components/booking/payment/PaymentPage";

type PaymentRouteQuery = Record<string, string | string[] | undefined>;

export default async function PaymentRoute({
  searchParams,
}: {
  searchParams: Promise<PaymentRouteQuery>;
}) {
  const query = await searchParams;
  const holdId = typeof query.holdId === "string" ? query.holdId : "";
  const raw = new URLSearchParams();
  Object.entries(query).forEach(([key, value]) => {
    if (typeof value === "string") raw.set(key, value);
  });
  return (
    <PaymentPage
      backQuery={allowlistedRecoveryParams(raw.toString()).toString()}
      holdId={holdId}
    />
  );
}
