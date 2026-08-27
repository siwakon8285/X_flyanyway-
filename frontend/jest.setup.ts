import "@testing-library/jest-dom";

Object.defineProperty(window, "matchMedia", {
  configurable: true,
  value: jest.fn().mockImplementation((query: string) => ({
    matches: query === "(prefers-reduced-motion: reduce)",
    media: query,
    onchange: null,
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    addListener: jest.fn(),
    removeListener: jest.fn(),
    dispatchEvent: jest.fn(),
  })),
  writable: true,
});

Object.defineProperty(window, "scrollTo", {
  configurable: true,
  value: jest.fn(),
  writable: true,
});

Object.defineProperty(Element.prototype, "scrollIntoView", {
  configurable: true,
  value: jest.fn(),
  writable: true,
});

class MockIntersectionObserver implements IntersectionObserver {
  readonly root: Element | Document | null = null;
  readonly rootMargin: string = "";
  readonly thresholds: readonly number[] = [];
  observe = jest.fn();
  unobserve = jest.fn();
  disconnect = jest.fn();
  takeRecords = () => [];
}

Object.defineProperty(window, "IntersectionObserver", {
  configurable: true,
  value: MockIntersectionObserver,
  writable: true,
});
