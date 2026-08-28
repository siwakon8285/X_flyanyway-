import { AircraftSummary } from "@/components/booking/detail/AircraftSummary";
import { CabinExperience } from "@/components/booking/detail/CabinExperience";
import { FlightDetailSummary } from "@/components/booking/detail/FlightDetailSummary";
import type { FlightDetailRequest } from "@/components/booking/detail/flightDetailUtils";
import { Container } from "@/components/layout/Container";
import { Reveal } from "@/components/motion/Reveal";

const FlightDetailPage = ({
  criteria,
  flight,
  query,
}: FlightDetailRequest) => (
  <main className="relative min-h-screen overflow-hidden pt-[calc(var(--header-height)+clamp(2rem,5vw,4rem))]">
    <div
      aria-hidden="true"
      className="pointer-events-none absolute inset-x-0 top-0 h-[34rem] bg-[radial-gradient(circle_at_82%_8%,rgba(255,212,0,0.09),transparent_30rem)]"
    />
    <Container className="relative">
      <Reveal as="div">
        <FlightDetailSummary criteria={criteria} flight={flight} query={query} />
      </Reveal>
      <AircraftSummary aircraft={flight.aircraft} />
      <CabinExperience
        flight={flight}
        query={query}
        searchedCabin={criteria.cabin}
      />
    </Container>
  </main>
);

export { FlightDetailPage };
