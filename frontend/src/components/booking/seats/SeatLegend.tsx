import { Check, Slash, UserRound } from "lucide-react";

const legendItems = [
  { icon: null, label: "Available", swatch: "border-white/35 bg-white/10" },
  { icon: Check, label: "Selected", swatch: "border-brand bg-brand/18 text-brand" },
  { icon: UserRound, label: "Booked", swatch: "border-white/12 bg-[#24272c] text-[#828993]" },
  { icon: Slash, label: "Unavailable", swatch: "border-white/8 bg-[#111318] text-[#626873]" },
] as const;

const SeatLegend = () => (
  <section aria-labelledby="seat-legend-heading">
    <h2 className="text-caption" id="seat-legend-heading">
      Seat legend
    </h2>
    <ul className="mt-4 grid grid-cols-2 gap-3 text-xs text-muted-foreground">
      {legendItems.map(({ icon: Icon, label, swatch }) => (
        <li className="flex items-center gap-2" key={label}>
          <span className={`flex size-7 items-center justify-center rounded-lg border ${swatch}`}>
            {Icon ? <Icon aria-hidden="true" className="size-3.5" /> : null}
          </span>
          {label}
        </li>
      ))}
    </ul>
  </section>
);

export { SeatLegend };
