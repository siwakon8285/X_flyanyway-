import { fireEvent, screen } from "@testing-library/react";

import { DashboardFilters, presetFilters } from "@/components/admin/dashboard/DashboardFilters";
import type { DashboardReportPeriod } from "@/components/admin/dashboard/dashboardPeriods";
import type { DashboardFilters as Filters } from "@/lib/admin/dashboardTypes";
import { render } from "@/tests/renderWithLanguage";

const baseFilters: Filters = {
  from: "2026-09-10",
  to: "2026-09-12",
  route: "BKK-NRT",
  cabin: "business",
  provider: "MOCK_BITCOIN",
};

const periodFilters = (period: DashboardReportPeriod, current = baseFilters) => presetFilters(period, current);

describe("Dashboard report periods", () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it("resolves Daily, Weekly, and Monthly using the Bangkok calendar", () => {
    jest.setSystemTime(new Date("2026-09-16T05:00:00.000Z"));

    expect(periodFilters("daily")).toMatchObject({ from: "2026-09-16", to: "2026-09-16" });
    expect(periodFilters("weekly")).toMatchObject({ from: "2026-09-14", to: "2026-09-16" });
    expect(periodFilters("monthly")).toMatchObject({ from: "2026-09-01", to: "2026-09-16" });
  });

  it.each([
    ["Monday", "2026-09-14T05:00:00.000Z", "2026-09-14", "2026-09-14"],
    ["the first day of a month", "2026-09-01T05:00:00.000Z", "2026-08-31", "2026-09-01"],
    ["the January year boundary", "2026-01-01T05:00:00.000Z", "2025-12-29", "2026-01-01"],
  ])("handles %s without using future dates", (_description, instant, expectedWeeklyFrom, expectedTo) => {
    jest.setSystemTime(new Date(instant));

    expect(periodFilters("weekly")).toMatchObject({ from: expectedWeeklyFrom, to: expectedTo });
  });

  it("uses the Bangkok calendar date near a UTC day boundary", () => {
    jest.setSystemTime(new Date("2026-09-15T17:30:00.000Z"));

    expect(periodFilters("daily")).toMatchObject({ from: "2026-09-16", to: "2026-09-16" });
  });

  it("keeps Monthly at the first day when Bangkok is on the first day of the month", () => {
    jest.setSystemTime(new Date("2026-01-01T00:30:00.000Z"));

    expect(periodFilters("monthly")).toMatchObject({ from: "2026-01-01", to: "2026-01-01" });
  });

  it("preserves manually selected Custom dates and existing filters", () => {
    jest.setSystemTime(new Date("2026-09-16T05:00:00.000Z"));

    expect(periodFilters("custom")).toEqual(baseFilters);
    expect(periodFilters("weekly")).toMatchObject({
      route: "BKK-NRT",
      cabin: "business",
      provider: "MOCK_BITCOIN",
    });
  });

  it("renders formal English period labels and applies a selected period", () => {
    jest.setSystemTime(new Date("2026-09-16T05:00:00.000Z"));
    const onApply = jest.fn();

    render(<DashboardFilters initial={baseFilters} loading={false} routes={["BKK-NRT"]} onApply={onApply} />);

    expect(screen.getByRole("button", { name: "Daily" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Weekly" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Monthly" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Custom" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Weekly" }));
    expect(onApply).toHaveBeenCalledWith({
      ...baseFilters,
      from: "2026-09-14",
      to: "2026-09-16",
    });
  });

  it("renders formal Thai period labels", () => {
    render(<DashboardFilters initial={baseFilters} loading={false} routes={[]} onApply={jest.fn()} />, { locale: "th" });

    expect(screen.getByRole("button", { name: "รายวัน" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "รายสัปดาห์" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "รายเดือน" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "กำหนดเอง" })).toBeInTheDocument();
  });
});
