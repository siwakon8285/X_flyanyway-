import {
  INITIAL_HASH_POSITIONING_ATTRIBUTE,
  runInitialHashBootstrap,
} from "@/lib/motion/initialHash";

const hasPositioningMarker = () =>
  document.documentElement.hasAttribute(INITIAL_HASH_POSITIONING_ATTRIBUTE);

describe("initial hash bootstrap", () => {
  beforeEach(() => {
    document.documentElement.removeAttribute(INITIAL_HASH_POSITIONING_ATTRIBUTE);
    window.history.replaceState(null, "", "/");
  });

  it("leaves a normal homepage visit visible", () => {
    runInitialHashBootstrap();

    expect(hasPositioningMarker()).toBe(false);
  });

  it("marks a homepage flight-search hash before positioning", () => {
    window.history.replaceState(
      null,
      "",
      "/?from=DXB&to=JFK#flight-search",
    );

    runInitialHashBootstrap();

    expect(hasPositioningMarker()).toBe(true);
  });

  it.each([
    "/#unknown-section",
    "/#%E0%A4%A",
    "/flights#flight-search",
  ])("does not conceal unsupported location %s", (location) => {
    window.history.replaceState(null, "", location);

    runInitialHashBootstrap();

    expect(hasPositioningMarker()).toBe(false);
  });
});
