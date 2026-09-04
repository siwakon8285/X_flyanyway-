import { render, screen } from "@testing-library/react";

import { verifyTicket } from "@/components/booking/ticket/ticketClient";
import { TicketVerifyPage } from "@/components/booking/ticket/TicketVerifyPage";
import {
  buildTicketVerificationUrl,
  resolveFrontendOrigin,
} from "@/components/booking/ticket/ticketVerificationUrl";
import { LanguageProvider } from "@/i18n/LanguageProvider";

jest.mock("@/components/booking/ticket/ticketClient", () => ({
  verifyTicket: jest.fn(),
}));

const mockVerifyTicket = verifyTicket as jest.Mock;

const renderTicketVerifyPage = (token: string) =>
  render(
    <LanguageProvider initialLocale="en">
      <TicketVerifyPage token={token} />
    </LanguageProvider>,
  );

describe("Ticket Verification URL & Page", () => {
  const originalEnv = process.env;

  beforeEach(() => {
    jest.clearAllMocks();
    process.env = { ...originalEnv };
    delete process.env.NEXT_PUBLIC_SITE_URL;
    delete process.env.NEXT_PUBLIC_APP_URL;
  });

  afterAll(() => {
    process.env = originalEnv;
  });

  describe("buildTicketVerificationUrl", () => {
    it("generates a valid verification URL containing the signed token", () => {
      const token = "v1.c76f62b0-95ad-4d4a-9b1e-624e75124119.a3f901bc45d6";
      const url = buildTicketVerificationUrl(token);

      expect(url).toMatch(/^https?:\/\/.+\/ticket\/verify\/.+/);
      expect(url).toContain(token);
    });

    it("respects NEXT_PUBLIC_SITE_URL if configured", () => {
      process.env.NEXT_PUBLIC_SITE_URL = "https://flyanyway.com";
      const token = "v1.test-uuid.test-signature";
      const url = buildTicketVerificationUrl(token);

      expect(url).toBe("https://flyanyway.com/ticket/verify/v1.test-uuid.test-signature");
    });

    it("contains strictly zero passenger or payment PII", () => {
      const token = "v1.12345678-1234-1234-1234-123456789abc.abcdef0123456789";
      const url = buildTicketVerificationUrl(token);

      // Verify absence of sensitive keywords
      expect(url).not.toContain("name");
      expect(url).not.toContain("passenger");
      expect(url).not.toContain("passport");
      expect(url).not.toContain("payment");
      expect(url).not.toContain("card");
      expect(url).not.toContain("amount");
    });

    it("resolves origin safely across environments", () => {
      expect(resolveFrontendOrigin()).toBeTruthy();
    });
  });

  describe("TicketVerifyPage Component", () => {
    const validToken = "v1.c76f62b0-95ad-4d4a-9b1e-624e75124119.signature";

    it("renders loading state initially while verifying token", () => {
      mockVerifyTicket.mockReturnValue(new Promise(() => {}));
      renderTicketVerifyPage(validToken);

      expect(screen.getByRole("status", { name: "Verifying ticket authenticity..." })).toBeInTheDocument();
    });

    it("renders verified flight information when token is valid", async () => {
      mockVerifyTicket.mockResolvedValue({
        departureDate: "2026-10-15",
        departureTime: "09:15",
        destinationCode: "HND",
        flightNumber: "XF 201",
        originCode: "BKK",
        seats: ["20A", "20B"],
        ticketStatus: "ISSUED",
        valid: true,
      });

      renderTicketVerifyPage(validToken);

      expect(await screen.findByText("Ticket Verified")).toBeInTheDocument();
      expect(screen.getByTestId("verify-flight-number")).toHaveTextContent("XF 201");
      expect(screen.getByTestId("verify-route")).toHaveTextContent("BKK → HND");
      expect(screen.getByTestId("verify-date")).toHaveTextContent("2026-10-15 · 09:15");
      expect(screen.getByTestId("verify-status")).toHaveTextContent("ISSUED");
      expect(screen.getByTestId("verify-seats")).toHaveTextContent("20A");
      expect(screen.getByTestId("verify-seats")).toHaveTextContent("20B");

      // Verify zero PII is displayed
      expect(screen.queryByText("Nara")).not.toBeInTheDocument();
      expect(screen.queryByText("Suri")).not.toBeInTheDocument();
      expect(screen.queryByText("THB")).not.toBeInTheDocument();

      // Security statement is present
      expect(screen.getByText(/Cryptographically verified via HMAC-SHA256 signature/)).toBeInTheDocument();
    });

    it("renders verification failed state when token is invalid or tampered", async () => {
      const tamperedToken = "v1.c76f62b0-95ad-4d4a-9b1e-624e75124119.tampered_signature";
      mockVerifyTicket.mockResolvedValue({
        valid: false,
      });

      renderTicketVerifyPage(tamperedToken);

      expect(await screen.findByText("Verification Failed")).toBeInTheDocument();
      expect(
        screen.getByText("This QR code or verification token is invalid, expired, or tampered."),
      ).toBeInTheDocument();
      expect(screen.queryByTestId("verify-flight-number")).not.toBeInTheDocument();
    });

    it("renders verification failed state on network error", async () => {
      mockVerifyTicket.mockRejectedValue(new Error("Network connection lost"));

      renderTicketVerifyPage(validToken);

      expect(await screen.findByText("Verification Failed")).toBeInTheDocument();
      expect(screen.getByRole("link", { name: "Return to Home" })).toBeInTheDocument();
    });
  });
});
