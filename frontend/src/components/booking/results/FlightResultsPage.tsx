import type { FlightSearchFormValues } from "@/components/booking/search/searchTypes";
import { FLIGHT_RESULT_FIXTURES } from "@/components/booking/results/flightResultFixtures";
import { FlightResultsList } from "@/components/booking/results/FlightResultsList";
import { FlightResultsState } from "@/components/booking/results/FlightResultsState";
import { filterFlightResults } from "@/components/booking/results/flightResultUtils";
import { SearchSummary } from "@/components/booking/results/SearchSummary";
import { Container } from "@/components/layout/Container";

const FlightResultsPage = ({
  criteria,
  query,
}: {
  criteria: FlightSearchFormValues | null;
  query: string;
}) => {
  if (!criteria) {
    return (
      <Container>
        <FlightResultsState
          description="Return to Flight Search to choose your itinerary."
          modifyHref="/#flight-search"
          title="Search details required"
        />
      </Container>
    );
  }

  const flights = filterFlightResults(FLIGHT_RESULT_FIXTURES, criteria);
  const modifyHref = `/?${query}#flight-search`;

  return (
    <section className="relative min-h-screen overflow-hidden py-section-sm pt-[calc(var(--header-height)+clamp(3rem,7vw,6rem))]">
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-x-0 top-0 h-[32rem] bg-[radial-gradient(circle_at_78%_10%,rgba(255,212,0,0.1),transparent_28rem)]"
      />
      <Container className="relative">
        <SearchSummary criteria={criteria} query={query} />
        <div className="pt-9 lg:pt-12">
          {flights.length > 0 ? (
            <FlightResultsList cabin={criteria.cabin} flights={flights} query={query} />
          ) : (
            <FlightResultsState
              description="Try adjusting your date, route, or cabin."
              headingLevel={2}
              modifyHref={modifyHref}
              title="No flights found"
            />
          )}
        </div>
      </Container>
    </section>
  );
};

export { FlightResultsPage };
