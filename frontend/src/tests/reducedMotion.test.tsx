import { act, render, screen } from "@testing-library/react";

import {
  REDUCED_MOTION_QUERY,
  useReducedMotion,
} from "@/lib/motion/reducedMotion";

const createMatchMediaController = (initialMatches: boolean) => {
  let matches = initialMatches;
  const eventTarget = new EventTarget();
  const mediaQueryList = Object.defineProperties(eventTarget, {
    addListener: { value: () => undefined },
    matches: { get: () => matches },
    media: { value: REDUCED_MOTION_QUERY },
    onchange: { value: null, writable: true },
    removeListener: { value: () => undefined },
  }) as MediaQueryList;
  const matchMedia = jest.fn<MediaQueryList, [query: string]>(
    () => mediaQueryList,
  );

  return {
    matchMedia,
    setMatches: (nextMatches: boolean) => {
      matches = nextMatches;
      mediaQueryList.dispatchEvent(new Event("change"));
    },
  };
};

const ReducedMotionProbe = () => (
  <output>{useReducedMotion() ? "reduced" : "full"}</output>
);

describe("useReducedMotion", () => {
  it("reads the operating-system preference and reacts to changes", () => {
    const controller = createMatchMediaController(true);
    window.matchMedia = controller.matchMedia;

    render(<ReducedMotionProbe />);
    expect(screen.getByText("reduced")).toBeInTheDocument();

    act(() => controller.setMatches(false));
    expect(screen.getByText("full")).toBeInTheDocument();
    expect(controller.matchMedia).toHaveBeenCalledWith(REDUCED_MOTION_QUERY);
  });
});
