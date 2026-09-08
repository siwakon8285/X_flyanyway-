import { fireEvent, screen, waitFor } from "@testing-library/react";
import { FlightsWorkspace } from "@/components/admin/flights/FlightsWorkspace";
import { render } from "@/tests/renderWithLanguage";

const flight = {
  id: "11111111-1111-4111-8111-111111111111", publicId: "xf-951-20261008", flightNumber: "XF 951",
  originCode: "BKK", destinationCode: "DXB", originTimeZone: "Asia/Bangkok", destinationTimeZone: "Asia/Dubai",
  operatingDate: "2026-10-08", departureTime: "09:20:00", arrivalTime: "13:05:00", arrivalDayOffset: 0,
  aircraftCode: "Boeing 787-9", status: "SCHEDULED", business: { available: true, priceAmount: 46900, currencyCode: "THB", capacity: 16 },
  first: { available: true, priceAmount: 78900, currencyCode: "THB", capacity: 4 }, version: 1, updatedAt: "2026-09-07T00:00:00Z",
};

describe("Flight Management", () => {
  beforeEach(() => { global.fetch = jest.fn().mockResolvedValue({ ok: true, json: async () => ({ items: [flight], total: 1, limit: 50, offset: 0 }) }); });

  it("renders an accessible operational list and filters without fake metrics", async () => {
    render(<FlightsWorkspace canWrite />);
    expect(await screen.findByRole("heading", { name: "Flight Management" })).toBeInTheDocument();
    expect(await screen.findByRole("region", { name: "Flight service registry" })).toHaveClass("xfo-board");
    expect(screen.getByRole("table", { name: "Flight service registry" })).toBeInTheDocument();
    expect(screen.getByRole("rowheader", { name: "XF 951" })).toBeInTheDocument();
    expect(screen.getByRole("cell", { name:/BKK.*DXB/ })).toBeInTheDocument();
    expect(screen.getAllByText("Scheduled").length).toBeGreaterThan(0);
    expect(screen.getByRole("link", { name: "Create flight" })).toHaveAttribute("href", "/admin/flights/new");
    fireEvent.change(screen.getByLabelText("Flight number"), { target: { value: "951" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply filters" }));
    await waitFor(() => expect(fetch).toHaveBeenLastCalledWith(expect.stringContaining("search=951"), expect.anything()));
  });

  it("keeps read-only staff operationally useful without mutation controls", async () => {
    render(<FlightsWorkspace canWrite={false} />);
    expect(await screen.findByRole("rowheader", { name: "XF 951" })).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "Create flight" })).not.toBeInTheDocument();
    expect(screen.getByText("Read-only access")).toBeInTheDocument();
  });

  it("shows localized loading and content", async () => {
    render(<FlightsWorkspace canWrite={false} />, { locale: "th" });
    expect(screen.getByText("กำลังโหลดเที่ยวบิน")).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "การจัดการเที่ยวบิน" })).toBeInTheDocument();
  });

  it("retries the same valid list query after a backend error", async () => {
    jest.mocked(fetch)
      .mockRejectedValueOnce(new Error("offline"))
      .mockResolvedValueOnce({ ok: true, json: async () => ({ items: [flight], total: 1, limit: 50, offset: 0 }) } as Response);
    render(<FlightsWorkspace canWrite={false} />);
    expect(await screen.findByRole("alert")).toHaveTextContent("temporarily unavailable");
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByRole("rowheader", { name: "XF 951" })).toBeInTheDocument();
    expect(fetch).toHaveBeenLastCalledWith("/admin/api/flights", expect.anything());
  });
});
