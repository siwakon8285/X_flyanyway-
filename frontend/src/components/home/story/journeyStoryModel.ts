type JourneyImage = {
  alt: string;
  src: string;
};

type JourneyChapter = {
  body: string;
  headline: string;
  id: "begin" | "move" | "arrive";
  images: readonly [JourneyImage, JourneyImage];
  label: string;
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
    alt: "City arrival and premium hospitality",
    src: "/images/hero/x-fly-journey-arrive-a.jpg",
  },
  arriveTraveler: {
    alt: "Traveler arriving at a premium city hotel",
    src: "/images/hero/x-fly-journey-arrive-b.jpg",
  },
  beginDocuments: {
    alt: "Passport and premium travel documents",
    src: "/images/hero/x-fly-journey-begin-a.jpg",
  },
  beginLounge: {
    alt: "Traveler preparing to depart from an airport lounge",
    src: "/images/hero/x-fly-journey-begin-b.jpg",
  },
  moveCabin: {
    alt: "Traveler in a private premium cabin",
    src: "/images/hero/x-fly-journey-move-a.jpg",
  },
  moveWindow: {
    alt: "Business traveler looking through an aircraft window",
    src: "/images/hero/x-fly-journey-move-b.jpg",
  },
} as const satisfies Record<string, JourneyImage>;

const journeyChapters = [
  {
    body: "Every considered journey begins before departure.",
    headline: "Where the journey begins.",
    id: "begin",
    images: [journeyImages.beginDocuments, journeyImages.beginLounge],
    label: "CHAPTER 1 · BEGIN",
  },
  {
    body: "Comfort and clarity, carried through every mile.",
    headline: "One considered path.",
    id: "move",
    images: [journeyImages.moveWindow, journeyImages.moveCabin],
    label: "CHAPTER 2 · MOVE",
  },
  {
    body: "Arrival should feel as composed as the journey itself.",
    headline: "A seamless arrival.",
    id: "arrive",
    images: [journeyImages.arriveTraveler, journeyImages.arriveCity],
    label: "CHAPTER 3 · ARRIVE",
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
