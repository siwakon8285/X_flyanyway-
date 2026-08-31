"use client";

import type { SeatSelectionRequest } from "@/components/booking/detail/flightDetailUtils";
import { AircraftSeatMap } from "@/components/booking/seats/AircraftSeatMap";
import { SeatSelectionSummary } from "@/components/booking/seats/SeatSelectionSummary";
import type { CabinSeatMap } from "@/components/booking/seats/seatMapTypes";
import { useSeatHold } from "@/components/booking/seats/useSeatHold";

const SeatMapExperience = ({
  map,
  request,
  requiredSeatCount,
}: {
  map: CabinSeatMap;
  request: SeatSelectionRequest;
  requiredSeatCount: number;
}) => {
  const seatHold = useSeatHold({ initialMap: map, request, requiredSeatCount });

  return (
    <div className="mt-10 grid min-w-0 gap-8 lg:grid-cols-[minmax(0,1fr)_20rem] lg:items-start">
      <AircraftSeatMap
        interactionLocked={seatHold.mutationPending}
        map={seatHold.map}
        onToggle={seatHold.handleToggle}
        pendingSeatIds={seatHold.pendingSeatIds}
        selectedSeatIds={seatHold.selectedSeatIds}
      />
      <SeatSelectionSummary
        continuePending={seatHold.continuePending}
        holdActive={Boolean(seatHold.hold)}
        limitReached={seatHold.limitReached}
        messageKey={seatHold.messageKey}
        mutationPending={seatHold.mutationPending}
        onContinue={seatHold.handleContinue}
        remainingMilliseconds={seatHold.remainingMilliseconds}
        request={request}
        requiredSeatCount={requiredSeatCount}
        selectedSeatIds={seatHold.selectedSeatIds}
      />
    </div>
  );
};

export { SeatMapExperience };
