import { render, screen } from "@testing-library/react";

import { Button } from "@/components/ui/Button";

describe("Button", () => {
  it("exposes its accessible name as a native button", () => {
    render(<Button>Review foundation</Button>);

    expect(
      screen.getByRole("button", { name: "Review foundation" }),
    ).toBeInTheDocument();
  });

  it("prevents interaction and exposes busy state while loading", () => {
    render(<Button loading>Save changes</Button>);

    const button = screen.getByRole("button", { name: "Save changes" });

    expect(button).toBeDisabled();
    expect(button).toHaveAttribute("aria-busy", "true");
  });

  it("keeps destructive actions visually distinct from primary actions", () => {
    render(<Button variant="destructive">Delete draft</Button>);

    expect(
      screen.getByRole("button", { name: "Delete draft" }),
    ).toHaveClass("bg-destructive");
  });
});
