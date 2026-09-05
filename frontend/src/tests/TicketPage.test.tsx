import { readFileSync } from "node:fs";
import path from "node:path";
import { act, fireEvent, render, screen } from "@testing-library/react";

import { BookingApiError } from "@/components/booking/ticket/ticketClient";
import type { TicketResponse } from "@/components/booking/ticket/ticketTypes";
import { TicketPage } from "@/components/booking/ticket/TicketPage";
import { LanguageProvider } from "@/i18n/LanguageProvider";

const mockGetTicket = jest.fn();

jest.mock("@/components/booking/ticket/ticketClient", () => {
  const actual = jest.requireActual(
    "@/components/booking/ticket/ticketVerificationUrl",
  );
  return {
    ...actual,
    BookingApiError: class BookingApiError extends Error {
      code: string;
      status: number;
      constructor({
        code,
        message,
        status,
      }: {
        code: string;
        message: string;
        status: number;
      }) {
        super(message);
        this.code = code;
        this.status = status;
      }
    },
    getTicket: (holdId: string, attemptId: string) =>
      mockGetTicket(holdId, attemptId),
  };
});

const mockTicketResponse: TicketResponse = {
  qrToken: "v1.12345678-1234-1234-1234-123456789abc.deadbeefcafebabe12345678",
  ticket: {
    amount: 49300,
    bookingReference: "XF7K9P",
    currencyCode: "THB",
    id: "12345678-1234-1234-1234-123456789abc",
    issuedAt: "2026-09-03T10:00:00Z",
    journey: {
      arrivalDayOffset: 0,
      arrivalTime: "15:45",
      cabin: "economy",
      departureDate: "2026-10-15",
      departureTime: "09:15",
      destinationCode: "HND",
      flightNumber: "XF 201",
      originCode: "BKK",
    },
    passengers: [
      { displayName: "MS. NARA SURI" },
      { displayName: "MR. ARUN SURI" },
    ],
    paymentStatus: "SUCCEEDED",
    seats: ["20A", "20B"],
    status: "ISSUED",
    ticketNumber: "026-1234567890",
  },
};

const renderTicketPage = (props = { attemptId: "attempt-1", holdId: "hold-1" }) =>
  render(
    <LanguageProvider initialLocale="en">
      <TicketPage {...props} />
    </LanguageProvider>,
  );

describe("TicketPage", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    Object.assign(navigator, {
      clipboard: {
        writeText: jest.fn().mockResolvedValue(undefined),
      },
    });
  });

  it("renders loading state initially while fetching ticket", () => {
    mockGetTicket.mockReturnValue(new Promise(() => {}));
    renderTicketPage();
    expect(screen.getByRole("status", { name: "Loading flight ticket" })).toBeInTheDocument();
  });

  it("renders authoritative ticket details on successful load", async () => {
    mockGetTicket.mockResolvedValue(mockTicketResponse);
    renderTicketPage();

    expect(await screen.findByRole("heading", { name: "Booking Confirmed" })).toBeInTheDocument();
    expect(screen.getByTestId("ticket-flight-number")).toHaveTextContent("XF 201");
    expect(screen.getByTestId("ticket-origin")).toHaveTextContent("BKK");
    expect(screen.getByTestId("ticket-destination")).toHaveTextContent("HND");
    expect(screen.getAllByText("15 Oct 2026")[0]).toBeInTheDocument();
    expect(screen.getByText("09:15")).toBeInTheDocument();
    expect(screen.getByText("15:45")).toBeInTheDocument();

    // Passenger & Seats
    const passengers = screen.getByTestId("ticket-passengers");
    expect(passengers).toHaveTextContent("MS. NARA SURI");
    expect(passengers).toHaveTextContent("MR. ARUN SURI");

    const seats = screen.getByTestId("ticket-seats");
    expect(seats).toHaveTextContent("20A");
    expect(seats).toHaveTextContent("20B");

    // Reference numbers
    expect(screen.getByTestId("ticket-booking-reference")).toHaveTextContent("XF7K9P");
    expect(screen.getByTestId("ticket-number")).toHaveTextContent("026-1234567890");

    // Financial
    expect(screen.getByTestId("ticket-amount")).toHaveTextContent("THB 49,300");
    expect(screen.getByText("Payment Succeeded")).toBeInTheDocument();

    // Status badge
    expect(screen.getByText("Ticket Issued")).toBeInTheDocument();

    expect(screen.queryByTestId("ticket-qr-container")).not.toBeInTheDocument();
    expect(screen.queryByText("Ticket Verification")).not.toBeInTheDocument();
    expect(document.body.innerHTML).not.toContain(mockTicketResponse.qrToken);
    expect(screen.getByText("Your booking is confirmed and payment has been completed. Keep your booking reference for future access.")).toBeInTheDocument();

    // Action buttons
    expect(screen.getByRole("button", { name: "Print Booking Summary" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Back to Home" })).toHaveAttribute("href", "/");
    expect(screen.queryByRole("link", { name: "Return Home" })).not.toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "Manage Booking" })).not.toBeInTheDocument();
    expect(document.querySelector("[data-ticket-page]")).toHaveClass("min-h-screen", "print:p-0");
  });

  it("copies booking reference and ticket number to clipboard", async () => {
    mockGetTicket.mockResolvedValue(mockTicketResponse);
    renderTicketPage();

    await screen.findByTestId("ticket-booking-reference");
    const copyButtons = screen.getAllByRole("button", { name: "Copy" });
    await act(async () => {
      fireEvent.click(copyButtons[0]);
    });

    expect(navigator.clipboard.writeText).toHaveBeenCalledWith("XF7K9P");
  });

  it("displays payment incomplete error if payment has not succeeded", async () => {
    mockGetTicket.mockRejectedValue(
      new BookingApiError({
        code: "TICKET_PAYMENT_INCOMPLETE",
        message: "Payment must succeed before ticket can be issued",
        status: 409,
      }),
    );
    renderTicketPage();

    expect(
      await screen.findByText("Ticket is available only after payment succeeds."),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Back to Payment" })).toHaveAttribute(
      "href",
      "/booking/payment?holdId=hold-1",
    );
  });

  it("displays unauthorized error when session is unauthorized", async () => {
    mockGetTicket.mockRejectedValue(
      new BookingApiError({
        code: "UNAUTHORIZED",
        message: "Session unauthorized",
        status: 401,
      }),
    );
    renderTicketPage();

    expect(
      await screen.findByText("You are not authorized to view this ticket."),
    ).toBeInTheDocument();
  });

  it("displays not found error when ticket parameters are missing", async () => {
    renderTicketPage({ attemptId: "", holdId: "" });

    expect(
      await screen.findByText("Ticket not found for this booking."),
    ).toBeInTheDocument();
  });

  it("calls window.print when print button is clicked", async () => {
    const printSpy = jest.spyOn(window, "print").mockImplementation(() => {});
    mockGetTicket.mockResolvedValue(mockTicketResponse);
    renderTicketPage();

    const printButton = await screen.findByRole("button", { name: "Print Booking Summary" });
    fireEvent.click(printButton);

    expect(printSpy).toHaveBeenCalled();
    printSpy.mockRestore();
  });
  it("renders a localized printable Thai summary with a responsive route", async () => {
    mockGetTicket.mockResolvedValue({ ...mockTicketResponse, ticket: { ...mockTicketResponse.ticket, journey: { ...mockTicketResponse.ticket.journey, arrivalDayOffset: 1 } } });
    render(<LanguageProvider initialLocale="th"><TicketPage attemptId="attempt-1" holdId="hold-1" /></LanguageProvider>);
    const heading = await screen.findByRole("heading", { name: "ยืนยันการจองแล้ว" });
    expect(heading.parentElement?.parentElement?.parentElement).not.toHaveClass("print:hidden");
    expect(screen.getByRole("button", { name: "พิมพ์สรุปการจอง" })).toBeInTheDocument();
    for (const link of screen.getAllByRole("link", { name: "กลับสู่หน้าหลัก" })) expect(link).toHaveAttribute("href", "/");
    expect(screen.queryByTestId("ticket-qr-container")).not.toBeInTheDocument();
    expect(screen.getByTestId("ticket-origin").parentElement?.parentElement).toHaveClass("flex-col", "sm:flex-row");
    expect(screen.getByText(/16.*2026/)).toBeInTheDocument();
  });

  it("removes the screen-height print constraint from the booking summary page", () => {
    const printCss = readFileSync(path.join(process.cwd(), "src/app/globals.css"), "utf8");

    expect(printCss).toMatch(/@media print[\s\S]*\[data-ticket-page\][\s\S]*min-height:\s*auto/);
    expect(printCss).toMatch(/\[data-ticket-page\][\s\S]*break-after:\s*auto/);
  });
});
