export type DashboardProvider = "STRIPE" | "MOCK_BITCOIN";
export const dashboardCabins = ["economy", "premium-economy", "business", "first"] as const;
export type DashboardCabin = typeof dashboardCabins[number];
export type DashboardFilters = { from: string; to: string; route: string; cabin: string; provider: DashboardProvider };
export type FlightPerformance = { flightNumber: string; route: string; departureDate: string; bookings: number; revenue: number };
export type DashboardData = {
  from: string; to: string; timeZone: "Asia/Bangkok"; currency: "THB"; provider: DashboardProvider; generatedAt: string;
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
  inventory: {
    bookedSeats: number; sellableSeats: number; occupancyPercent: number | null;
    flights: { flightNumber: string; route: string; departureDate: string; bookedSeats: number; sellableSeats: number; occupancyPercent: number | null }[];
  };
  availableRoutes: string[];
};
