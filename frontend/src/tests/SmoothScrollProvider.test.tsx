import { render, screen } from "@testing-library/react";
import Lenis from "lenis";

import { SmoothScrollProvider } from "@/components/motion/SmoothScrollProvider";

jest.mock("lenis", () => ({
  __esModule: true,
  default: jest.fn(() => ({
    destroy: jest.fn(),
    off: jest.fn(),
    on: jest.fn(),
    raf: jest.fn(),
  })),
}));

describe("SmoothScrollProvider", () => {
  it("owns one Lenis instance and destroys it on unmount", () => {
    window.matchMedia = jest.fn().mockReturnValue({
      matches: false,
      media: "(prefers-reduced-motion: reduce)",
      onchange: null,
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
      addListener: jest.fn(),
      removeListener: jest.fn(),
      dispatchEvent: jest.fn(),
    });

    const { rerender, unmount } = render(
      <SmoothScrollProvider>
        <p>Native content remains available.</p>
      </SmoothScrollProvider>,
    );

    rerender(
      <SmoothScrollProvider>
        <p>Native content remains available.</p>
      </SmoothScrollProvider>,
    );

    expect(screen.getByText("Native content remains available.")).toBeInTheDocument();
    const mockedLenis = jest.mocked(Lenis);
    expect(mockedLenis).toHaveBeenCalledTimes(1);

    unmount();
    const instance = mockedLenis.mock.results[0]?.value;
    expect(instance?.destroy).toHaveBeenCalledTimes(1);
  });
});
