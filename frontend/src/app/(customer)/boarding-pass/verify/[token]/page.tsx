import type { Metadata } from "next";

import { BoardingPassVerifyPage } from "@/components/boarding-pass/BoardingPassVerifyPage";

export const metadata: Metadata = {
  description: "Current signed Boarding Pass verification for X-Fly flights.",
  title: "Boarding Pass Verification · X-Fly Anyway",
};

export default async function BoardingPassVerifyRoute({
  params,
}: {
  params: Promise<{ token: string }>;
}) {
  const { token } = await params;
  return <BoardingPassVerifyPage token={token} />;
}
