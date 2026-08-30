import type { CabinClass } from "@/components/booking/search/searchTypes";
import type { TranslationKey } from "@/i18n/types";

type CabinPresentation = {
  accentClass: string;
  altKey: TranslationKey;
  descriptionKey: TranslationKey;
  featureKeys: readonly [TranslationKey, TranslationKey, TranslationKey];
  glowClass: string;
  id: CabinClass;
  image: string;
  imagePosition: string;
  labelKey: TranslationKey;
  shortLabelKey: TranslationKey;
  surfaceClass: string;
  tabInteractionClass: string;
};

const CABIN_PRESENTATIONS = [
  {
    accentClass: "text-[#8eb8df]",
    altKey: "flightDetail.cabinContent.economy.alt",
    descriptionKey: "flightDetail.cabinContent.economy.description",
    featureKeys: [
      "flightDetail.cabinContent.economy.feature1",
      "flightDetail.cabinContent.economy.feature2",
      "flightDetail.cabinContent.economy.feature3",
    ],
    glowClass:
      "bg-[radial-gradient(circle_at_76%_22%,rgba(72,119,165,0.28),transparent_46%)]",
    id: "economy",
    image: "/images/cabins/x-fly-cabin-economy-v1.png",
    imagePosition: "object-[52%_48%]",
    labelKey: "common.cabins.economy",
    shortLabelKey: "common.cabins.economy",
    surfaceClass: "border-[#4b6780]/45 bg-[#101821]/88",
    tabInteractionClass:
      "hover:border-[#8eb8df]/55 hover:shadow-[0_8px_24px_rgb(82_135_184/0.13)] data-[state=active]:shadow-[0_10px_28px_rgb(82_135_184/0.16)]",
  },
  {
    accentClass: "text-[#91c8c4]",
    altKey: "flightDetail.cabinContent.premiumEconomy.alt",
    descriptionKey: "flightDetail.cabinContent.premiumEconomy.description",
    featureKeys: ["flightDetail.cabinContent.premiumEconomy.feature1", "flightDetail.cabinContent.premiumEconomy.feature2", "flightDetail.cabinContent.premiumEconomy.feature3"],
    glowClass:
      "bg-[radial-gradient(circle_at_76%_22%,rgba(58,121,130,0.25),transparent_46%)]",
    id: "premium-economy",
    image: "/images/cabins/x-fly-cabin-premium-economy-v1.png",
    imagePosition: "object-[50%_50%]",
    labelKey: "common.cabins.premiumEconomy",
    shortLabelKey: "common.cabins.premiumEconomyShort",
    surfaceClass: "border-[#47767b]/45 bg-[#101a1d]/88",
    tabInteractionClass:
      "hover:border-[#91c8c4]/55 hover:shadow-[0_8px_24px_rgb(72_145_148/0.13)] data-[state=active]:shadow-[0_10px_28px_rgb(72_145_148/0.16)]",
  },
  {
    accentClass: "text-[#c8a1cf]",
    altKey: "flightDetail.cabinContent.business.alt",
    descriptionKey: "flightDetail.cabinContent.business.description",
    featureKeys: ["flightDetail.cabinContent.business.feature1", "flightDetail.cabinContent.business.feature2", "flightDetail.cabinContent.business.feature3"],
    glowClass:
      "bg-[radial-gradient(circle_at_76%_22%,rgba(105,49,105,0.3),transparent_46%)]",
    id: "business",
    image: "/images/cabins/x-fly-cabin-business-v1.png",
    imagePosition: "object-[48%_50%]",
    labelKey: "common.cabins.business",
    shortLabelKey: "common.cabins.business",
    surfaceClass: "border-[#70466f]/45 bg-[#1b111d]/88",
    tabInteractionClass:
      "hover:border-[#c8a1cf]/55 hover:shadow-[0_8px_24px_rgb(126_72_130/0.15)] data-[state=active]:shadow-[0_10px_28px_rgb(126_72_130/0.18)]",
  },
  {
    accentClass: "text-[#e7bd70]",
    altKey: "flightDetail.cabinContent.first.alt",
    descriptionKey: "flightDetail.cabinContent.first.description",
    featureKeys: ["flightDetail.cabinContent.first.feature1", "flightDetail.cabinContent.first.feature2", "flightDetail.cabinContent.first.feature3"],
    glowClass:
      "bg-[radial-gradient(circle_at_76%_22%,rgba(151,62,43,0.28),transparent_46%)]",
    id: "first",
    image: "/images/cabins/x-fly-cabin-first-v1.png",
    imagePosition: "object-[50%_50%]",
    labelKey: "common.cabins.first",
    shortLabelKey: "common.cabins.first",
    surfaceClass: "border-[#76502f]/50 bg-[#1d1410]/88",
    tabInteractionClass:
      "hover:border-[#e7bd70]/55 hover:shadow-[0_8px_24px_rgb(178_112_59/0.14)] data-[state=active]:shadow-[0_10px_28px_rgb(178_112_59/0.18)]",
  },
] as const satisfies readonly CabinPresentation[];

const CABIN_PRESENTATION_BY_ID = Object.fromEntries(
  CABIN_PRESENTATIONS.map((cabin) => [cabin.id, cabin]),
) as Record<CabinClass, (typeof CABIN_PRESENTATIONS)[number]>;

const cabinLabelKeys = Object.fromEntries(
  CABIN_PRESENTATIONS.map((cabin) => [cabin.id, cabin.labelKey]),
) as Record<CabinClass, TranslationKey>;

export { CABIN_PRESENTATION_BY_ID, CABIN_PRESENTATIONS, cabinLabelKeys };
export type { CabinPresentation };
