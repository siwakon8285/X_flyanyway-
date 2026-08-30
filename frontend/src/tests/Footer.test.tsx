import { screen } from "@testing-library/react";
import { render } from "@/tests/renderWithLanguage";

import Home from "@/app/page";
import { Footer } from "@/components/layout/Footer";
import { SiteShell } from "@/components/layout/SiteShell";

describe("Footer", () => {
  it("renders the public footer landmark and navigation groups", () => {
    render(<Footer />);

    expect(screen.getByRole("contentinfo")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Explore" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Travel" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Company" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Support" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Legal" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Privacy" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Terms" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Cabins" })).toHaveAttribute(
      "href",
      "#cabins",
    );
    expect(screen.getByRole("link", { name: "Book a Flight" })).toHaveAttribute(
      "href",
      "/#flight-search",
    );
    expect(screen.getByRole("link", { name: "About X-Fly" })).toHaveAttribute(
      "href",
      "#journey-experience",
    );
  });

  it("renders the final flight search before the footer", () => {
    const { container } = render(
      <SiteShell>
        <Home />
      </SiteShell>,
    );
    const flightSearch = container.querySelector("#flight-search");
    const footer = container.querySelector("footer");

    expect(flightSearch).not.toBeNull();
    expect(footer).not.toBeNull();
    expect(flightSearch?.compareDocumentPosition(footer as Node)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
  });
});
