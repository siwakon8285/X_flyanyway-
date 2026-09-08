import { fireEvent, screen, waitFor, within } from "@testing-library/react";

import { BookingDetailWorkspace } from "@/components/admin/bookings/BookingDetailWorkspace";
import { BookingsWorkspace } from "@/components/admin/bookings/BookingsWorkspace";
import type { BookingDetail, BookingListPage } from "@/lib/admin/bookingTypes";
import { render } from "@/tests/renderWithLanguage";

const detail: BookingDetail = {
  bookingReference: "XF22DDD222", bookingStatus: "CONFIRMED", createdAt: "2026-09-01T08:00:00Z",
  journey: { flightNumber: "XF 880", originCode: "SYD", destinationCode: "CDG", travelDate: "2026-12-20", departureAt: "2026-12-19T23:00:00Z", departureTime: "10:00", arrivalDate: "2026-12-20", arrivalTime: "19:00", originTimeZone: "Australia/Sydney", aircraftCode: "Airbus A350-1000", cabin: "first", flightStatus: "CANCELLED" },
  passengers: [{ ordinal: 1, passengerType: "ADULT", displayName: "History Preserved", gender: "FEMALE" }], seats: ["1K"], contact: { phoneCountryCode: "+66", phoneNumber: "812345678" },
  payment: { method: "CARD", provider: "STRIPE", status: "SUCCEEDED", amount: { amount: 99500, currencyCode: "THB" }, succeededAt: "2026-09-01T08:00:00Z" },
  ticket: { ticketNumber: "XFTABCDEFGHJKL", status: "ISSUED", issuedAt: "2026-09-01T08:01:00Z", cancelledAt: null },
  cancellation: { eligibility: "ELIGIBLE", cutoffAt: "2026-12-18T23:00:00Z", cancelledAt: null, refundStatus: null, refundAmount: null, refundedAt: null }, audit: [],
};

const page: BookingListPage = { items: [{ bookingReference: "XF22BBB222", leadPassengerName: "Legacy Economy", passengerCount: 1, flightNumber: "XF 981", originCode: "SYD", destinationCode: "CDG", travelDate: "2026-12-18", departureAt: "2026-12-17T23:00:00Z", cabin: "economy", bookingStatus: "CONFIRMED", flightStatus: "SCHEDULED", paymentStatus: "SUCCEEDED", ticketStatus: "ISSUED", refundStatus: null, bookedAt: "2026-09-01T08:00:00Z" }], nextOffset: null };
const historicalPage: BookingListPage = {
  items: [
    ...page.items,
    { ...page.items[0], bookingReference:"XF22PPP222", leadPassengerName:"Legacy Premium", cabin:"premium-economy" },
  ],
  nextOffset: null,
};
const mockResponse = (data: unknown, status = 200) => ({ ok: status >= 200 && status < 300, status, json: async () => data }) as Response;

describe("Booking Management", () => {
  beforeEach(() => { global.fetch = jest.fn(); });

  it("searches server-side in a body and renders operational historical-cabin results", async () => {
    jest.mocked(fetch).mockResolvedValue(mockResponse(page));
    render(<BookingsWorkspace />);
    expect(await screen.findByText("XF22BBB222")).toBeInTheDocument();
    expect(screen.getByText("Economy · historical")).toBeInTheDocument();
    expect(screen.getByText("Historical cabin")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Passenger name"), { target: { value: "Legacy Economy" } });
    fireEvent.click(screen.getByRole("button", { name: "Search bookings" }));
    await waitFor(() => expect(fetch).toHaveBeenCalledTimes(2));
    const [url, init] = jest.mocked(fetch).mock.calls[1];
    expect(url).toBe("/admin/api/bookings/search");
    expect(String(url)).not.toContain("Legacy");
    expect(init?.body).toContain("Legacy Economy");
    expect(screen.getByRole("region", { name: "Booking Operations" })).toBeInTheDocument();
  });

  it("offers exactly All, Business and First while unfiltered results retain every historical cabin", async () => {
    jest.mocked(fetch).mockResolvedValue(mockResponse(historicalPage));
    const english = render(<BookingsWorkspace />);
    await screen.findByText("XF22PPP222");
    expect(within(screen.getByLabelText("Cabin")).getAllByRole("option").map((option) => option.textContent)).toEqual(["All", "Business", "First"]);
    expect(screen.queryByRole("option", { name:/Economy/ })).not.toBeInTheDocument();
    expect(screen.queryByRole("option", { name:/Premium Economy/ })).not.toBeInTheDocument();
    expect(await screen.findByText("Economy · historical")).toBeInTheDocument();
    expect(screen.getAllByText("Premium Economy · historical").length).toBeGreaterThan(0);
    english.unmount();

    render(<BookingsWorkspace />, { locale:"th" });
    await screen.findByText("XF22PPP222");
    expect(within(screen.getByLabelText("ชั้นโดยสาร")).getAllByRole("option").map((option) => option.textContent)).toEqual(["ทั้งหมด", "ชั้นธุรกิจ", "ชั้นหนึ่ง"]);
    expect(screen.getAllByText("ชั้นประหยัด · ข้อมูลเดิม").length).toBeGreaterThan(0);
    expect(screen.getAllByText("ชั้นประหยัดพรีเมียม · ข้อมูลเดิม").length).toBeGreaterThan(0);
  });

  it.each([["Business", "business"], ["First", "first"]])("submits the %s cabin filter unchanged", async (_label, cabin) => {
    jest.mocked(fetch).mockResolvedValue(mockResponse(page));
    render(<BookingsWorkspace />);
    await screen.findByText("XF22BBB222");
    fireEvent.change(screen.getByLabelText("Cabin"), { target:{ value:cabin } });
    fireEvent.click(screen.getByRole("button", { name:"Search bookings" }));
    await waitFor(() => expect(fetch).toHaveBeenCalledTimes(2));
    expect(JSON.parse(String(jest.mocked(fetch).mock.calls[1][1]?.body))).toMatchObject({ cabin });
  });

  it("keeps exact booking-reference lookup capable of returning a historical booking", async () => {
    jest.mocked(fetch).mockResolvedValue(mockResponse(page));
    render(<BookingsWorkspace />);
    await screen.findByText("XF22BBB222");
    fireEvent.change(screen.getByLabelText("Booking Reference"), { target:{ value:"XF22BBB222" } });
    fireEvent.click(screen.getByRole("button", { name:"Search bookings" }));
    await waitFor(() => expect(fetch).toHaveBeenCalledTimes(2));
    expect(JSON.parse(String(jest.mocked(fetch).mock.calls[1][1]?.body))).toMatchObject({ bookingReference:"XF22BBB222" });
    expect(await screen.findByText("Economy · historical")).toBeInTheDocument();
  });

  it("reuses the native whole-field date picker and sends the selected travel date", async () => {
    jest.mocked(fetch).mockResolvedValue(mockResponse(page));
    const { container } = render(<BookingsWorkspace />);
    await screen.findByText("XF22BBB222");
    const input = screen.getByLabelText("Travel date") as HTMLInputElement;
    const control = input.closest("[data-date-picker-control]") as HTMLElement;
    const showPicker = jest.fn();
    Object.defineProperty(input, "showPicker", { configurable:true, value:showPicker });

    expect(input).toHaveAttribute("type", "date");
    expect(container.querySelector("[data-calendar-icon]")).toBeInTheDocument();
    fireEvent.click(control);
    expect(showPicker).toHaveBeenCalledTimes(1);
    expect(input).toHaveFocus();

    fireEvent.change(input, { target:{ value:"2026-12-18" } });
    fireEvent.click(screen.getByRole("button", { name:"Search bookings" }));
    await waitFor(() => expect(fetch).toHaveBeenCalledTimes(2));
    expect(JSON.parse(String(jest.mocked(fetch).mock.calls[1][1]?.body))).toMatchObject({ travelDate:"2026-12-18" });
    expect(fireEvent.keyDown(input, { key:"Tab" })).toBe(true);
  });

  it("uses the selected application locale for timestamps and correct passenger grammar", async () => {
    const countPage: BookingListPage = {
      items: [0, 1, 2].map((passengerCount, index) => ({
        ...page.items[0],
        bookingReference: `XF22COUNT${index}`,
        passengerCount,
      })),
      nextOffset: null,
    };
    jest.mocked(fetch).mockResolvedValue(mockResponse(countPage));
    const english = render(<BookingsWorkspace />);
    await screen.findByText("XF22COUNT0");
    const englishTimestamp = english.container.querySelector('time[datetime="2026-09-01T08:00:00Z"]');
    expect(englishTimestamp).toHaveTextContent("2026");
    expect(englishTimestamp).not.toHaveTextContent("2569");
    expect(screen.getByText("0 passengers")).toBeInTheDocument();
    expect(screen.getByText("1 passenger")).toBeInTheDocument();
    expect(screen.getByText("2 passengers")).toBeInTheDocument();
    english.unmount();

    render(<BookingsWorkspace />, { locale:"th" });
    await screen.findByText("XF22COUNT0");
    expect(document.querySelector('time[datetime="2026-09-01T08:00:00Z"]')).toHaveTextContent("2569");
    expect(screen.getByText("ผู้โดยสาร 1 คน")).toBeInTheDocument();
    expect(screen.getByText("ผู้โดยสาร 2 คน")).toBeInTheDocument();
  });

  it("renders localized empty and error states", async () => {
    jest.mocked(fetch).mockResolvedValueOnce(mockResponse({ items: [], nextOffset: null }));
    const view = render(<BookingsWorkspace />, { locale: "th" });
    expect(await screen.findByText("ไม่พบการจอง")).toBeInTheDocument();
    view.unmount();
    jest.mocked(fetch).mockResolvedValueOnce(mockResponse(null, 403));
    render(<BookingsWorkspace />);
    expect(await screen.findByRole("alert")).toHaveTextContent("You do not have permission");
  });

  it("keeps flight, booking, payment and ticket states visibly separate", async () => {
    jest.mocked(fetch).mockResolvedValue(mockResponse(detail));
    render(<BookingDetailWorkspace bookingReference={detail.bookingReference} canManage />);
    expect(await screen.findByRole("heading", { level: 1, name: detail.bookingReference })).toBeInTheDocument();
    expect(screen.getByText("This flight is cancelled. The booking, payment and ticket remain separate authoritative records.")).toBeInTheDocument();
    expect(screen.getAllByText("Confirmed").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Cancelled").length).toBeGreaterThan(0);
    expect(screen.getByText(/99,500/)).toBeInTheDocument();
    expect(screen.getByText("XFTABCDEFGHJKL")).toBeInTheDocument();
    expect(screen.getByText("1K")).toBeInTheDocument();
  });

  it("keeps a historical Premium Economy cabin truthful on exact detail", async () => {
    const historicalDetail: BookingDetail = { ...detail, journey:{ ...detail.journey, cabin:"premium-economy" } };
    jest.mocked(fetch).mockResolvedValue(mockResponse(historicalDetail));
    render(<BookingDetailWorkspace bookingReference={historicalDetail.bookingReference} canManage={false} />);
    await screen.findByRole("heading", { level:1, name:historicalDetail.bookingReference });
    expect(screen.getByText("Premium Economy · historical")).toBeInTheDocument();
    expect(screen.getByText("Historical cabin")).toBeInTheDocument();
  });

  it("requires confirmation and renders only the authoritative cancellation response", async () => {
    const cancelled: BookingDetail = { ...detail, bookingStatus: "CANCELLED", ticket: { ...detail.ticket, status: "CANCELLED", cancelledAt: "2026-09-08T08:00:00Z" }, cancellation: { ...detail.cancellation, eligibility: "UNAVAILABLE", cancelledAt: "2026-09-08T08:00:00Z", refundStatus: "PENDING", refundAmount: { amount: 99500, currencyCode: "THB" } }, audit: [{ action: "STAFF_BOOKING_CANCELLED", actorEmail: "synthetic.operator@x-fly.test", createdAt: "2026-09-08T08:00:00Z" }] };
    jest.mocked(fetch).mockResolvedValueOnce(mockResponse(detail)).mockResolvedValueOnce(mockResponse(cancelled));
    render(<BookingDetailWorkspace bookingReference={detail.bookingReference} canManage />);
    fireEvent.click(await screen.findByRole("button", { name: "Cancel booking / full refund" }));
    expect(screen.getByRole("dialog", { name: `Cancel booking ${detail.bookingReference}?` })).toHaveTextContent("100% refund");
    fireEvent.click(screen.getByRole("button", { name: "Confirm cancellation" }));
    expect(await screen.findByText(/Booking cancelled. The returned authoritative/)).toBeInTheDocument();
    expect(screen.getByText("synthetic.operator@x-fly.test")).toBeInTheDocument();
    const [, init] = jest.mocked(fetch).mock.calls[1];
    expect(init?.method).toBe("POST");
    expect(new Headers(init?.headers).get("x-x-fly-csrf")).toBe("1");
  });

  it("does not expose cancellation controls without manage permission", async () => {
    jest.mocked(fetch).mockResolvedValue(mockResponse(detail));
    render(<BookingDetailWorkspace bookingReference={detail.bookingReference} canManage={false} />);
    await screen.findByRole("heading", { level: 1, name: detail.bookingReference });
    expect(screen.queryByRole("button", { name: "Cancel booking / full refund" })).not.toBeInTheDocument();
  });

  it("keeps the authoritative record and reports a failed mutation without optimistic success", async () => {
    jest.mocked(fetch).mockResolvedValueOnce(mockResponse(detail)).mockResolvedValueOnce(mockResponse(null, 422)).mockResolvedValueOnce(mockResponse(detail));
    render(<BookingDetailWorkspace bookingReference={detail.bookingReference} canManage />);
    fireEvent.click(await screen.findByRole("button", { name: "Cancel booking / full refund" }));
    fireEvent.click(screen.getByRole("button", { name: "Confirm cancellation" }));
    expect(await screen.findByText(/Cancellation was not completed/)).toBeInTheDocument();
    expect(screen.getAllByText("Confirmed").length).toBeGreaterThan(0);
    expect(screen.queryByText(/Booking cancelled. The returned authoritative/)).not.toBeInTheDocument();
  });
});
