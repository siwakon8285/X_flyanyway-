export type DashboardProvider = "STRIPE" | "MOCK_BITCOIN";
export const dashboardCabins = ["business", "first"] as const;
export type DashboardCabin = typeof dashboardCabins[number];
export type DashboardFilters = { from: string; to: string; route: string; cabin: string; provider: DashboardProvider };
export type FlightPerformance = { flightNumber: string; route: string; departureDate: string; bookings: number; revenue: number };
export type DashboardNationality = { nationalityCode: string; passengerCount: number; percentage: number };
export type DashboardData = {
  from: string; to: string; timeZone: "Asia/Bangkok"; currency: "THB"; provider: DashboardProvider; generatedAt: string;
  activeCabins: DashboardCabin[];
  summary: {
    grossRevenue: number; totalBookings: number; ticketsIssued: number; cancelledBookings: number;
    cancellationRatePercent: number | null; refundCount: number; refundValue: number;
    pendingRefundCount: number; pendingRefundValue: number; attentionRefundCount: number;
    averageBookingValue: number | null;
  };
  trends: { date: string; bookings: number; revenue: number }[];
  routes: { route: string; bookings: number; revenue: number }[];
  cabins: { cabin: DashboardCabin; bookings: number; revenue: number }[];
  flights: FlightPerformance[]; revenueFlights: FlightPerformance[];
  nationalityDistribution: DashboardNationality[];
  inventory: {
    bookedSeats: number; sellableSeats: number; occupancyPercent: number | null;
    flights: { flightNumber: string; route: string; departureDate: string; bookedSeats: number; sellableSeats: number; occupancyPercent: number | null }[];
  };
  availableRoutes: string[];
};

export function dashboardDataReconciles(data: DashboardData): boolean {
  if (!Array.isArray(data.activeCabins) || !Array.isArray(data.cabins) || !Array.isArray(data.trends) || !Array.isArray(data.nationalityDistribution)) return false;
  if (data.activeCabins.length !== dashboardCabins.length
    || data.activeCabins.some((cabin, index) => cabin !== dashboardCabins[index])) return false;
  if (data.cabins.some((row) => !dashboardCabins.includes(row.cabin))
    || new Set(data.cabins.map((row) => row.cabin)).size !== data.cabins.length) return false;

  const cabinBookings = data.cabins.reduce((sum, row) => sum + row.bookings, 0);
  const cabinRevenue = data.cabins.reduce((sum, row) => sum + row.revenue, 0);
  const trendBookings = data.trends.reduce((sum, row) => sum + row.bookings, 0);
  const trendRevenue = data.trends.reduce((sum, row) => sum + row.revenue, 0);
  if (data.nationalityDistribution.some((row) => !row.nationalityCode || !Number.isInteger(row.passengerCount) || row.passengerCount < 0 || !Number.isFinite(row.percentage) || row.percentage < 0)) return false;
  const expectedAverage = data.summary.totalBookings ? data.summary.grossRevenue / data.summary.totalBookings : null;
  const averageReconciles = expectedAverage === null
    ? data.summary.averageBookingValue === null
    : data.summary.averageBookingValue !== null
      && Math.abs(data.summary.averageBookingValue - expectedAverage) < 0.005;

  return cabinBookings === data.summary.totalBookings
    && cabinRevenue === data.summary.grossRevenue
    && trendBookings === data.summary.totalBookings
    && trendRevenue === data.summary.grossRevenue
    && averageReconciles;
}
