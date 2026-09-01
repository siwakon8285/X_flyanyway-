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

const buildExtrasHandoffHref = (holdId: string) =>
  `/booking/extras?${new URLSearchParams({ holdId }).toString()}`;

export { buildExtrasHandoffHref, buildPassengerInformationHref };
