import type { Metadata } from "next";

import { TicketVerifyPage } from "@/components/booking/ticket/TicketVerifyPage";

export const metadata: Metadata = {
  description:
    "Official cryptographic ticket verification for X-Fly flights.",
  title: "Ticket Verification · X-Fly Anyway",
};

export default async function TicketVerifyRoute({
  params,
}: {
  params: Promise<{ token: string }>;
}) {
  const { token } = await params;
  return <TicketVerifyPage token={token} />;
}
