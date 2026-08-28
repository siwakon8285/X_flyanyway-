import type { SeatSelectionRequest } from "@/components/booking/detail/flightDetailUtils";
import { SeatMapExperience } from "@/components/booking/seats/SeatMapExperience";
import { SeatMapHeader } from "@/components/booking/seats/SeatMapHeader";
import type { CabinSeatMap } from "@/components/booking/seats/seatMapTypes";
import { getRequiredSeatCount } from "@/components/booking/seats/seatMapUtils";
import { Container } from "@/components/layout/Container";

const SeatMapPage = ({
  request,
  seatMap,
}: {
  request: SeatSelectionRequest;
  seatMap: CabinSeatMap | null;
}) => {
  const requiredSeatCount = getRequiredSeatCount(request.criteria.passengers);
  const availableSeatCount =
    seatMap?.rows
      .flatMap((row) => row.groups.flat())
      .filter((seat) => seat.availability === "available").length ?? 0;
  const hasEnoughSeats = availableSeatCount >= requiredSeatCount;

  return (
    <main className="relative min-h-screen min-w-0 overflow-x-clip pb-section-md pt-[calc(var(--header-height)+clamp(2rem,5vw,4rem))]">
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-x-0 top-0 h-[36rem] bg-[radial-gradient(circle_at_50%_0%,rgba(255,212,0,0.055),transparent_34rem)]"
      />
      <Container className="relative min-w-0">
        <SeatMapHeader request={request} requiredSeatCount={requiredSeatCount} />
        {seatMap && hasEnoughSeats ? (
          <SeatMapExperience
            map={seatMap}
            request={request}
            requiredSeatCount={requiredSeatCount}
          />
        ) : (
          <section
            aria-labelledby="seat-map-unavailable-heading"
            className="mt-10 rounded-surface border border-border bg-surface/75 p-8"
          >
            <h2 className="text-h3" id="seat-map-unavailable-heading">
              {seatMap ? "Not enough fixture seats" : "Seat map unavailable"}
            </h2>
            <p className="mt-3 max-w-xl text-body text-muted-foreground">
              {seatMap
                ? `This local cabin fixture has ${availableSeatCount} selectable ${availableSeatCount === 1 ? "seat" : "seats"}, but this booking requires ${requiredSeatCount}. Return to the flight to choose another cabin.`
                : "This cabin does not have a local seat layout. Return to the flight to choose another cabin."}
            </p>
          </section>
        )}
      </Container>
    </main>
  );
};

export { SeatMapPage };
