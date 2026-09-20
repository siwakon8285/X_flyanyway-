import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";

import { BookingApiError } from "@/components/booking/api/bookingApiClient";
import { ManageBookingDetailsPage } from "@/components/manage-booking/ManageBookingDetailsPage";
import { ManageBookingPage } from "@/components/manage-booking/ManageBookingPage";
import type { ManageBookingDetails } from "@/components/manage-booking/manageBookingTypes";
import { LanguageProvider } from "@/i18n/LanguageProvider";

const mockCurrent = jest.fn();
const mockLookup = jest.fn();
const mockCancel = jest.fn();
const mockReplace = jest.fn();
jest.mock("next/navigation", () => ({ useRouter: () => ({ replace: mockReplace }) }));

jest.mock("@/components/manage-booking/manageBookingClient", () => ({
  getCurrentManageBooking: () => mockCurrent(),
  lookupManageBooking: (input: unknown) => mockLookup(input),
  cancelCurrentManageBooking: () => mockCancel(),
}));

const booking: ManageBookingDetails = {
  bookingReference: "XFABCDEFGH",
  status: "CONFIRMED",
  journey: {
    arrivalDate: "2026-11-04",
    arrivalTime: "16:40",
    cabin: "economy",
    departureAt: "2026-11-04T02:15:00Z",
    departureDate: "2026-11-04",
    departureTime: "09:15",
    departureTimeZone: "Asia/Bangkok",
    destinationCode: "LHR",
    flightNumber: "XF 201",
    originCode: "BKK",
  },
  passengers: [
    { ordinal: 1, displayName: "Nara Van der Meer", travelDocumentStatus: "COMPLETE" },
    { ordinal: 2, displayName: "Mali Van der Meer", travelDocumentStatus: "COMPLETE" },
  ],
  seats: ["20A", "20B"],
  extras: [
    { passengerOrdinal: 1, productCode: "BAG_30KG", category: "BAGGAGE", quantity: 1 },
    { passengerOrdinal: 2, productCode: "MEAL_VEGETARIAN", category: "MEAL", quantity: 1 },
  ],
  payment: { status: "SUCCEEDED", amount: { amount: 49300, currencyCode: "THB" } },
  ticket: { status: "ISSUED", ticketNumber: "XFTABCDEFGHIJKL", issuedAt: "2026-09-04T04:00:00Z" },
  cancellation: { eligibility: "ELIGIBLE", cutoffAt: "2026-11-03T02:15:00Z" },
  qrToken: "v1.12345678-1234-1234-1234-123456789abc.deadbeef",
};

const unauthorized = () =>
  new BookingApiError({
    code: "MANAGE_BOOKING_UNAUTHORIZED",
    message: "authorization required",
    status: 401,
  });

const renderPage = (locale: "en" | "th" = "en", details = false) =>
  render(
    <LanguageProvider initialLocale={locale}>
      {details ? <ManageBookingDetailsPage /> : <ManageBookingPage />}
    </LanguageProvider>,
  );

describe("ManageBookingPage", () => {
  it("always shows lookup even when a previous booking is authorized", async () => {
    mockCurrent.mockResolvedValue(booking);
    renderPage();
    expect(await screen.findByRole("heading", { name: "Manage Booking" })).toBeInTheDocument();
    expect(mockCurrent).not.toHaveBeenCalled();
    expect(screen.queryByText(booking.bookingReference)).not.toBeInTheDocument();
  });
  beforeEach(() => {
    jest.clearAllMocks();
    mockCurrent.mockRejectedValue(unauthorized());
  });

  it("renders an accessible lookup form and validates required fields", async () => {
    renderPage();

    expect(await screen.findByRole("heading", { name: "Manage Booking" })).toBeInTheDocument();
    expect(screen.getByLabelText("Booking Reference")).toHaveAttribute("autocomplete", "off");
    expect(screen.getByLabelText("Last Name")).toHaveAttribute("autocomplete", "family-name");
    expect(screen.queryByLabelText(/email/i)).not.toBeInTheDocument();
    expect(screen.getAllByRole("textbox")).toHaveLength(2);

    fireEvent.click(screen.getByRole("button", { name: "Find Booking" }));
    expect(await screen.findByText("Enter your booking reference.")).toBeInTheDocument();
    expect(screen.getByText("Enter your last name.")).toBeInTheDocument();
    expect(mockLookup).not.toHaveBeenCalled();
  });

  it("keeps lookup fields equal-width on larger screens and stacked on mobile", async () => {
    renderPage();

    await screen.findByRole("heading", { name: "Manage Booking" });
    const bookingReference = screen.getByLabelText("Booking Reference");
    const lastName = screen.getByLabelText("Last Name");
    const bookingReferenceField = bookingReference.parentElement;
    const lastNameField = lastName.parentElement;
    const layout = bookingReferenceField?.parentElement;

    expect(layout).toHaveClass(
      "grid",
      "items-start",
      "gap-6",
      "sm:grid-cols-[repeat(2,minmax(0,1fr))]",
    );
    expect(bookingReferenceField).toHaveClass("min-w-0");
    expect(lastNameField).toHaveClass("min-w-0");
    expect(bookingReference).toHaveClass("h-12", "w-full");
    expect(lastName).toHaveClass("h-12", "w-full");
    expect(screen.getByRole("button", { name: "Find Booking" })).toHaveClass(
      "w-full",
      "sm:w-auto",
    );
  });

  it("uses one generic not-found message for rejected credentials", async () => {
    mockLookup.mockRejectedValue(
      new BookingApiError({
        code: "BOOKING_NOT_FOUND",
        message: "internal message must not be rendered",
        status: 404,
      }),
    );
    renderPage();
    await screen.findByRole("heading", { name: "Manage Booking" });

    fireEvent.change(screen.getByLabelText("Booking Reference"), {
      target: { value: "XFABCDEFGH" },
    });
    fireEvent.change(screen.getByLabelText("Last Name"), {
      target: { value: "SURI" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Find Booking" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "We couldn't find a booking with those details.",
    );
    expect(screen.queryByText("internal message must not be rendered")).not.toBeInTheDocument();
  });

  it("renders authoritative booking sections without sensitive values", async () => {
    mockCurrent.mockResolvedValue(booking);
    renderPage("en", true);

    expect(await screen.findByRole("heading", { name: "Your Booking" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Find another booking" })).toHaveAttribute(
      "href",
      "/manage-booking",
    );
    expect(screen.queryByRole("link", { name: "Return Home" })).not.toBeInTheDocument();
    expect(screen.getByTestId("e-ticket-booking-reference")).toHaveTextContent("XFABCDEFGH");
    expect(screen.getByTestId("e-ticket-origin")).toHaveTextContent("BKK");
    expect(screen.getByTestId("e-ticket-destination")).toHaveTextContent("LHR");
    expect(screen.getByTestId("e-ticket-flight-number")).toHaveTextContent("XF 201");
    expect(screen.getAllByText(/04 Nov 2026/).length).toBeGreaterThanOrEqual(2);
    expect(screen.queryByText("2026-11-04")).not.toBeInTheDocument();
    expect(screen.getByTestId("e-ticket-passengers")).toHaveTextContent("Nara Van der Meer");
    expect(screen.getByTestId("e-ticket-seats")).toHaveTextContent("20A");
    expect(screen.getByText("+30 kg")).toBeInTheDocument();
    expect(screen.getByText("THB 49,300")).toBeInTheDocument();
    expect(screen.getByText("XFTABCDEFGHIJKL")).toBeInTheDocument();
    expect(screen.getByTestId("ticket-qr-container")).toHaveAttribute("data-verification-url", expect.stringContaining(booking.qrToken));
    expect(screen.getByText("Eligible for cancellation")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel booking" })).toBeInTheDocument();
    expect(screen.queryByText(/check-in/i)).not.toBeInTheDocument();
    expect(document.body.textContent).not.toContain("passport");
    expect(document.body.textContent).not.toContain("example.com");
  });

  it("omits the empty Extras card while preserving the booking details in EN and TH", async () => {
    const currentBooking = { ...booking, extras: [] };
    mockCurrent.mockResolvedValue(currentBooking);
    const english = renderPage("en", true);

    expect(await screen.findByRole("heading", { name: "Your Booking" })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Extras" })).not.toBeInTheDocument();
    for (const heading of ["Passengers", "Seats", "Payment", "Cancellation", "E-Ticket"]) {
      expect(screen.getByRole("heading", { name: heading })).toBeInTheDocument();
    }

    english.unmount();
    mockCurrent.mockResolvedValue(currentBooking);
    renderPage("th", true);

    expect(await screen.findByRole("heading", { name: "การจองของคุณ" })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "บริการเสริม" })).not.toBeInTheDocument();
    for (const heading of ["ผู้โดยสาร", "ที่นั่ง", "การชำระเงิน", "การยกเลิก", "E-Ticket"]) {
      expect(screen.getByRole("heading", { name: heading })).toBeInTheDocument();
    }
  });

  it("renders the Manage Booking E-Ticket as a responsive horizontal travel document", async () => {
    mockCurrent.mockResolvedValue(booking);
    renderPage("en", true);

    expect(await screen.findByRole("heading", { name: "E-Ticket" })).toBeInTheDocument();
    expect(screen.getByText("Boarding-pass style travel document")).toBeInTheDocument();

    expect(screen.getByTestId("customer-e-ticket-layout")).toHaveClass(
      "grid",
      "grid-cols-1",
      "md:grid-cols-[minmax(0,1fr)_17rem]",
    );
    expect(screen.getByTestId("e-ticket-origin")).toHaveTextContent("BKK");
    expect(screen.getByTestId("e-ticket-destination")).toHaveTextContent("LHR");
    expect(screen.getByTestId("e-ticket-flight-number")).toHaveTextContent("XF 201");
    expect(screen.getByTestId("e-ticket-departure-date")).toHaveTextContent("04 Nov 2026");
    expect(screen.getByTestId("e-ticket-departure-time")).toHaveTextContent("09:15");
    expect(screen.getByTestId("e-ticket-cabin")).toHaveTextContent("Economy");
    expect(screen.getByTestId("e-ticket-passengers")).toHaveTextContent("Nara Van der Meer");
    expect(screen.getByTestId("e-ticket-passengers")).toHaveTextContent("Mali Van der Meer");
    expect(screen.getByTestId("e-ticket-seats")).toHaveTextContent("20A");
    expect(screen.getByTestId("e-ticket-seats")).toHaveTextContent("20B");
    expect(screen.getByTestId("e-ticket-booking-reference")).toHaveTextContent("XFABCDEFGH");
    expect(screen.getByTestId("e-ticket-ticket-number")).toHaveTextContent("XFTABCDEFGHIJKL");
    expect(screen.getByTestId("e-ticket-status")).toHaveTextContent("Ticket Issued");
    expect(screen.getByTestId("ticket-qr-container")).toHaveAttribute(
      "data-verification-url",
      expect.stringContaining(booking.qrToken),
    );
    expect(screen.getByTestId("e-ticket-print-action")).toHaveClass("print:hidden");
    const qrStub = screen.getByRole("complementary", { name: "Ticket Verification" });
    for (const label of ["Seats", "Booking Reference", "Flight"]) {
      expect(within(qrStub).getByText(label, { selector: "dt" })).toHaveClass(
        "text-white/70",
        "print:text-neutral-600",
      );
    }
    expect(within(qrStub).getByText("Scan this QR code to verify ticket authenticity.")).toHaveClass(
      "text-white/70",
      "print:text-neutral-700",
    );
    expect(within(qrStub).getByText("Cryptographically signed · Zero personal data encoded")).toHaveClass(
      "text-white/70",
      "print:text-neutral-700",
    );
    expect(screen.queryByText(/gate|terminal|boarding time|checked in|boarding authorized/i)).not.toBeInTheDocument();
  });

  it("prints the redesigned E-Ticket through the browser print flow", async () => {
    const printSpy = jest.spyOn(window, "print").mockImplementation(() => {});
    mockCurrent.mockResolvedValue(booking);
    renderPage("en", true);

    fireEvent.click(await screen.findByRole("button", { name: "Print E-ticket" }));

    expect(printSpy).toHaveBeenCalledTimes(1);
    printSpy.mockRestore();
  });

  it("keeps optional values safe and preserves an unfamiliar historical cabin value", async () => {
    mockCurrent.mockResolvedValue({
      ...booking,
      journey: { ...booking.journey, cabin: "premium-suite", departureTime: null },
      passengers: [{ ...booking.passengers[0], displayName: "A Passenger With A Deliberately Long Name For Layout Coverage" }],
      seats: [],
      ticket: { ...booking.ticket, ticketNumber: "XFTICKETNUMBERTHATISLONGERTHANEXPECTED" },
    });
    renderPage("en", true);

    expect(await screen.findByTestId("e-ticket-cabin")).toHaveTextContent("premium-suite");
    expect(screen.queryByTestId("e-ticket-departure-time")).not.toBeInTheDocument();
    expect(screen.getByTestId("e-ticket-ticket-number")).toHaveTextContent("XFTICKETNUMBERTHATISLONGERTHANEXPECTED");
    expect(screen.getByTestId("e-ticket-passengers")).toHaveTextContent(
      "A Passenger With A Deliberately Long Name For Layout Coverage",
    );
    expect(document.body.textContent).not.toContain("undefined");
    expect(document.body.textContent).not.toContain("null");
  });

  it("renders the E-Ticket labels in Thai without changing authoritative identifiers", async () => {
    mockCurrent.mockResolvedValue(booking);
    renderPage("th", true);

    expect(await screen.findByRole("heading", { name: "E-Ticket" })).toBeInTheDocument();
    expect(screen.getByText("เอกสารการเดินทางในรูปแบบบัตรโดยสาร")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "พิมพ์ตั๋วอิเล็กทรอนิกส์" })).toBeInTheDocument();
    expect(screen.getByTestId("e-ticket-booking-reference")).toHaveTextContent("XFABCDEFGH");
    expect(screen.getByText("XFTABCDEFGHIJKL")).toBeInTheDocument();
    expect(screen.getByTestId("e-ticket-cabin")).toHaveTextContent("ชั้นประหยัด");
  });

  it("confirms a full free refund without sending identity or money", async () => {
    mockCurrent.mockResolvedValue(booking);
    mockCancel.mockResolvedValue({ refundStatus: "PENDING" });
    renderPage("en", true);
    fireEvent.click(await screen.findByRole("button", { name: "Cancel booking" }));
    expect(screen.getByText("Cancellation is permanent. Your ticket will be cancelled and your seats released.")).toBeInTheDocument();
    expect(screen.getByText("THB 0")).toBeInTheDocument();
    expect(screen.getAllByText("THB 49,300").length).toBeGreaterThan(1);
    fireEvent.click(screen.getByRole("button", { name: "Confirm cancellation" }));
    await waitFor(() => expect(mockCancel).toHaveBeenCalledWith());
  });

  it("submits two credentials and navigates only after successful lookup", async () => {
    mockLookup.mockResolvedValue(booking);
    renderPage();
    await screen.findByRole("heading", { name: "Manage Booking" });

    fireEvent.change(screen.getByLabelText("Booking Reference"), {
      target: { value: " xFabcdefgh " },
    });
    fireEvent.change(screen.getByLabelText("Last Name"), {
      target: { value: " Van der Meer " },
    });
    fireEvent.click(screen.getByRole("button", { name: "Find Booking" }));

    await waitFor(() =>
      expect(mockLookup).toHaveBeenCalledWith({
        bookingReference: "XFABCDEFGH",
        lastName: "Van der Meer",
      }),
    );
    await waitFor(() => expect(mockReplace).toHaveBeenCalledWith("/manage-booking/details"));
    expect(screen.queryByText(booking.bookingReference)).not.toBeInTheDocument();
  });

  it("renders the lookup experience in Thai", async () => {
    renderPage("th");
    expect(await screen.findByRole("heading", { name: "จัดการการจอง" })).toBeInTheDocument();
    expect(screen.getByLabelText("นามสกุล")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "ค้นหาการจอง" })).toBeInTheDocument();
  });

  it("uses the established Thai date and currency presentation for booking details", async () => {
    mockCurrent.mockResolvedValue(booking);
    renderPage("th", true);

    expect(await screen.findByRole("heading", { name: "การจองของคุณ" })).toBeInTheDocument();
    expect(screen.getAllByText(/2026/).length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText("THB 49,300")).toBeInTheDocument();
    expect(screen.queryByText("2026-11-04")).not.toBeInTheDocument();
  });

  it("clearly represents a cancelled trip without active actions", async () => {
    mockCurrent.mockResolvedValue({
      ...booking,
      status: "CANCELLED",
      ticket: { ...booking.ticket, status: "CANCELLED" },
      cancellation: { eligibility: "UNAVAILABLE", cutoffAt: booking.cancellation.cutoffAt, refundStatus: "PROCESSING", refundAmount: booking.payment.amount },
    } satisfies ManageBookingDetails);
    renderPage("en", true);

    expect((await screen.findAllByText("Booking cancelled")).length).toBeGreaterThan(0);
    expect(screen.getByText("Cancelled")).toBeInTheDocument();
    expect(screen.getByText("Refund processing")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Cancel booking" })).not.toBeInTheDocument();
    expect(screen.queryByText(/check-in/i)).not.toBeInTheDocument();
    expect(
      screen.getByText("Seat records are shown for reference. This booking is not active."),
    ).toBeInTheDocument();
  });
  it("polls only a nonterminal refund and stops after success", async () => {
    jest.useFakeTimers();
    const processing = { ...booking, status: "CANCELLED" as const, ticket: { ...booking.ticket, status: "CANCELLED" as const }, cancellation: { eligibility: "UNAVAILABLE" as const, cutoffAt: booking.cancellation.cutoffAt, refundStatus: "PROCESSING" as const, refundAmount: booking.payment.amount } };
    const succeeded = { ...processing, cancellation: { ...processing.cancellation, refundStatus: "SUCCEEDED" as const } };
    mockCurrent.mockResolvedValueOnce(processing).mockResolvedValueOnce(succeeded);
    renderPage("en", true);
    await act(async () => { await Promise.resolve(); });
    expect(screen.getByText("Refund processing")).toBeInTheDocument();
    await act(async () => { jest.advanceTimersByTime(5000); await Promise.resolve(); });
    expect(screen.getByText("Refund completed")).toBeInTheDocument();
    await act(async () => { jest.advanceTimersByTime(10000); await Promise.resolve(); });
    expect(mockCurrent).toHaveBeenCalledTimes(2);
    jest.useRealTimers();
  });
  it("blocks direct details without authorization and reveals no booking", async () => {
    renderPage("en", true);
    expect(await screen.findByRole("alert")).toHaveTextContent("Your Manage Booking access expired.");
    expect(screen.queryByText(booking.bookingReference)).not.toBeInTheDocument();
    expect(screen.queryByTestId("ticket-qr-container")).not.toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Find Booking" })).toHaveAttribute("href", "/manage-booking");
  });

  it("loads only the newly authorized booking on details", async () => {
    const next = { ...booking, bookingReference: "XF23456789", seats: ["21A"], journey: { ...booking.journey, departureDate: "2026-12-05" } };
    mockCurrent.mockResolvedValue(next);
    renderPage("en", true);
    expect(await screen.findByTestId("e-ticket-booking-reference")).toHaveTextContent(next.bookingReference);
    expect(screen.queryByText(booking.bookingReference)).not.toBeInTheDocument();
    expect(screen.queryByText("20A")).not.toBeInTheDocument();
    expect(screen.getByTestId("e-ticket-seats")).toHaveTextContent("21A");
  });
});
