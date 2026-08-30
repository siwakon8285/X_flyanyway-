import { fireEvent, render, screen, waitFor } from "@testing-library/react";

import { LanguageToggle } from "@/components/layout/LanguageToggle";
import { LanguageProvider } from "@/i18n/LanguageProvider";
import { ScrollTrigger } from "@/lib/motion/gsap";
import { LocaleLayoutSync } from "@/components/motion/LocaleLayoutSync";

jest.mock("@/lib/motion/gsap", () => ({
  ScrollTrigger: { refresh: jest.fn() },
}));

describe("LocaleLayoutSync", () => {
  it("refreshes text-dependent ScrollTrigger measurements only after a locale change", async () => {
    const originalRequestAnimationFrame = window.requestAnimationFrame;
    window.requestAnimationFrame = (callback) => {
      callback(0);
      return 1;
    };

    render(
      <LanguageProvider initialLocale="en">
        <LocaleLayoutSync />
        <LanguageToggle />
      </LanguageProvider>,
    );

    expect(ScrollTrigger.refresh).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button"));

    await waitFor(() => expect(ScrollTrigger.refresh).toHaveBeenCalledTimes(1));

    window.requestAnimationFrame = originalRequestAnimationFrame;
  });
});
