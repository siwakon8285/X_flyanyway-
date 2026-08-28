import { Container } from "@/components/layout/Container";
import { Skeleton } from "@/components/ui/Skeleton";

export default function SeatMapLoading() {
  return (
    <Container>
      <main
        aria-label="Loading seat map"
        className="min-h-screen py-section-sm pt-[calc(var(--header-height)+clamp(2rem,5vw,4rem))]"
        role="status"
      >
        <span className="sr-only">Loading seat map</span>
        <header data-seat-map-skeleton>
          <Skeleton className="h-11 w-36" />
          <Skeleton className="mt-8 h-5 w-24" />
          <Skeleton className="mt-4 h-20 w-full max-w-2xl" />
          <Skeleton className="mt-5 h-6 w-full max-w-xl" />
        </header>
        <section
          className="mt-10 grid gap-8 lg:grid-cols-[minmax(0,1fr)_20rem]"
          data-seat-map-skeleton
        >
          <Skeleton className="h-[42rem] rounded-[7rem_7rem_3rem_3rem]" />
          <Skeleton className="h-96" />
        </section>
      </main>
    </Container>
  );
}
