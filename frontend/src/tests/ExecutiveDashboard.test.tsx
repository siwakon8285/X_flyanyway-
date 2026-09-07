import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { ExecutiveDashboard } from "@/components/admin/dashboard/ExecutiveDashboard";
import { render } from "@/tests/renderWithLanguage";
import { dashboardFixture } from "@/tests/fixtures/dashboard";

const response = (data = dashboardFixture, status = 200) => ({ ok: status === 200, status, json: async () => data } as Response);
const setChartBounds = (chart: HTMLElement | SVGElement) => jest.spyOn(chart, "getBoundingClientRect").mockReturnValue({ bottom: 278, height: 278, left: 0, right: 680, top: 0, width: 680, x: 0, y: 0, toJSON: () => ({}) });
const fireChartPointer = (chart: HTMLElement | SVGElement, type: "pointerdown" | "pointermove", clientX: number, pointerType: "mouse" | "touch") => {
  const event = new MouseEvent(type, { bubbles: true, clientX });
  Object.defineProperty(event, "pointerType", { value: pointerType });
  fireEvent(chart, event);
};

describe("Executive Dashboard", () => {
  beforeEach(() => { global.fetch = jest.fn().mockResolvedValue(response()); });
  it("renders authoritative KPIs, chart values and route/cabin/flight aggregates", async () => {
    const { container } = render(<ExecutiveDashboard />);
    expect(await screen.findByRole("heading", { name: "Commercial performance" })).toBeInTheDocument();
    const summary = await screen.findByRole("region", { name: "Executive summary" });
    expect(within(summary).getByText("THB 30,000")).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Gross booking revenue trend" })).toHaveAttribute("data-values", "8000,22000");
    expect(screen.getByRole("img", { name: "Booking demand trend" })).toHaveAttribute("data-values", "1,2");
    expect(container.querySelector(".exec-revenue-chart [data-chart-line]")?.getAttribute("d")).toContain("C");
    expect(screen.getByRole("img", { name: "Instrumented cabin mix by bookings" })).toHaveAttribute("data-values", "2,0,1,0");
    expect(screen.getByRole("img", { name: "Recorded occupancy 20%: 4 booked seats from 20 sellable seats" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "30 days" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByText("Stripe · test mode", { selector: "p" })).toBeInTheDocument();
    expect(screen.getAllByText("XF101").length).toBeGreaterThan(0);
  });
  it("snaps revenue inspection to authoritative daily points and updates the exact date and value", async () => {
    render(<ExecutiveDashboard />);
    const chart = await screen.findByRole("img", { name: "Gross booking revenue trend" });
    setChartBounds(chart);
    fireChartPointer(chart, "pointermove", 300, "mouse");
    let tooltip = screen.getByTestId("revenue-inspection");
    expect(within(tooltip).getByText("Date")).toBeInTheDocument();
    expect(within(tooltip).getByText("01 Sept 2026")).toBeInTheDocument();
    expect(within(tooltip).getByText("Gross Booking Revenue")).toBeInTheDocument();
    expect(within(tooltip).getByText("THB 8,000")).toBeInTheDocument();
    fireChartPointer(chart, "pointermove", 400, "mouse");
    tooltip = screen.getByTestId("revenue-inspection");
    expect(within(tooltip).getByText("02 Sept 2026")).toBeInTheDocument();
    expect(within(tooltip).getByText("THB 22,000")).toBeInTheDocument();
  });
  it("supports touch and single-tab-stop keyboard inspection for booking demand", async () => {
    render(<ExecutiveDashboard />);
    const chart = await screen.findByRole("img", { name: "Booking demand trend" });
    setChartBounds(chart);
    fireChartPointer(chart, "pointerdown", 70, "touch");
    let tooltip = screen.getByTestId("bookings-inspection");
    expect(within(tooltip).getByText("01 Sept 2026")).toBeInTheDocument();
    expect(within(tooltip).getByText("Bookings")).toBeInTheDocument();
    expect(within(tooltip).getByText("1")).toBeInTheDocument();
    act(() => chart.focus());
    fireEvent.keyDown(chart, { key: "Home" });
    expect(within(screen.getByTestId("bookings-inspection")).getByText("1")).toBeInTheDocument();
    fireEvent.keyDown(chart, { key: "ArrowRight" });
    tooltip = screen.getByTestId("bookings-inspection");
    expect(within(tooltip).getByText("02 Sept 2026")).toBeInTheDocument();
    expect(within(tooltip).getByText("2")).toBeInTheDocument();
    expect(chart).toHaveFocus();
  });
  it("applies date, route, cabin and provider filters to the backend request", async () => {
    render(<ExecutiveDashboard />);
    await screen.findByRole("region", { name: "Executive summary" });
    fireEvent.change(screen.getByLabelText("From"), { target: { value: "2026-09-01" } });
    fireEvent.change(screen.getByLabelText("To"), { target: { value: "2026-09-02" } });
    fireEvent.change(screen.getByLabelText("Route"), { target: { value: "BKK-NRT" } });
    fireEvent.change(screen.getByLabelText("Cabin"), { target: { value: "business" } });
    fireEvent.change(screen.getByLabelText("Payment records"), { target: { value: "MOCK_BITCOIN" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply filters" }));
    await waitFor(() => expect(fetch).toHaveBeenLastCalledWith("/admin/api/dashboard?from=2026-09-01&to=2026-09-02&route=BKK-NRT&cabin=business&provider=MOCK_BITCOIN", expect.objectContaining({ cache: "no-store" })));
  });
  it("shows loading without invented metrics", () => {
    jest.mocked(fetch).mockReturnValue(new Promise(() => {}));
    render(<ExecutiveDashboard />);
    expect(screen.getByRole("status")).toHaveTextContent("Loading commercial performance");
    expect(screen.queryByText("THB 30,000")).not.toBeInTheDocument();
  });
  it("switches the accessible flight ranking board without changing its semantics", async () => {
    render(<ExecutiveDashboard />);
    await screen.findByRole("region", { name: "Executive summary" });
    const mostBooked = screen.getByRole("tab", { name: "Most booked" });
    const highestRevenue = screen.getByRole("tab", { name: "Highest revenue" });
    expect(mostBooked).toHaveAttribute("aria-selected", "true");
    fireEvent.click(highestRevenue);
    expect(highestRevenue).toHaveAttribute("aria-selected", "true");
    expect(mostBooked).toHaveAttribute("aria-selected", "false");
    expect(screen.getByRole("tabpanel", { name: "Highest revenue" })).toContainElement(screen.getByRole("rowheader", { name: /XF101/ }));
    fireEvent.keyDown(highestRevenue, { key: "ArrowLeft" });
    expect(mostBooked).toHaveAttribute("aria-selected", "true");
    expect(mostBooked).toHaveFocus();
  });
  it("keeps the previous authoritative snapshot visible while a new cohort loads", async () => {
    let resolveRefresh!: (value: Response) => void;
    jest.mocked(fetch).mockResolvedValueOnce(response()).mockReturnValueOnce(new Promise((resolve) => { resolveRefresh = resolve; }));
    render(<ExecutiveDashboard />);
    await screen.findByRole("region", { name: "Executive summary" });
    fireEvent.click(screen.getByRole("button", { name: "7 days" }));
    expect(screen.getAllByText("THB 30,000").length).toBeGreaterThan(0);
    expect(screen.getByText("Updating intelligence")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Updating" })).toBeDisabled();
    await act(async () => resolveRefresh(response()));
  });
  it("replaces both chart signals with the newly filtered authoritative series", async () => {
    const refreshed = { ...dashboardFixture, generatedAt: "2026-09-02T12:01:00Z", trends: [{ date: "2026-09-02", bookings: 5, revenue: 41000 }] };
    jest.mocked(fetch).mockResolvedValueOnce(response()).mockResolvedValueOnce(response(refreshed));
    render(<ExecutiveDashboard />);
    await screen.findByRole("region", { name: "Executive summary" });
    fireEvent.click(screen.getByRole("button", { name: "7 days" }));
    await waitFor(() => expect(screen.getByRole("img", { name: "Gross booking revenue trend" })).toHaveAttribute("data-values", "41000"));
    expect(screen.getByRole("img", { name: "Booking demand trend" })).toHaveAttribute("data-values", "5");
  });
  it("clears inspected data on filter refresh and only inspects the replacement dataset", async () => {
    const refreshed = { ...dashboardFixture, generatedAt: "2026-09-03T12:01:00Z", trends: [{ date: "2026-09-03", bookings: 5, revenue: 41000 }] };
    let resolveRefresh!: (value: Response) => void;
    jest.mocked(fetch).mockResolvedValueOnce(response()).mockReturnValueOnce(new Promise((resolve) => { resolveRefresh = resolve; }));
    render(<ExecutiveDashboard />);
    const initialChart = await screen.findByRole("img", { name: "Gross booking revenue trend" });
    setChartBounds(initialChart);
    fireChartPointer(initialChart, "pointermove", 70, "mouse");
    expect(screen.getByTestId("revenue-inspection")).toHaveTextContent("01 Sept 2026");
    fireEvent.click(screen.getByRole("button", { name: "7 days" }));
    expect(screen.queryByTestId("revenue-inspection")).not.toBeInTheDocument();
    await act(async () => resolveRefresh(response(refreshed)));
    await waitFor(() => expect(screen.getByRole("img", { name: "Gross booking revenue trend" })).toHaveAttribute("data-values", "41000"));
    const refreshedChart = screen.getByRole("img", { name: "Gross booking revenue trend" });
    setChartBounds(refreshedChart);
    fireChartPointer(refreshedChart, "pointermove", 340, "mouse");
    expect(screen.getByTestId("revenue-inspection")).toHaveTextContent("03 Sept 2026");
    expect(screen.getByTestId("revenue-inspection")).toHaveTextContent("THB 41,000");
    expect(screen.getByTestId("revenue-inspection")).not.toHaveTextContent("01 Sept 2026");
  });
  it("removes previously visible analytics if refreshed authorization is rejected", async () => {
    jest.mocked(fetch).mockResolvedValueOnce(response()).mockResolvedValueOnce(response(dashboardFixture, 403));
    render(<ExecutiveDashboard />);
    await screen.findByRole("region", { name: "Executive summary" });
    fireEvent.click(screen.getByRole("button", { name: "7 days" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("You do not have permission");
    expect(screen.queryByText("THB 30,000")).not.toBeInTheDocument();
  });
  it("provides an honest empty state and unavailable ratios", async () => {
    jest.mocked(fetch).mockResolvedValue(response({ ...dashboardFixture, summary: { grossRevenue: 0, totalBookings: 0, ticketsIssued: 0, cancelledBookings: 0, cancellationRatePercent: null, refundCount: 0, refundValue: 0, pendingRefundCount: 0, pendingRefundValue: 0, attentionRefundCount: 0, averageBookingValue: null }, trends: [], routes: [], cabins: [], flights: [], revenueFlights: [], inventory: { bookedSeats: 0, sellableSeats: 0, occupancyPercent: null, flights: [] } }));
    render(<ExecutiveDashboard />);
    expect(await screen.findByText("No successful bookings in this selection.")).toBeInTheDocument();
    expect(screen.queryByText(/leading route/i)).not.toBeInTheDocument();
    expect(screen.getAllByText("—").length).toBeGreaterThan(0);
  });
  it.each([401, 403, 422, 500])("handles HTTP %s without retaining sensitive analytics", async (status) => {
    jest.mocked(fetch).mockResolvedValue(response(dashboardFixture, status));
    render(<ExecutiveDashboard />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
    expect(screen.queryByText("THB 30,000")).not.toBeInTheDocument();
    if (status === 401) expect(screen.getByRole("link", { name: "Sign in again" })).toHaveAttribute("href", "/admin/login");
  });
  it("renders Thai labels and localized chart/table alternatives", async () => {
    render(<ExecutiveDashboard />, { locale: "th" });
    expect(await screen.findByRole("region", { name: "สรุปสำหรับผู้บริหาร" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "ผลการดำเนินงานเชิงพาณิชย์" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "ใช้ตัวกรอง" })).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "แนวโน้มรายได้รวมจากการจอง" })).toBeInTheDocument();
    expect(screen.getByRole("img", { name: /อัตราการจองที่นั่งที่บันทึก 20%/ })).toBeInTheDocument();
    const chart = screen.getByRole("img", { name: "แนวโน้มจำนวนการจอง" });
    setChartBounds(chart);
    fireChartPointer(chart, "pointermove", 70, "mouse");
    const tooltip = screen.getByTestId("bookings-inspection");
    expect(within(tooltip).getByText("วันที่")).toBeInTheDocument();
    expect(within(tooltip).getByText("01 ก.ย. 2026")).toBeInTheDocument();
    expect(within(tooltip).getByText("การจอง")).toBeInTheDocument();
  });
  it("renders final, usable dashboard content immediately for reduced-motion users", async () => {
    expect(window.matchMedia("(prefers-reduced-motion: reduce)").matches).toBe(true);
    const { container } = render(<ExecutiveDashboard />);
    await screen.findByRole("region", { name: "Executive summary" });
    expect(container.querySelector("[data-exec-title-line]")).not.toHaveAttribute("style");
    expect(container.querySelector("[data-route-line]")).not.toHaveAttribute("style", expect.stringContaining("transform"));
    expect(screen.getByRole("tab", { name: "Most booked" })).toBeEnabled();
  });
  it("discards obsolete responses after filter changes", async () => {
    let resolveFirst!: (value: Response) => void;
    jest.mocked(fetch).mockReturnValueOnce(new Promise((resolve) => { resolveFirst = resolve; }));
    render(<ExecutiveDashboard />);
    fireEvent.click(screen.getByRole("button", { name: "7 days" }));
    await screen.findByRole("region", { name: "Executive summary" });
    await act(async () => resolveFirst(response({ ...dashboardFixture, summary: { ...dashboardFixture.summary, grossRevenue: 99999 } })));
    expect(screen.queryByText("THB 99,999")).not.toBeInTheDocument();
  });
});
