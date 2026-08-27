import { ArrowLeft, SearchX } from "lucide-react";

import { buttonVariants } from "@/components/ui/Button";

const FlightResultsState = ({
  description,
  headingLevel = 1,
  modifyHref,
  title,
}: {
  description: string;
  headingLevel?: 1 | 2;
  modifyHref: string;
  title: string;
}) => {
  const Heading = headingLevel === 1 ? "h1" : "h2";

  return (
    <section className="flex min-h-[65svh] items-center py-section-md" aria-labelledby="results-state-heading">
      <div className="mx-auto max-w-xl text-center">
        <SearchX aria-hidden="true" className="mx-auto size-10 text-brand" />
        <p className="mt-6 text-label text-brand">Flight results</p>
        <Heading className="mt-3 text-h2 uppercase" id="results-state-heading">
          {title}
        </Heading>
        <p className="mx-auto mt-5 max-w-md text-body-lg text-muted-foreground">
          {description}
        </p>
        <a className={`${buttonVariants({ size: "lg" })} mt-8`} href={modifyHref}>
          <ArrowLeft aria-hidden="true" />
          Modify Search
        </a>
      </div>
    </section>
  );
};

export { FlightResultsState };
