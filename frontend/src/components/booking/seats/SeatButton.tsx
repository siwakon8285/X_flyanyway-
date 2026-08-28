"use client";

import { motion } from "motion/react";

import { cabinLabels } from "@/components/booking/cabin/cabinPresentation";
import { SeatVisual } from "@/components/booking/seats/seat-models/SeatVisual";
import { SEAT_MAP_PRESENTATION } from "@/components/booking/seats/seatMapPresentation";
import type { AircraftSeat } from "@/components/booking/seats/seatMapTypes";
import { useReducedMotion } from "@/lib/motion/reducedMotion";
import { cn } from "@/lib/utils/cn";

const SeatButton = ({
  onToggle,
  seat,
  selected,
}: {
  onToggle: (seatId: string) => void;
  seat: AircraftSeat;
  selected: boolean;
}) => {
  const reducedMotion = useReducedMotion();
  const presentation = SEAT_MAP_PRESENTATION[seat.cabin];
  const disabled = seat.availability !== "available";
  const announcedStatus = selected ? "selected" : seat.availability;

  return (
    <motion.button
      animate={
        reducedMotion
          ? undefined
          : { scale: selected ? 1.03 : 1, y: selected ? -2 : 0 }
      }
      aria-label={`Seat ${seat.seatNumber}, ${cabinLabels[seat.cabin]}, ${seat.position}, ${announcedStatus}`}
      aria-pressed={disabled ? undefined : selected}
      className={cn(
        "relative flex shrink-0 items-center justify-center rounded-xl border outline-none transition-[border-color,background-color,box-shadow,color] duration-200 focus-visible:ring-2 focus-visible:ring-brand focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed motion-reduce:transition-colors",
        presentation.seatButtonClass,
        disabled
          ? seat.availability === "booked"
            ? "border-transparent bg-transparent text-[#828993] opacity-65"
            : "border-transparent bg-transparent text-[#626873] opacity-55"
          : selected
            ? "border-brand/55 bg-brand/8 text-brand shadow-[0_8px_26px_rgb(255_212_0/0.28)]"
            : presentation.availableClass,
      )}
      data-seat
      data-seat-number={seat.seatNumber}
      disabled={disabled}
      onClick={() => onToggle(seat.id)}
      transition={{ duration: reducedMotion ? 0 : 0.22, ease: [0.22, 1, 0.36, 1] }}
      type="button"
      whileHover={
        disabled || reducedMotion ? undefined : { scale: 1.055, y: -4 }
      }
      whileTap={disabled || reducedMotion ? undefined : { scale: 0.96, y: 0 }}
    >
      <SeatVisual
        availability={seat.availability}
        cabin={seat.cabin}
        seatNumber={seat.seatNumber}
        selected={selected}
      />
    </motion.button>
  );
};

export { SeatButton };
