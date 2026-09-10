import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { FlightEditor } from "@/components/admin/flights/FlightEditor";
import { render } from "@/tests/renderWithLanguage";

const flight = { id:"11111111-1111-4111-8111-111111111111",publicId:"xf-951-20261008",flightNumber:"XF 951",originCode:"BKK",destinationCode:"DXB",originTimeZone:"Asia/Bangkok",destinationTimeZone:"Asia/Dubai",operatingDate:"2026-10-08",departureTime:"09:20:00",arrivalTime:"13:05:00",arrivalDayOffset:0,aircraftCode:"Boeing 787-9",status:"SCHEDULED",business:{available:true,priceAmount:46900,currencyCode:"THB",capacity:16},first:{available:true,priceAmount:78900,currencyCode:"THB",capacity:4},version:1,updatedAt:"2026-09-07T00:00:00Z",audit:[] };
const references = { airports:[{code:"BKK",name:"Suvarnabhumi Airport",city:"Bangkok",countryCode:"TH",countryName:"Thailand",timeZone:"Asia/Bangkok"},{code:"DXB",name:"Dubai International Airport",city:"Dubai",countryCode:"AE",countryName:"United Arab Emirates",timeZone:"Asia/Dubai"}],aircraft:["Boeing 787-9"] };
const createdAudit = { id:"audit-created",actorEmail:"creator@x-fly.test",action:"FLIGHT_CREATED" as const,beforeState:null,afterState:{},createdAt:"2026-09-07T00:00:00Z" };
const cancelledAudit = { id:"audit-cancelled",actorEmail:"manager@x-fly.test",action:"FLIGHT_CANCELLED" as const,beforeState:{},afterState:{},createdAt:"2026-09-10T02:17:00Z" };

describe("Flight editor", () => {
  beforeEach(() => { global.fetch = jest.fn(async (url: string | URL | Request) => ({ ok:true,status:200,json:async()=>String(url).includes("reference-data")?references:flight })) as jest.Mock; });

  it("groups authoritative create fields and exposes only Business and First", async () => {
    render(<FlightEditor canWrite mode="new" />);
    expect(await screen.findByRole("heading", { name:"Create flight" })).toBeInTheDocument();
    for (const section of ["Identity","Schedule","Aircraft / Inventory","Commercial"]) expect(screen.getByRole("group", { name:section })).toBeInTheDocument();
    expect(screen.getByLabelText("Business price (THB)")).toBeInTheDocument();
    expect(screen.getByLabelText("First capacity")).toBeInTheDocument();
    expect(screen.getByLabelText("Operational flight summary")).toHaveTextContent("XF / New service");
    expect(screen.getByLabelText("Operational flight summary")).toHaveTextContent("Route pending");
    expect(screen.queryByText("Economy")).not.toBeInTheDocument();
    fireEvent.submit(screen.getByRole("form", { name:"Create flight" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Review the highlighted flight fields.");
  });

  it("uses one explicit field-control contract without applying it to the checkbox", async () => {
    render(<FlightEditor canWrite mode="new" />);
    expect(await screen.findByRole("heading", { name:"Create flight" })).toBeInTheDocument();

    for (const label of [
      "Flight number", "Origin", "Destination", "Departure date", "Departure", "Arrival",
      "Aircraft context", "Business capacity", "First capacity", "Business price (THB)",
      "First price (THB)",
    ]) {
      expect(screen.getByLabelText(label)).toHaveClass("xfo-control");
    }

    const checkbox = screen.getByRole("checkbox", { name:"Arrives next day" });
    const departure = screen.getByLabelText("Departure");
    const arrival = screen.getByLabelText("Arrival");
    expect(departure).toHaveAttribute("type", "time");
    expect(arrival).toHaveAttribute("type", "time");
    expect(departure).toBeRequired();
    expect(arrival).toBeRequired();
    expect(checkbox).toHaveClass("xfo-checkbox-input");
    expect(checkbox).not.toHaveClass("xfo-control");
    expect(checkbox).toHaveAttribute("type", "checkbox");
    checkbox.focus();
    expect(checkbox).toHaveFocus();
    expect(fireEvent.keyDown(checkbox, { key:" ", code:"Space" })).toBe(true);
    fireEvent.change(departure, { target:{ value:"09:20" } });
    fireEvent.click(screen.getByText("Arrives next day"));
    expect(checkbox).toBeChecked();
    expect(departure).toHaveValue("09:20");
  });

  it("keeps critical field labels and the compact checkbox localized in Thai", async () => {
    render(<FlightEditor canWrite mode="new" />, { locale:"th" });
    expect(await screen.findByRole("heading", { name:"สร้างเที่ยวบิน" })).toBeInTheDocument();
    expect(screen.getByLabelText("วันออกเดินทาง")).toHaveClass("xfo-date-input");
    expect(screen.getByRole("combobox", { name:"ต้นทาง" })).toHaveClass("xfo-airport-input");
    expect(screen.getByRole("checkbox", { name:"ถึงวันถัดไป" })).toHaveClass("xfo-checkbox-input");
  });

  it("uses an accessible consequence-specific cancellation dialog and authoritative response", async () => {
    render(<FlightEditor canWrite flightId={flight.id} mode="detail" />);
    expect(await screen.findByRole("heading", { name:"XF 951" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name:"Cancel flight" }));
    const dialog = screen.getByRole("dialog", { name:"Cancel XF 951?" });
    expect(dialog).toHaveTextContent("BKK → DXB");
    expect(dialog).toHaveTextContent("unavailable for all new searches, holds and payments");
    expect(within(dialog).queryByRole("button", { name:"Close dialog" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name:"Keep scheduled" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(fetch).not.toHaveBeenCalledWith(expect.stringContaining("/cancel"), expect.anything());

    const trigger = screen.getByRole("button", { name:"Cancel flight" });
    trigger.focus();
    fireEvent.click(trigger);
    await waitFor(() => expect(screen.getByRole("button", { name:"Keep scheduled" })).toHaveFocus());
    fireEvent.keyDown(document, { key:"Escape" });
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    await waitFor(() => expect(trigger).toHaveFocus());

    fireEvent.click(trigger);
    const cancelled = { ...flight,status:"CANCELLED" as const,version:2 };
    jest.mocked(fetch)
      .mockResolvedValueOnce({ ok:true,status:200,json:async()=>cancelled } as Response)
      .mockResolvedValueOnce({ ok:true,status:200,json:async()=>({ ...cancelled,audit:[cancelledAudit] }) } as Response);
    fireEvent.click(screen.getByRole("button", { name:"Confirm flight cancellation" }));
    await waitFor(() => expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/cancel"), expect.objectContaining({ method:"POST", credentials:"same-origin", headers:expect.objectContaining({ "X-X-Fly-CSRF":"1" }) })));
  });

  it("renders the English cancellation date in Gregorian presentation without changing the machine date", async () => {
    render(<FlightEditor canWrite flightId={flight.id} mode="detail" />);
    expect(await screen.findByRole("heading", { name:"XF 951" })).toBeInTheDocument();
    expect(screen.getByLabelText("Departure date")).toHaveValue("2026-10-08");

    fireEvent.click(screen.getByRole("button", { name:"Cancel flight" }));
    const dialog = screen.getByRole("dialog");
    expect(dialog).toHaveTextContent("08 Oct 2026");
    expect(dialog).not.toHaveTextContent("2026-10-08");
    expect(within(dialog).queryByRole("button", { name:"Close dialog" })).not.toBeInTheDocument();
    expect(screen.getByLabelText("Departure date")).toHaveValue("2026-10-08");
  });

  it("renders the Thai cancellation date in the Buddhist Era without changing the machine date", async () => {
    render(<FlightEditor canWrite flightId={flight.id} mode="detail" />, { locale:"th" });
    expect(await screen.findByRole("heading", { name:"XF 951" })).toBeInTheDocument();
    expect(screen.getByLabelText("วันออกเดินทาง")).toHaveValue("2026-10-08");

    fireEvent.click(screen.getByRole("button", { name:"ยกเลิกเที่ยวบิน" }));
    const dialog = screen.getByRole("dialog");
    expect(dialog).toHaveTextContent("08 ต.ค. 2569");
    expect(dialog).not.toHaveTextContent("2026");
    expect(within(dialog).queryByRole("button", { name:"ปิดกล่องโต้ตอบ" })).not.toBeInTheDocument();
    expect(screen.getByLabelText("วันออกเดินทาง")).toHaveValue("2026-10-08");
  });

  it("applies cancellation immediately and replaces history from one authoritative detail refresh", async () => {
    const cancelled = { ...flight, status:"CANCELLED" as const, version:2, updatedAt:"2026-09-10T02:17:00Z" };
    const refreshed = { ...cancelled, audit:[cancelledAudit, createdAudit] };
    let detailReads = 0;
    let resolveRefresh: (response: Response) => void = () => undefined;
    const refresh = new Promise<Response>((resolve) => { resolveRefresh = resolve; });
    jest.mocked(fetch).mockImplementation((url, init) => {
      const path = String(url);
      if (path.includes("reference-data")) return Promise.resolve({ ok:true,status:200,json:async()=>references } as Response);
      if (init?.method === "POST") return Promise.resolve({ ok:true,status:200,json:async()=>cancelled } as Response);
      detailReads += 1;
      return detailReads === 1
        ? Promise.resolve({ ok:true,status:200,json:async()=>flight } as Response)
        : refresh;
    });

    render(<FlightEditor canWrite flightId={flight.id} mode="detail" />);
    expect(await screen.findByRole("heading", { name:"XF 951" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name:"Cancel flight" }));
    fireEvent.click(screen.getByRole("button", { name:"Confirm flight cancellation" }));

    expect(await screen.findByText("Cancelled")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name:"Cancel flight" })).not.toBeInTheDocument();
    expect(screen.getByLabelText("Departure date")).toBeDisabled();
    expect(screen.getByText("No mutation history is available.")).toBeInTheDocument();
    await waitFor(() => expect(detailReads).toBe(2));

    await act(async () => {
      resolveRefresh({ ok:true,status:200,json:async()=>refreshed } as Response);
    });
    expect(await screen.findByText("Flight cancelled by manager@x-fly.test")).toBeInTheDocument();
    expect(screen.getByText("Flight created by creator@x-fly.test")).toBeInTheDocument();

    const calls = jest.mocked(fetch).mock.calls;
    expect(calls.filter(([, init]) => init?.method === "POST")).toHaveLength(1);
    const detailCalls = calls.filter(([url, init]) => String(url) === `/admin/api/flights/${flight.id}` && !init?.method);
    expect(detailCalls).toHaveLength(2);
    expect(detailCalls[1]?.[1]).toEqual(expect.objectContaining({ cache:"no-store", credentials:"same-origin" }));
  });

  it("retains the cancelled state and previous history when detail synchronization fails", async () => {
    const initial = { ...flight, audit:[createdAudit] };
    const cancelled = { ...flight, status:"CANCELLED" as const, version:2, updatedAt:"2026-09-10T02:17:00Z" };
    let detailReads = 0;
    jest.mocked(fetch).mockImplementation(async (url, init) => {
      const path = String(url);
      if (path.includes("reference-data")) return { ok:true,status:200,json:async()=>references } as Response;
      if (init?.method === "POST") return { ok:true,status:200,json:async()=>cancelled } as Response;
      detailReads += 1;
      return detailReads === 1
        ? { ok:true,status:200,json:async()=>initial } as Response
        : { ok:false,status:503,json:async()=>({}) } as Response;
    });

    render(<FlightEditor canWrite flightId={flight.id} mode="detail" />, { locale:"th" });
    expect(await screen.findByRole("heading", { name:"XF 951" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name:"ยกเลิกเที่ยวบิน" }));
    fireEvent.click(screen.getByRole("button", { name:"ยืนยันยกเลิกเที่ยวบิน" }));

    expect(await screen.findByText("ยกเลิกแล้ว")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name:"ยกเลิกเที่ยวบิน" })).not.toBeInTheDocument();
    expect(screen.getByText("สร้างเที่ยวบิน โดย creator@x-fly.test")).toBeInTheDocument();
    expect(await screen.findByRole("alert")).toHaveTextContent("ยกเลิกเที่ยวบินแล้ว แต่ไม่สามารถโหลดประวัติล่าสุดได้ กรุณารีเฟรชรายละเอียดในภายหลัง");

    const calls = jest.mocked(fetch).mock.calls;
    expect(calls.filter(([, init]) => init?.method === "POST")).toHaveLength(1);
    expect(calls.filter(([url, init]) => String(url) === `/admin/api/flights/${flight.id}` && !init?.method)).toHaveLength(2);
  });

  it("does not render edit or cancellation actions for read-only staff", async () => {
    render(<FlightEditor canWrite={false} flightId={flight.id} mode="detail" />);
    expect(await screen.findByRole("heading", { name:"XF 951" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name:"Cancel flight" })).not.toBeInTheDocument();
    expect(screen.getByText("Read-only access")).toBeInTheDocument();
  });

  it("keeps origin and destination validation after searchable selection", async () => {
    render(<FlightEditor canWrite flightId={flight.id} mode="detail" />);
    expect(await screen.findByRole("heading", { name:"XF 951" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("combobox", { name:"Destination" }));
    fireEvent.change(screen.getByRole("combobox", { name:"Destination" }), {
      target: { value:"BKK" },
    });
    fireEvent.click(screen.getByRole("option", { name:/BKK.*Bangkok.*Thailand/ }));
    fireEvent.submit(screen.getByRole("form", { name:"XF 951" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Review the highlighted flight fields.");
    expect(fetch).not.toHaveBeenCalledWith(
      expect.stringContaining(`/admin/api/flights/${flight.id}`),
      expect.objectContaining({ method:"PUT" }),
    );
  });

  it("localizes audit actions and timestamps in Thai", async () => {
    const audited = { ...flight, audit:[{ id:"audit-1",actorEmail:"manager@x-fly.test",action:"FLIGHT_CREATED" as const,beforeState:null,afterState:{},createdAt:"2026-09-07T00:00:00Z" }] };
    jest.mocked(fetch).mockImplementation(async (url) => ({ ok:true,status:200,json:async()=>String(url).includes("reference-data")?references:audited }) as Response);
    render(<FlightEditor canWrite={false} flightId={flight.id} mode="detail" />, { locale:"th" });
    expect(await screen.findByText(/สร้างเที่ยวบิน โดย manager@x-fly\.test/)).toBeInTheDocument();
    expect(document.querySelector('time')).toHaveTextContent("2569");
    expect(screen.queryByText(/FLIGHT_CREATED/)).not.toBeInTheDocument();
  });
});
