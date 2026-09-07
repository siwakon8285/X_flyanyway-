import type { DashboardData } from "@/lib/admin/dashboardTypes";

export const dashboardFixture: DashboardData = {
  from: "2026-09-01", to: "2026-09-02", timeZone: "Asia/Bangkok", currency: "THB", provider: "STRIPE", generatedAt: "2026-09-02T12:00:00Z",
  summary: { grossRevenue: 30000, totalBookings: 3, ticketsIssued: 3, cancelledBookings: 1, cancellationRatePercent: 100 / 3, refundCount: 1, refundValue: 8000, pendingRefundCount: 0, pendingRefundValue: 0, attentionRefundCount: 0, averageBookingValue: 10000 },
  trends: [{ date: "2026-09-01", bookings: 1, revenue: 8000 }, { date: "2026-09-02", bookings: 2, revenue: 22000 }],
  routes: [{ route: "BKK-NRT", bookings: 3, revenue: 30000 }],
  cabins: [{ cabin: "economy", bookings: 2, revenue: 16000 }, { cabin: "business", bookings: 1, revenue: 14000 }],
  flights: [{ flightNumber: "XF101", route: "BKK-NRT", departureDate: "2026-09-02", bookings: 3, revenue: 30000 }],
  revenueFlights: [{ flightNumber: "XF101", route: "BKK-NRT", departureDate: "2026-09-02", bookings: 3, revenue: 30000 }],
  inventory: { bookedSeats: 4, sellableSeats: 20, occupancyPercent: 20, flights: [{ flightNumber: "XF102", route: "BKK-NRT", departureDate: "2026-09-02", bookedSeats: 0, sellableSeats: 10, occupancyPercent: 0 }] },
  availableRoutes: ["BKK-NRT", "BKK-HKT"],
};
