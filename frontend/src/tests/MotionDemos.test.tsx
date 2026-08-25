import { fireEvent, render, screen } from "@testing-library/react";

import { FlipDemo } from "@/components/motion/FlipDemo";
import { MotionPresenceDemo } from "@/components/motion/MotionPresenceDemo";

describe("motion demonstrations", () => {
  it("changes Flip layout state without making motion required", () => {
    render(<FlipDemo />);

    const toggle = screen.getByRole("button", { name: "Move demo card" });
    expect(toggle).toHaveAttribute("aria-pressed", "false");

    fireEvent.click(toggle);

    expect(toggle).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByText("Layout card")).toBeInTheDocument();
  });

  it("keeps the Motion UI proof operable with reduced motion", () => {
    render(<MotionPresenceDemo />);

    const toggle = screen.getByRole("button", { name: "Toggle UI notice" });
    expect(screen.getByText("Component-level UI motion")).toBeInTheDocument();

    fireEvent.click(toggle);
    expect(screen.queryByText("Component-level UI motion")).not.toBeInTheDocument();
  });
});
