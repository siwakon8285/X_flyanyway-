import { fireEvent, screen, waitFor } from "@testing-library/react";
import { render } from "@/tests/renderWithLanguage";

import { Header } from "@/components/layout/Header";

describe("Header", () => {
  it("renders the public brand, semantic navigation, and booking CTA", () => {
    render(<Header />);

    expect(
      screen.getByRole("link", { name: "X-Fly Anyway home" }),
    ).toBeInTheDocument();

    const navigation = screen.getByRole("navigation", {
      name: "Primary navigation",
    });

    expect(navigation).toContainElement(
      screen.getByRole("link", { name: "Explore" }),
    );
    expect(navigation).toContainElement(
      screen.getByRole("link", { name: "Destinations" }),
    );
    expect(navigation).toContainElement(
      screen.getByRole("link", { name: "Cabins" }),
    );
    expect(navigation).toContainElement(
      screen.getByRole("link", { name: "Experience" }),
    );
    expect(screen.getByRole("link", { name: "Book a Flight" })).toHaveAttribute(
      "href",
      "/#flight-search",
    );
    expect(screen.getByRole("link", { name: "Explore" })).toHaveAttribute(
      "href",
      "#global-reach",
    );
    expect(screen.getByRole("link", { name: "Destinations" })).toHaveAttribute(
      "href",
      "#global-reach",
    );
    expect(screen.getByRole("link", { name: "Cabins" })).toHaveAttribute(
      "href",
      "#cabins",
    );
    expect(screen.getByRole("link", { name: "Experience" })).toHaveAttribute(
      "href",
      "#journey-experience",
    );
  });

  it("renders desktop and mobile navigation without React warnings", () => {
    const consoleError = jest.spyOn(console, "error").mockImplementation(() => undefined);

    try {
      render(<Header />);
      fireEvent.click(screen.getByRole("button", { name: "Open navigation menu" }));

      expect(consoleError).not.toHaveBeenCalled();
    } finally {
      consoleError.mockRestore();
    }
  });

  it("opens mobile navigation and returns focus to its trigger after Escape", async () => {
    render(<Header />);

    const trigger = screen.getByRole("button", { name: "Open navigation menu" });
    expect(trigger).toHaveAttribute("aria-controls", "mobile-navigation-menu");
    expect(trigger).toHaveAttribute("aria-expanded", "false");

    trigger.focus();
    fireEvent.click(trigger);

    expect(trigger).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByRole("dialog", { name: "Navigation" })).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: /close/i })).toHaveLength(1);
    expect(
      screen.getByRole("button", { name: "Close navigation menu" }),
    ).toBeInTheDocument();

    fireEvent.keyDown(document, { key: "Escape" });

    await waitFor(() => {
      expect(screen.queryByRole("dialog", { name: "Navigation" })).not.toBeInTheDocument();
      expect(trigger).toHaveAttribute("aria-expanded", "false");
      expect(trigger).toHaveFocus();
    });
  });

  it("remains operable across repeated open and close cycles", async () => {
    render(<Header />);

    const trigger = screen.getByRole("button", { name: "Open navigation menu" });

    for (const closeWithEscape of [true, false]) {
      trigger.focus();
      fireEvent.click(trigger);
      expect(screen.getByRole("dialog", { name: "Navigation" })).toBeInTheDocument();

      if (closeWithEscape) {
        fireEvent.keyDown(document, { key: "Escape" });
      } else {
        fireEvent.click(screen.getByRole("button", { name: "Close navigation menu" }));
      }

      await waitFor(() => {
        expect(screen.queryByRole("dialog", { name: "Navigation" })).not.toBeInTheDocument();
        expect(trigger).toHaveAttribute("aria-expanded", "false");
        expect(trigger).toHaveFocus();
      });
    }
  });
});
