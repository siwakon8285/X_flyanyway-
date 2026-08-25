import { render, screen } from "@testing-library/react";

import { ParallaxMedia } from "@/components/motion/ParallaxMedia";
import { PinnedSection } from "@/components/motion/PinnedSection";

describe("motion fallbacks", () => {
  it("keeps parallax and pinned content static and readable under reduced motion", () => {
    render(
      <>
        <ParallaxMedia aria-label="Parallax proof">
          <span>Static visual surface</span>
        </ParallaxMedia>
        <PinnedSection aria-label="Pinned proof">
          <p>Static section content</p>
        </PinnedSection>
      </>,
    );

    expect(screen.getByLabelText("Parallax proof")).toHaveTextContent(
      "Static visual surface",
    );
    expect(screen.getByRole("region", { name: "Pinned proof" })).toHaveTextContent(
      "Static section content",
    );
  });
});
