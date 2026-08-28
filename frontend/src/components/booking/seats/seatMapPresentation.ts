import type { CabinClass } from "@/components/booking/search/searchTypes";

type SeatMapPresentation = {
  accentClass: string;
  aircraftClass: string;
  availableClass: string;
  cabin: CabinClass;
  columnGuide: string;
  glowClass: string;
  rowClass: string;
  seatButtonClass: string;
};

const SEAT_MAP_PRESENTATION = {
  economy: {
    accentClass: "text-[#8eb8df]",
    aircraftClass: "border-white/12 bg-[#090d12]",
    availableClass:
      "border-transparent bg-transparent text-[#8eb8df] hover:border-[#9bc5e8]/45 hover:bg-[#27445d]/16 hover:shadow-[0_9px_20px_rgb(73_126_171/0.2)]",
    cabin: "economy",
    columnGuide: "Window · A B C · Aisle · D E F · Window",
    glowClass:
      "bg-[radial-gradient(circle_at_50%_12%,rgba(72,119,165,0.22),transparent_48%)]",
    rowClass: "gap-0.5 min-[26rem]:gap-1.5",
    seatButtonClass: "size-9 min-[23rem]:size-10 min-[26rem]:size-11",
  },
  "premium-economy": {
    accentClass: "text-[#91c8c4]",
    aircraftClass: "border-white/12 bg-[#090d12]",
    availableClass:
      "border-transparent bg-transparent text-[#91c8c4] hover:border-[#a8ded9]/45 hover:bg-[#28545a]/16 hover:shadow-[0_9px_22px_rgb(67_147_148/0.2)]",
    cabin: "premium-economy",
    columnGuide: "Window · A C · Aisle · D F · Window",
    glowClass:
      "bg-[radial-gradient(circle_at_50%_12%,rgba(58,133,139,0.22),transparent_48%)]",
    rowClass: "gap-1 min-[23rem]:gap-2.5",
    seatButtonClass: "size-11 min-[23rem]:size-13",
  },
  business: {
    accentClass: "text-[#c8a1cf]",
    aircraftClass: "border-white/12 bg-[#090d12]",
    availableClass:
      "border-transparent bg-transparent text-[#c8a1cf] hover:border-[#d3a9d8]/45 hover:bg-[#4b294d]/16 hover:shadow-[0_10px_24px_rgb(126_72_130/0.22)]",
    cabin: "business",
    columnGuide: "Window A · Aisle · D G · Aisle · K Window",
    glowClass:
      "bg-[radial-gradient(circle_at_50%_12%,rgba(120,54,119,0.25),transparent_50%)]",
    rowClass: "gap-3",
    seatButtonClass: "size-16",
  },
  first: {
    accentClass: "text-[#e7bd70]",
    aircraftClass: "border-white/12 bg-[#090d12]",
    availableClass:
      "border-transparent bg-transparent text-[#e7bd70] hover:border-[#efc77b]/45 hover:bg-[#4c241f]/16 hover:shadow-[0_12px_28px_rgb(178_112_59/0.22)]",
    cabin: "first",
    columnGuide: "Private suite A · Aisle · Private suite K",
    glowClass:
      "bg-[radial-gradient(circle_at_50%_12%,rgba(160,69,45,0.24),transparent_52%)]",
    rowClass: "gap-2 min-[23rem]:gap-5",
    seatButtonClass: "h-20 w-18 min-[23rem]:w-22",
  },
} as const satisfies Record<CabinClass, SeatMapPresentation>;

export { SEAT_MAP_PRESENTATION };
