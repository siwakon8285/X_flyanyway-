import { render, screen } from "@testing-library/react";

import { Container } from "@/components/layout/Container";

describe("Container", () => {
  it("renders content inside the shared content-width boundary", () => {
    render(<Container>Foundation content</Container>);

    expect(screen.getByText("Foundation content")).toHaveClass(
      "max-w-content",
      "px-page-gutter",
    );
  });

  it("supports the reading-width boundary without dropping custom classes", () => {
    render(
      <Container className="test-marker" size="reading">
        Editorial copy
      </Container>,
    );

    expect(screen.getByText("Editorial copy")).toHaveClass(
      "max-w-reading",
      "test-marker",
    );
  });
});
