const gsapEasings = {
  standard: "power2.out",
  enter: "power3.out",
  exit: "power2.in",
  cinematic: "expo.out",
} as const;

const motionEasings = {
  standard: [0.25, 0.1, 0.25, 1],
  enter: [0.16, 1, 0.3, 1],
  exit: [0.4, 0, 1, 1],
} as const;

export { gsapEasings, motionEasings };
