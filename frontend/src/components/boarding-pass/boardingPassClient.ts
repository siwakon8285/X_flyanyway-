import { requestJson } from "@/components/booking/api/bookingApiClient";
import type { BoardingPassVerification } from "@/lib/admin/ticketOperationsTypes";

const buildBoardingPassVerificationUrl = (token: string, explicitOrigin?: string) => {
  const origin = (
    explicitOrigin ??
    (typeof window !== "undefined" ? window.location.origin : "http://localhost:3000")
  ).replace(/\/+$/, "");
  return `${origin}/boarding-pass/verify/${encodeURIComponent(token)}`;
};

const verifyBoardingPass = (token: string) =>
  requestJson<BoardingPassVerification>(
    `/boarding-passes/verify/${encodeURIComponent(token)}`,
  );

export { buildBoardingPassVerificationUrl, verifyBoardingPass };
