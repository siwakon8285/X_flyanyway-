import { Container } from "@/components/layout/Container";
import { Skeleton } from "@/components/ui/Skeleton";

export default function FlightResultsLoading() {
  return (
    <Container>
      <section
        aria-label="Loading flight results"
        aria-live="polite"
        className="min-h-screen py-section-sm pt-[calc(var(--header-height)+clamp(3rem,7vw,6rem))]"
      >
        <span className="sr-only">Loading flight results</span>
        <Skeleton className="h-4 w-28" />
        <Skeleton className="mt-5 h-20 w-full max-w-xl" />
        <Skeleton className="mt-4 h-6 w-52" />
        <div className="mt-12 space-y-4 border-t border-border pt-10">
          {[0, 1, 2].map((item) => (
            <Skeleton className="h-72 w-full" key={item} />
          ))}
        </div>
      </section>
    </Container>
  );
}
