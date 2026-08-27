"use client";

import { useSyncExternalStore } from "react";
import { useRouter } from "next/navigation";

import { FlightSearchForm } from "@/components/booking/search/FlightSearchForm";
import {
  parseFlightSearch,
  serializeFlightSearch,
} from "@/components/booking/search/searchState";
import { Container } from "@/components/layout/Container";
import { Reveal } from "@/components/motion/Reveal";

const subscribeToLocation = (onStoreChange: () => void) => {
  window.addEventListener("popstate", onStoreChange);
  return () => window.removeEventListener("popstate", onStoreChange);
};
const getLocationSearch = () => window.location.search;
const getServerLocationSearch = () => "";

const FlightSearchSection = () => {
  const router = useRouter();
  const locationSearch = useSyncExternalStore(
    subscribeToLocation,
    getLocationSearch,
    getServerLocationSearch,
  );
  const initialValues = parseFlightSearch(new URLSearchParams(locationSearch));

  return (
    <section
      aria-labelledby="flight-search-heading"
      className="relative scroll-mt-header overflow-hidden border-y border-border bg-[radial-gradient(circle_at_85%_15%,rgba(255,212,0,0.09),transparent_24rem),linear-gradient(180deg,#101010_0%,#090909_100%)] py-section-md"
      id="flight-search"
    >
      <div
        aria-hidden="true"
        className="absolute inset-y-0 right-[8%] hidden w-px bg-gradient-to-b from-transparent via-brand/30 to-transparent lg:block"
      />
      <Container className="relative">
        <Reveal as="div" variant="fade-up">
          <div className="grid gap-6 border-b border-border pb-8 lg:grid-cols-[minmax(0,0.75fr)_minmax(20rem,0.45fr)] lg:items-end lg:gap-16">
            <div>
              <p className="text-label text-brand">Flight search · Earth routes</p>
              <h2
                className="mt-4 max-w-[11ch] text-h1 uppercase text-balance"
                id="flight-search-heading"
              >
                Where will you go next?
              </h2>
            </div>
            <p className="max-w-md text-body-lg text-muted-foreground lg:justify-self-end">
              Choose your route and shape the journey in a few precise steps.
            </p>
          </div>
          <div className="pt-8 lg:pt-10">
            <FlightSearchForm
              initialValues={initialValues}
              key={locationSearch}
              onValidSubmit={(values) => {
                const query = serializeFlightSearch(values);
                router.push(`/flights?${query}`);
              }}
            />
          </div>
        </Reveal>
      </Container>
    </section>
  );
};

export { FlightSearchSection };
