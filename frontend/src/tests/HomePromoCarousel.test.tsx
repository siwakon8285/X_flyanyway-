import { act, fireEvent, screen, within } from "@testing-library/react";

import { HomePage as Home } from "@/components/home/HomePage";
import { render } from "@/tests/renderWithLanguage";

type TestIntersectionObserver = IntersectionObserver & {
  emit: (entries: readonly Partial<IntersectionObserverEntry>[]) => void;
};

const originalIntersectionObserver = window.IntersectionObserver;
const observerRecords: TestIntersectionObserver[] = [];

class ControlledIntersectionObserver implements IntersectionObserver {
  readonly root: Element | Document | null;
  readonly rootMargin: string;
  readonly thresholds: readonly number[];
  private readonly callback: IntersectionObserverCallback;

  constructor(
    callback: IntersectionObserverCallback,
    options: IntersectionObserverInit = {},
  ) {
    this.callback = callback;
    this.root = options.root ?? null;
    this.rootMargin = options.rootMargin ?? "0px";
    this.thresholds = Array.isArray(options.threshold)
      ? options.threshold
      : [options.threshold ?? 0];
    observerRecords.push(this as TestIntersectionObserver);
  }

  observe = jest.fn();
  unobserve = jest.fn();
  disconnect = jest.fn();
  takeRecords = () => [];

  emit(entries: readonly Partial<IntersectionObserverEntry>[]) {
    this.callback(
      entries.map((entry) => entry as IntersectionObserverEntry),
      this,
    );
  }
}

beforeAll(() => {
  Object.defineProperty(window, "IntersectionObserver", {
    configurable: true,
    value: ControlledIntersectionObserver,
    writable: true,
  });
});

afterAll(() => {
  Object.defineProperty(window, "IntersectionObserver", {
    configurable: true,
    value: originalIntersectionObserver,
    writable: true,
  });
});

beforeEach(() => {
  observerRecords.length = 0;
});

const getPromoTrack = () => {
  const track = screen
    .getByRole("region", { name: /Featured promotions|โปรโมชั่นแนะนำ/ })
    .querySelector("[data-promo-track]");
  expect(track).toBeInTheDocument();
  return track as HTMLElement;
};

const getPromoSlides = (track: HTMLElement) =>
  Array.from(track.querySelectorAll<HTMLElement>("[data-promo-slide]"));

const getPromoObserver = (track: HTMLElement) => {
  const observer = observerRecords.find((record) => record.root === track);
  if (!observer) {
    throw new Error("Promotion track IntersectionObserver was not created");
  }
  return observer;
};

const mockTrackScrolling = (track: HTMLElement) => {
  const scrollTo = jest.fn();
  const slides = getPromoSlides(track);

  Object.defineProperty(track, "scrollTo", {
    configurable: true,
    value: scrollTo,
  });
  slides.forEach((slide, index) => {
    Object.defineProperty(slide, "offsetLeft", {
      configurable: true,
      value: index * 320,
    });
  });

  return scrollTo;
};

const advanceToMoonSlide = (nextLabel = "Next promotion") => {
  const nextButton = screen.getByRole("button", { name: nextLabel });
  fireEvent.click(nextButton);
  fireEvent.click(nextButton);
  fireEvent.click(nextButton);
};

describe("homepage promotion carousel", () => {
  it("replaces the former featured journey card with an accessible carousel", () => {
    render(<Home />);

    expect(
      screen.getByRole("region", { name: "Featured promotions" }),
    ).toBeInTheDocument();
    expect(screen.queryByText("Featured journey")).not.toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Travel, elevated." })).toBeInTheDocument();
    expect(screen.getByText("PREMIUM CAMPAIGN")).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Featured promotions" }),
    ).toHaveAttribute("id", "offers");

    const signature = screen.getByText("X-Fly Signature");
    expect(signature).toHaveAttribute("data-x-fly-signature");
    expect(signature).toHaveClass(
      "motion-safe:hover:scale-[1.025]",
      "motion-reduce:transform-none",
    );
  });

  it("renders all four promotions in the native below-lg scroll track", () => {
    render(<Home />);

    const track = getPromoTrack();
    const slides = getPromoSlides(track);

    expect(track).toHaveClass(
      "flex",
      "overflow-x-auto",
      "snap-x",
      "snap-mandatory",
      "overscroll-x-contain",
      "scroll-smooth",
      "lg:overflow-hidden",
    );
    expect(track).not.toHaveClass("touch-pan-y");
    expect(slides).toHaveLength(4);
    expect(slides.map((slide) => slide.dataset.promoSlide)).toEqual([
      "premium",
      "earlyBooking",
      "family",
      "moon",
    ]);
    expect(slides.every((slide) => slide.classList.contains("snap-start"))).toBe(true);
    expect(
      slides.every((slide) => slide.classList.contains("flex-[0_0_100%]")),
    ).toBe(true);
  });

  it("updates the active pagination state when the observed slide becomes dominant", () => {
    render(<Home />);

    const carousel = screen.getByRole("region", { name: "Featured promotions" });
    const track = getPromoTrack();
    const slides = getPromoSlides(track);
    const dots = within(carousel).getAllByRole("button", { name: /Go to promotion/ });

    expect(track).toHaveAttribute("data-promo-active-slide", "premium");
    expect(dots[0]).toHaveAttribute("aria-current", "true");

    act(() => {
      getPromoObserver(track).emit([
        {
          intersectionRatio: 0.9,
          isIntersecting: true,
          target: slides[1],
        },
      ]);
    });

    expect(track).toHaveAttribute("data-promo-active-slide", "earlyBooking");
    expect(dots[1]).toHaveAttribute("aria-current", "true");
    expect(dots[0]).not.toHaveAttribute("aria-current");
  });

  it("scrolls to the selected slide when a dot is activated", () => {
    render(<Home />);

    const carousel = screen.getByRole("region", { name: "Featured promotions" });
    const track = getPromoTrack();
    const scrollTo = mockTrackScrolling(track);
    const dots = within(carousel).getAllByRole("button", { name: /Go to promotion/ });

    fireEvent.click(dots[2]);

    expect(scrollTo).toHaveBeenCalledWith({ behavior: "auto", left: 640 });
    expect(track).toHaveAttribute("data-promo-active-slide", "family");
    expect(dots[2]).toHaveAttribute("aria-current", "true");
  });

  it("preserves desktop arrow navigation and wrapped moveTo behavior", () => {
    render(<Home />);

    const carousel = screen.getByRole("region", { name: "Featured promotions" });
    const track = getPromoTrack();
    const scrollTo = mockTrackScrolling(track);
    const nextButton = within(carousel).getByRole("button", { name: "Next promotion" });
    const previousButton = within(carousel).getByRole("button", { name: "Previous promotion" });

    expect(carousel.querySelector("[data-promo-arrow-controls]")).toHaveClass(
      "hidden",
      "lg:flex",
    );
    expect(nextButton).toBeInTheDocument();
    expect(previousButton).toBeInTheDocument();

    fireEvent.click(nextButton);
    expect(track).toHaveAttribute("data-promo-active-slide", "earlyBooking");

    fireEvent.click(previousButton);
    expect(track).toHaveAttribute("data-promo-active-slide", "premium");

    fireEvent.click(previousButton);
    expect(track).toHaveAttribute("data-promo-active-slide", "moon");
    expect(scrollTo).toHaveBeenCalled();
  });

  it("keeps the Business campaign badge localized in English", () => {
    render(<Home />);
    const englishTrack = getPromoTrack();
    expect(within(getPromoSlides(englishTrack)[0]).getByText("PREMIUM CAMPAIGN")).toBeInTheDocument();
  });

  it("keeps the Business campaign badge localized in Thai", () => {
    render(<Home />, { locale: "th" });
    const thaiTrack = getPromoTrack();
    expect(within(getPromoSlides(thaiTrack)[0]).getByText("แคมเปญพรีเมียม")).toBeInTheDocument();
  });

  it("renders the English Moon campaign as the fourth slide without a booking CTA", () => {
    render(<Home />);
    const slides = getPromoSlides(getPromoTrack());
    const moonSlide = slides[3];

    expect(moonSlide).toHaveAttribute("data-promo-slide", "moon");
    expect(within(moonSlide).getByRole("heading", { name: "Destination, Moon." })).toBeInTheDocument();
    expect(within(moonSlide).getByText("X-Fly Beyond")).toBeInTheDocument();
    expect(within(moonSlide).getByText("BEYOND EARTH · FUTURE JOURNEY")).toBeInTheDocument();
    expect(within(moonSlide).getByText(/A new frontier is approaching\./)).toBeInTheDocument();
    expect(within(moonSlide).getByText("COMING NEXT YEAR")).toBeInTheDocument();
    expect(within(moonSlide).queryByRole("link")).not.toBeInTheDocument();
    expect(moonSlide.querySelector('[data-promo-visual="moon"] img')).toHaveAttribute("alt", "");
  });

  it("preserves the Thai Moon campaign copy without a booking CTA", () => {
    render(<Home />, { locale: "th" });
    const slides = getPromoSlides(getPromoTrack());
    const moonSlide = slides[3];

    expect(within(moonSlide).getByRole("heading", { name: /จุดหมายถัดไป\s+ดวงจันทร์/ })).toBeInTheDocument();
    expect(within(moonSlide).getByText("เหนือขอบฟ้า · การเดินทางแห่งอนาคต")).toBeInTheDocument();
    expect(within(moonSlide).getByText(/อีกหนึ่งก้าวแห่งการเดินทางกำลังจะมาถึง/)).toBeInTheDocument();
    expect(within(moonSlide).getByText("พบกันปีหน้า")).toBeInTheDocument();
    expect(within(moonSlide).queryByRole("link")).not.toBeInTheDocument();
  });

  it("moves to the Moon slide with the existing desktop arrow labels", () => {
    render(<Home />);
    advanceToMoonSlide();

    expect(getPromoTrack()).toHaveAttribute("data-promo-active-slide", "moon");
    expect(screen.getByRole("button", { name: "Previous promotion" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Next promotion" })).toBeInTheDocument();
  });
});
