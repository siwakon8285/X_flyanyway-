import type { TranslationKey } from "@/i18n/types";

type JourneyImage = {
  altKey: TranslationKey;
  src: string;
};

type JourneyChapter = {
  bodyKey: TranslationKey;
  headlineKey: TranslationKey;
  id: "begin" | "move" | "arrive";
  images: readonly [JourneyImage, JourneyImage];
  labelKey: TranslationKey;
};

type JourneyDepthRole = "front" | "queued" | "deep" | "off-deck";

type JourneySlotGeometry = {
  filter: string;
  height: string;
  inset: string;
  opacity: number;
  top: string;
  zIndex: number;
};

const journeyImages = {
  arriveCity: {
    altKey: "home.journey.arrive.imageCity",
    src: "/images/hero/x-fly-journey-arrive-a.jpg",
  },
  arriveTraveler: {
    altKey: "home.journey.arrive.imageTraveler",
    src: "/images/hero/x-fly-journey-arrive-b.jpg",
  },
  beginDocuments: {
    altKey: "home.journey.begin.imageDocuments",
    src: "/images/hero/x-fly-journey-begin-a.jpg",
  },
  beginLounge: {
    altKey: "home.journey.begin.imageLounge",
    src: "/images/hero/x-fly-journey-begin-b.jpg",
  },
  moveCabin: {
    altKey: "home.journey.move.imageCabin",
    src: "/images/hero/x-fly-journey-move-a.jpg",
  },
  moveWindow: {
    altKey: "home.journey.move.imageWindow",
    src: "/images/hero/x-fly-journey-move-b.jpg",
  },
} as const satisfies Record<string, JourneyImage>;

const journeyChapters = [
  {
    bodyKey: "home.journey.begin.body",
    headlineKey: "home.journey.begin.headline",
    id: "begin",
    images: [journeyImages.beginDocuments, journeyImages.beginLounge],
    labelKey: "home.journey.begin.label",
  },
  {
    bodyKey: "home.journey.move.body",
    headlineKey: "home.journey.move.headline",
    id: "move",
    images: [journeyImages.moveWindow, journeyImages.moveCabin],
    labelKey: "home.journey.move.label",
  },
  {
    bodyKey: "home.journey.arrive.body",
    headlineKey: "home.journey.arrive.headline",
    id: "arrive",
    images: [journeyImages.arriveTraveler, journeyImages.arriveCity],
    labelKey: "home.journey.arrive.label",
  },
] as const satisfies readonly JourneyChapter[];

const journeyStageCards = {
  left: [
    journeyImages.beginDocuments,
    journeyImages.moveWindow,
    journeyImages.arriveTraveler,
    journeyImages.arriveCity,
  ],
  right: [
    journeyImages.beginLounge,
    journeyImages.moveCabin,
    journeyImages.arriveCity,
    journeyImages.arriveTraveler,
  ],
} as const;

const journeyPromotionBeats = [
  { chapterAdvance: "move", promotesStackIndex: 1 },
  { chapterAdvance: "arrive", promotesStackIndex: 2 },
] as const;

const journeyMotionTiming = {
  copyChangeOffset: 0.98,
  interpolationEase: "none",
  pairStarts: [0.55, 2.45],
  promotionDuration: 0.88,
  scrollDistanceVh: 2.3,
  scrubSmoothing: 0.45,
} as const;

const journeyDepthRoles = ["front", "queued", "deep", "off-deck"] as const;

const journeyStageSlots = {
  left: {
    front: {
      filter: "blur(0.15px)",
      height: "47%",
      inset: "0%",
      opacity: 0.92,
      top: "43%",
      zIndex: 30,
    },
    queued: {
      filter: "blur(0.45px)",
      height: "35%",
      inset: "47%",
      opacity: 0.36,
      top: "12%",
      zIndex: 22,
    },
    deep: {
      filter: "blur(0.95px)",
      height: "24%",
      inset: "52%",
      opacity: 0.1,
      top: "1%",
      zIndex: 9,
    },
    "off-deck": {
      filter: "blur(1px)",
      height: "24%",
      inset: "38%",
      opacity: 0,
      top: "0%",
      zIndex: 0,
    },
  },
  right: {
    front: {
      filter: "blur(0px)",
      height: "62%",
      inset: "0%",
      opacity: 1,
      top: "38%",
      zIndex: 36,
    },
    queued: {
      filter: "blur(0.75px)",
      height: "29%",
      inset: "45%",
      opacity: 0.24,
      top: "4%",
      zIndex: 18,
    },
    deep: {
      filter: "blur(1.15px)",
      height: "20%",
      inset: "54%",
      opacity: 0.07,
      top: "0%",
      zIndex: 7,
    },
    "off-deck": {
      filter: "blur(1px)",
      height: "24%",
      inset: "37%",
      opacity: 0,
      top: "0%",
      zIndex: 0,
    },
  },
} as const satisfies Record<
  keyof typeof journeyStageCards,
  Record<JourneyDepthRole, JourneySlotGeometry>
>;

export {
  journeyChapters,
  journeyDepthRoles,
  journeyImages,
  journeyMotionTiming,
  journeyPromotionBeats,
  journeyStageCards,
  journeyStageSlots,
};
export type {
  JourneyChapter,
  JourneyDepthRole,
  JourneyImage,
  JourneySlotGeometry,
};
