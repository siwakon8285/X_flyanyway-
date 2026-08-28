import type { CabinClass } from "@/components/booking/search/searchTypes";

type CabinPresentation = {
  accentClass: string;
  alt: string;
  description: string;
  features: readonly [string, string, string];
  glowClass: string;
  id: CabinClass;
  image: string;
  imagePosition: string;
  label: string;
  shortLabel: string;
  surfaceClass: string;
  tabInteractionClass: string;
};

const CABIN_PRESENTATIONS = [
  {
    accentClass: "text-[#8eb8df]",
    alt: "Economy cabin seating aboard X-Fly with spacious rows and aircraft windows",
    description: "Practical comfort shaped for an effortless journey.",
    features: [
      "Practical comfort",
      "Standard recline",
      "Personal entertainment",
    ],
    glowClass:
      "bg-[radial-gradient(circle_at_76%_22%,rgba(72,119,165,0.28),transparent_46%)]",
    id: "economy",
    image: "/images/cabins/x-fly-cabin-economy-v1.png",
    imagePosition: "object-[52%_48%]",
    label: "Economy",
    shortLabel: "Economy",
    surfaceClass: "border-[#4b6780]/45 bg-[#101821]/88",
    tabInteractionClass:
      "hover:border-[#8eb8df]/55 hover:shadow-[0_8px_24px_rgb(82_135_184/0.13)] data-[state=active]:shadow-[0_10px_28px_rgb(82_135_184/0.16)]",
  },
  {
    accentClass: "text-[#91c8c4]",
    alt: "Premium Economy seating aboard X-Fly with expanded width and legroom",
    description: "More room and calm for the distance ahead.",
    features: ["Wider seat", "More legroom", "Enhanced service"],
    glowClass:
      "bg-[radial-gradient(circle_at_76%_22%,rgba(58,121,130,0.25),transparent_46%)]",
    id: "premium-economy",
    image: "/images/cabins/x-fly-cabin-premium-economy-v1.png",
    imagePosition: "object-[50%_50%]",
    label: "Premium Economy",
    shortLabel: "Premium",
    surfaceClass: "border-[#47767b]/45 bg-[#101a1d]/88",
    tabInteractionClass:
      "hover:border-[#91c8c4]/55 hover:shadow-[0_8px_24px_rgb(72_145_148/0.13)] data-[state=active]:shadow-[0_10px_28px_rgb(72_145_148/0.16)]",
  },
  {
    accentClass: "text-[#c8a1cf]",
    alt: "Business Class lie-flat pod and private side console aboard X-Fly",
    description: "Space to focus, dine, and arrive ready.",
    features: ["Lie-flat seat", "Direct aisle access", "Premium dining"],
    glowClass:
      "bg-[radial-gradient(circle_at_76%_22%,rgba(105,49,105,0.3),transparent_46%)]",
    id: "business",
    image: "/images/cabins/x-fly-cabin-business-v1.png",
    imagePosition: "object-[48%_50%]",
    label: "Business",
    shortLabel: "Business",
    surfaceClass: "border-[#70466f]/45 bg-[#1b111d]/88",
    tabInteractionClass:
      "hover:border-[#c8a1cf]/55 hover:shadow-[0_8px_24px_rgb(126_72_130/0.15)] data-[state=active]:shadow-[0_10px_28px_rgb(126_72_130/0.18)]",
  },
  {
    accentClass: "text-[#e7bd70]",
    alt: "First Class private suite with luxury dining and personalized setting aboard X-Fly",
    description: "A private expression of flight, considered in every detail.",
    features: ["Private suite", "Maximum personal space", "Signature service"],
    glowClass:
      "bg-[radial-gradient(circle_at_76%_22%,rgba(151,62,43,0.28),transparent_46%)]",
    id: "first",
    image: "/images/cabins/x-fly-cabin-first-v1.png",
    imagePosition: "object-[50%_50%]",
    label: "First",
    shortLabel: "First",
    surfaceClass: "border-[#76502f]/50 bg-[#1d1410]/88",
    tabInteractionClass:
      "hover:border-[#e7bd70]/55 hover:shadow-[0_8px_24px_rgb(178_112_59/0.14)] data-[state=active]:shadow-[0_10px_28px_rgb(178_112_59/0.18)]",
  },
] as const satisfies readonly CabinPresentation[];

const CABIN_PRESENTATION_BY_ID = Object.fromEntries(
  CABIN_PRESENTATIONS.map((cabin) => [cabin.id, cabin]),
) as Record<CabinClass, (typeof CABIN_PRESENTATIONS)[number]>;

const cabinLabels = Object.fromEntries(
  CABIN_PRESENTATIONS.map((cabin) => [cabin.id, cabin.label]),
) as Record<CabinClass, string>;

export { CABIN_PRESENTATION_BY_ID, CABIN_PRESENTATIONS, cabinLabels };
export type { CabinPresentation };
