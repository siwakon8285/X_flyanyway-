import { Check, Slash, UserRound } from "lucide-react";

import type { CabinClass } from "@/components/booking/search/searchTypes";
import type { SeatAvailability } from "@/components/booking/seats/seatMapTypes";

const StatusContent = ({
  availability,
  seatNumber,
  selected,
}: {
  availability: SeatAvailability;
  seatNumber: string;
  selected: boolean;
}) => (
  <span className="relative z-20 flex flex-col items-center text-[0.625rem] font-semibold leading-none drop-shadow-[0_1px_2px_rgb(0_0_0/0.8)]">
    {selected ? <Check aria-hidden="true" className="mb-0.5 size-3.5" /> : null}
    {!selected && availability === "booked" ? (
      <UserRound aria-hidden="true" className="mb-0.5 size-3.5" />
    ) : null}
    {!selected && availability === "unavailable" ? (
      <Slash aria-hidden="true" className="mb-0.5 size-3.5" />
    ) : null}
    {seatNumber}
  </span>
);

const SeatVisual = ({
  availability,
  cabin,
  seatNumber,
  selected,
}: {
  availability: SeatAvailability;
  cabin: CabinClass;
  seatNumber: string;
  selected: boolean;
}) => {
  if (cabin === "business") {
    return (
      <span
        className="relative flex size-12 items-center justify-center overflow-hidden"
        data-seat-form="lie-flat-pod"
        data-seat-model="business"
      >
        <span
          className="absolute inset-0 rounded-[1rem_0.35rem_1rem_0.7rem] border border-current/65 bg-current/8 shadow-[inset_0_0_0_1px_rgb(0_0_0/0.22)]"
          data-seat-part="privacy-shell"
        />
        <span
          className="absolute bottom-1.5 left-1.5 top-1.5 w-6 -rotate-3 rounded-[0.7rem_0.45rem_0.55rem_0.45rem] border border-current/35 bg-current/18 before:absolute before:inset-x-1 before:top-1 before:h-2 before:rounded-full before:bg-current/25 after:absolute after:bottom-1 after:left-1/2 after:h-4 after:w-4 after:-translate-x-1/2 after:rounded-md after:bg-black/18"
          data-seat-part="seat"
        />
        <span
          className="absolute bottom-1.5 right-1.5 top-1.5 w-2 rounded-full border border-current/30 bg-black/20 before:absolute before:left-1/2 before:top-1 before:size-1 before:-translate-x-1/2 before:rounded-full before:bg-current/50"
          data-seat-part="console"
        />
        <StatusContent availability={availability} seatNumber={seatNumber} selected={selected} />
      </span>
    );
  }

  if (cabin === "first") {
    return (
      <span
        className="relative flex h-16 w-18 items-center justify-center overflow-hidden"
        data-seat-form="private-suite"
        data-seat-model="first"
      >
        <span
          className="absolute inset-0 rounded-[1.3rem_0.4rem_1.3rem_0.8rem] border border-current/70 bg-current/7 shadow-[inset_0_0_0_1px_rgb(0_0_0/0.25)] before:absolute before:bottom-1 before:right-0 before:h-5 before:border-r-2 before:border-current/55"
          data-seat-part="suite-shell"
        />
        <span
          className="absolute bottom-2 left-2 top-2 w-9 rounded-[1rem_0.65rem_0.8rem_0.6rem] border border-current/40 bg-current/18 before:absolute before:inset-x-1 before:top-1 before:h-3 before:rounded-full before:bg-current/24 after:absolute after:bottom-1 after:left-1/2 after:h-5 after:w-6 after:-translate-x-1/2 after:rounded-lg after:bg-black/18"
          data-seat-part="armchair"
        />
        <span
          className="absolute bottom-2 right-2 top-2 w-3 rounded-full border border-current/35 bg-black/22 before:absolute before:left-1/2 before:top-1.5 before:size-1.5 before:-translate-x-1/2 before:rounded-full before:border before:border-current/50"
          data-seat-part="console"
        />
        <StatusContent availability={availability} seatNumber={seatNumber} selected={selected} />
      </span>
    );
  }

  if (cabin === "premium-economy") {
    return (
      <span
        className="relative flex h-10 w-9 items-center justify-center"
        data-seat-form="wide-recliner"
        data-seat-model="premium-economy"
      >
        <span
          className="absolute left-1 right-1 top-0 h-2.5 rounded-[0.65rem_0.65rem_0.35rem_0.35rem] border border-current/55 bg-current/24"
          data-seat-part="headrest"
        />
        <span
          className="absolute inset-x-1 bottom-2 top-2 rounded-[0.4rem_0.4rem_0.3rem_0.3rem] border-x border-current/45 bg-current/14"
          data-seat-part="padded-backrest"
        />
        <span
          className="absolute bottom-0.5 left-1 right-1 h-3 rounded-[0.25rem_0.25rem_0.45rem_0.45rem] border border-current/45 bg-current/20"
          data-seat-part="seat-pan"
        />
        <span
          className="absolute inset-y-2 left-0.5 right-0.5 border-x-2 border-current/45"
          data-seat-part="armrests"
        />
        <StatusContent availability={availability} seatNumber={seatNumber} selected={selected} />
      </span>
    );
  }

  return (
    <span
      className="relative flex h-8 w-7 items-center justify-center"
      data-seat-form="conventional"
      data-seat-model="economy"
    >
      <span
        className="absolute left-1 right-1 top-0 h-2 rounded-[0.5rem_0.5rem_0.25rem_0.25rem] border border-current/50 bg-current/22"
        data-seat-part="headrest"
      />
      <span
        className="absolute inset-x-1 bottom-2 top-1.5 rounded-[0.3rem_0.3rem_0.2rem_0.2rem] border-x border-current/40 bg-current/12"
        data-seat-part="backrest"
      />
      <span
        className="absolute bottom-0.5 left-1 right-1 h-2.5 rounded-[0.2rem_0.2rem_0.35rem_0.35rem] border border-current/45 bg-current/18"
        data-seat-part="seat-pan"
      />
      <span
        className="absolute inset-y-1.5 left-0.5 right-0.5 border-x border-current/45"
        data-seat-part="armrests"
      />
      <StatusContent availability={availability} seatNumber={seatNumber} selected={selected} />
    </span>
  );
};

export { SeatVisual };
