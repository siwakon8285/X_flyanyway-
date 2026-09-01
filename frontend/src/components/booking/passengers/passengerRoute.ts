import type { CabinClass } from "@/components/booking/search/searchTypes";

const buildPassengerInformationHref = ({
  flightId,
  holdId,
  query,
  selectedCabin,
}: {
  flightId: string;
  holdId: string;
  query: string;
  selectedCabin: CabinClass;
}) => {
  const params = new URLSearchParams(query);
  params.set("flightId", flightId);
  params.set("selectedCabin", selectedCabin);
  params.set("holdId", holdId);
  params.delete("seats");
  return `/booking/passengers?${params.toString()}`;
};

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

const allowlistedRecoveryParams = (query: string) => {
  const result = new URLSearchParams();
  new URLSearchParams(query).forEach((value, key) => {
    if (recoveryKeys.has(key)) result.set(key, value);
  });
  return result;
};

const buildExtrasHandoffHref = ({
  holdId,
  query,
}: {
  holdId: string;
  query: string;
}) => {
  const params = allowlistedRecoveryParams(query);
  params.set("holdId", holdId);
  return `/booking/extras?${params.toString()}`;
};

const buildReviewHandoffHref = (holdId: string) =>
  `/booking/review?${new URLSearchParams({ holdId }).toString()}`;

export {
  allowlistedRecoveryParams,
  buildExtrasHandoffHref,
  buildPassengerInformationHref,
  buildReviewHandoffHref,
};
