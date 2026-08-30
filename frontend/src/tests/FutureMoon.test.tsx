import { screen, within } from "@testing-library/react";
import { render } from "@/tests/renderWithLanguage";

import Home from "@/app/page";
import { FutureMoonStory } from "@/components/home/moon/FutureMoonStory";

describe("Future Moon Story Section", () => {
  it("renders the section with accessible landmarks and semantic structure", () => {
    const { container } = render(<FutureMoonStory />);
    const section = container.querySelector("#future-moon");

    expect(section).not.toBeNull();
    expect(section).toHaveAttribute("data-moon-story");

    expect(
      within(section as HTMLElement).getByRole("heading", {
        level: 2,
        name: /NEXT:\s+THE MOON\./i,
      }),
    ).toBeInTheDocument();

    expect(
      within(section as HTMLElement).getByText("FUTURE DESTINATION"),
    ).toBeInTheDocument();

    expect(
      within(section as HTMLElement).getByText("A new destination is on the horizon."),
    ).toBeInTheDocument();

    expect(
      within(section as HTMLElement).getByText("COMING NEXT YEAR"),
    ).toBeInTheDocument();
  });

  it("contains NO duplicated 156-country messaging or global network copy in this section", () => {
    const { container } = render(<FutureMoonStory />);
    const section = container.querySelector("#future-moon");

    expect(section).not.toBeNull();
    expect(within(section as HTMLElement).queryByText(/156/)).not.toBeInTheDocument();
    expect(within(section as HTMLElement).queryByText(/countries/i)).not.toBeInTheDocument();
    expect(within(section as HTMLElement).queryByText(/global network/i)).not.toBeInTheDocument();
    expect(within(section as HTMLElement).queryByText(/worldwide connectivity/i)).not.toBeInTheDocument();
    expect(within(section as HTMLElement).queryByText(/the world is only the beginning/i)).not.toBeInTheDocument();
  });

  it("renders the texture-based Moon visual with photographic texture, glow, and trajectory", () => {
    const { container } = render(<FutureMoonStory />);

    expect(container.querySelector("[data-moon-sphere]")).toBeInTheDocument();
    expect(container.querySelector("[data-moon-surface]")).toBeInTheDocument();
    expect(container.querySelector("[data-moon-trajectory]")).toBeInTheDocument();
    expect(container.querySelector("[data-moon-trajectory-svg]")).toBeInTheDocument();
    expect(container.querySelector("[data-moon-glow]")).toBeInTheDocument();

    // Verify the Moon uses a dedicated photographic texture asset (single image, no tiling)
    const textureImages = container.querySelectorAll<HTMLImageElement>(
      "[data-moon-surface] img",
    );
    expect(textureImages).toHaveLength(1);
    textureImages.forEach((img) => {
      expect(img.getAttribute("src") ?? img.getAttribute("srcset") ?? "").toBeTruthy();
    });
  });

  it("applies hover scale interaction via CSS class on the moon sphere", () => {
    const { container } = render(<FutureMoonStory />);
    const sphere = container.querySelector("[data-moon-sphere]");

    expect(sphere).not.toBeNull();
    // The sphere wrapper should have the hover scale class
    expect(sphere?.className).toContain("hover:scale-");
  });

  it("isolates the photographic Moon from its JPEG perimeter and keeps the route behind it", () => {
    const { container } = render(<FutureMoonStory />);
    const surface = container.querySelector<HTMLElement>("[data-moon-surface]");
    const texture = surface?.querySelector<HTMLImageElement>("img");
    const trajectory = container.querySelector<SVGElement>(
      "[data-moon-trajectory-svg]",
    );
    const sphere = container.querySelector<HTMLElement>("[data-moon-sphere]");

    expect(surface).not.toBeNull();
    expect(surface?.className).toContain("overflow-hidden");
    expect(surface?.style.maskImage).toContain("radial-gradient");
    expect(texture?.className).toContain("scale-");

    expect(trajectory).not.toBeNull();
    expect(sphere).not.toBeNull();
    expect(Number(getComputedStyle(trajectory as SVGElement).zIndex)).toBeLessThan(
      Number(getComputedStyle(sphere as HTMLElement).zIndex),
    );

    expect(container.querySelectorAll("[data-moon-aura]")).toHaveLength(3);
    expect(sphere?.className).toContain("motion-safe:hover:scale-[1.04]");
    expect(sphere?.className).toContain("motion-reduce:transition-none");
  });

  it("contains no booking controls, flight search forms, fares, prices, or reservations", () => {
    const { container } = render(<FutureMoonStory />);
    const section = container.querySelector("#future-moon");

    expect(section).not.toBeNull();
    expect(within(section as HTMLElement).queryByRole("combobox")).not.toBeInTheDocument();
    expect(within(section as HTMLElement).queryByRole("textbox")).not.toBeInTheDocument();
    expect(within(section as HTMLElement).queryByRole("spinbutton")).not.toBeInTheDocument();
    expect(within(section as HTMLElement).queryByRole("radio")).not.toBeInTheDocument();
    expect(within(section as HTMLElement).queryByText(/book moon/i)).not.toBeInTheDocument();
    expect(within(section as HTMLElement).queryByText(/\$\d+/)).not.toBeInTheDocument();
    expect(within(section as HTMLElement).queryByText(/select date/i)).not.toBeInTheDocument();
  });

  it("preserves single H1 hierarchy when integrated into Home page", () => {
    render(<Home />);

    const h1s = screen.getAllByRole("heading", { level: 1 });
    expect(h1s).toHaveLength(1);

    expect(
      screen.getByRole("heading", {
        level: 2,
        name: /NEXT:\s+THE MOON\./i,
      }),
    ).toBeInTheDocument();
  });
});
