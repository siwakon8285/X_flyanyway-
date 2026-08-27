const motionMediaQueries = {
  desktop: "(min-width: 64rem)",
  horizontalJourney: "(min-width: 64rem) and (min-height: 52rem)",
  mobile: "(max-width: 47.999rem)",
  parallax: "(min-width: 48rem)",
  tablet: "(min-width: 48rem) and (max-width: 63.999rem)",
} as const;

export { motionMediaQueries };
