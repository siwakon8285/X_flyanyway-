"use client";

import { useState } from "react";

import type { SeatSelectionRequest } from "@/components/booking/detail/flightDetailUtils";
import { AircraftSeatMap } from "@/components/booking/seats/AircraftSeatMap";
import { SeatSelectionSummary } from "@/components/booking/seats/SeatSelectionSummary";
import type { CabinSeatMap } from "@/components/booking/seats/seatMapTypes";
import { toggleSeatSelection } from "@/components/booking/seats/seatMapUtils";

const SeatMapExperience = ({
  map,
  request,
  requiredSeatCount,
}: {
  map: CabinSeatMap;
  request: SeatSelectionRequest;
  requiredSeatCount: number;
}) => {
  const [selectedSeatIds, setSelectedSeatIds] = useState<Set<string>>(
    () => new Set(),
  );
  const [limitReached, setLimitReached] = useState(false);

  const handleToggle = (seatId: string) => {
    const result = toggleSeatSelection({
      requiredSeatCount,
      seatId,
      selectedSeatIds,
    });
    setSelectedSeatIds(result.selectedSeatIds);
    setLimitReached(result.limitReached);
  };

  return (
    <div className="mt-10 grid min-w-0 gap-8 lg:grid-cols-[minmax(0,1fr)_20rem] lg:items-start">
      <AircraftSeatMap
        map={map}
        onToggle={handleToggle}
        selectedSeatIds={selectedSeatIds}
      />
      <SeatSelectionSummary
        limitReached={limitReached}
        request={request}
        requiredSeatCount={requiredSeatCount}
        selectedSeatIds={selectedSeatIds}
      />
    </div>
  );
};

export { SeatMapExperience };
