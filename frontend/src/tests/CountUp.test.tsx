import { act, render, screen } from "@testing-library/react";

import { CountUp } from "@/components/motion/CountUp";

describe("CountUp", () => {
  it("renders the final stable value for reduced-motion users", () => {
    render(<CountUp decimals={1} end={92} suffix="%" />);

    expect(screen.getByLabelText("92.0%")).toHaveTextContent("92.0%");
  });

  it("restores the final value when reduced motion is enabled at runtime", () => {
    let matches = false;
    const eventTarget = new EventTarget();
    const mediaQueryList = Object.defineProperties(eventTarget, {
      addListener: { value: () => undefined },
      matches: { get: () => matches },
      media: { value: "(prefers-reduced-motion: reduce)" },
      onchange: { value: null, writable: true },
      removeListener: { value: () => undefined },
    }) as MediaQueryList;
    window.matchMedia = jest.fn<MediaQueryList, [query: string]>(
      () => mediaQueryList,
    );

    render(<CountUp end={156} />);
    const output = screen.getByLabelText("156");
    expect(output).toHaveTextContent("0");

    act(() => {
      matches = true;
      mediaQueryList.dispatchEvent(new Event("change"));
    });

    expect(output).toHaveTextContent("156");
  });
});
