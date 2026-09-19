export const dashboardReportPeriods = ["daily", "weekly", "monthly", "custom"] as const;
export type DashboardReportPeriod = typeof dashboardReportPeriods[number];

export type DashboardPeriodRange = { from: string; to: string };

const dashboardTimeZone = "Asia/Bangkok";
const bangkokDateFormatter = new Intl.DateTimeFormat("en-US", {
  timeZone: dashboardTimeZone,
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
});

const bangkokDate = (now: Date) => {
  const parts = Object.fromEntries(
    bangkokDateFormatter.formatToParts(now).map(({ type, value }) => [type, value]),
  );
  return `${parts.year}-${parts.month}-${parts.day}`;
};

const shiftDate = (date: string, days: number) => {
  const shifted = new Date(`${date}T00:00:00Z`);
  shifted.setUTCDate(shifted.getUTCDate() + days);
  return shifted.toISOString().slice(0, 10);
};

export function resolveDashboardPeriod(
  period: DashboardReportPeriod,
  now = new Date(),
  customRange?: DashboardPeriodRange,
): DashboardPeriodRange {
  const today = bangkokDate(now);
  if (period === "custom") return customRange ? { from: customRange.from, to: customRange.to } : { from: today, to: today };
  if (period === "daily") return { from: today, to: today };
  if (period === "monthly") return { from: `${today.slice(0, 8)}01`, to: today };

  const dayOfWeek = new Date(`${today}T00:00:00Z`).getUTCDay();
  const daysSinceMonday = (dayOfWeek + 6) % 7;
  return { from: shiftDate(today, -daysSinceMonday), to: today };
}
