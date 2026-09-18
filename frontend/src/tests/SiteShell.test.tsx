import { screen } from "@testing-library/react";

import { ReviewPage } from "@/components/booking/review/ReviewPage";
import { SiteShell } from "@/components/layout/SiteShell";
import { ManageBookingPage } from "@/components/manage-booking/ManageBookingPage";
import { render } from "@/tests/renderWithLanguage";

describe("customer shell landmarks", () => {
  it.each([
    ["manage booking", <ManageBookingPage key="manage-booking" />],
    ["review recovery", <ReviewPage backQuery="" holdId="" key="review-recovery" />],
  ])("renders one main landmark for the %s surface", (_surface, page) => {
    render(<SiteShell>{page}</SiteShell>);

    expect(screen.getAllByRole("main")).toHaveLength(1);
    expect(screen.getByRole("main")).toHaveAttribute("id", "main-content");
  });

  it("provides a localized skip link before the customer shell", () => {
    const { container } = render(
      <SiteShell>
        <section aria-labelledby="surface-heading">
          <h1 id="surface-heading">Customer surface</h1>
        </section>
      </SiteShell>,
    );

    const skipLink = screen.getByRole("link", { name: "Skip to main content" });

    expect(skipLink).toHaveAttribute("href", "#main-content");
    expect(skipLink).toHaveClass("sr-only", "focus:not-sr-only");
    expect(container.firstElementChild).toBe(skipLink);
  });
});
