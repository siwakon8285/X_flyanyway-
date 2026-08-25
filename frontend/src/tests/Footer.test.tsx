import { render, screen } from "@testing-library/react";

import { Footer } from "@/components/layout/Footer";

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
    expect(screen.getByRole("link", { name: "About X-Fly" })).toHaveAttribute(
      "href",
      "#journey-experience",
    );
  });
});
