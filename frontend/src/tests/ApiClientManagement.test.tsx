import { fireEvent, screen, waitFor, within } from "@testing-library/react";

import { ApiClientEditor } from "@/components/admin/api-clients/ApiClientEditor";
import { ApiClientsWorkspace } from "@/components/admin/api-clients/ApiClientsWorkspace";
import { useLanguage } from "@/i18n/LanguageProvider";
import type { ApiClientDetail, ApiClientRecord, ApiScopeDefinition } from "@/lib/admin/apiClientTypes";
import { render } from "@/tests/renderWithLanguage";

const mockReplace = jest.fn();

jest.mock("next/navigation", () => ({
  useRouter: () => ({ replace: mockReplace }),
}));

const scopes: ApiScopeDefinition[] = [
  { code:"analytics:read", description:"Future read access to approved aggregate analytics" },
  { code:"flights:read", description:"Future read access to approved flight data" },
];
const client: ApiClientRecord = {
  clientId:"XFCABCDEFGHJKLMNPQR", name:"Marketing Insights", description:"Approved aggregates",
  status:"SUSPENDED", allowedScopes:["analytics:read"], version:1,
  createdAt:"2026-09-09T02:00:00Z", updatedAt:"2026-09-09T02:00:00Z",
  createdBy:"api@x.test", updatedBy:"api@x.test",
};
const flightsClient: ApiClientRecord = { ...client, clientId:"XFCFLIGHTDATA234567", name:"Flight Data", status:"ACTIVE", allowedScopes:["flights:read"] };
const bothClient: ApiClientRecord = { ...client, clientId:"XFCBOTHSCOPES23456", name:"Both Scopes", status:"ACTIVE", allowedScopes:["analytics:read", "flights:read"] };
const detail: ApiClientDetail = {
  ...client,
  audit:[{ actorEmail:"api@x.test", action:"CLIENT_CREATED", before:null,
    after:{ name:client.name, description:client.description, status:client.status, allowedScopes:client.allowedScopes },
    createdAt:client.createdAt }],
};

const auditEntry = (
  action: ApiClientDetail["audit"][number]["action"],
  record: ApiClientRecord,
  actorEmail = "operator@x.test",
): ApiClientDetail["audit"][number] => ({
  actorEmail,
  action,
  before: {
    name:client.name,
    description:client.description,
    status:client.status,
    allowedScopes:client.allowedScopes,
  },
  after: {
    name:record.name,
    description:record.description,
    status:record.status,
    allowedScopes:record.allowedScopes,
  },
  createdAt:record.updatedAt,
});

const response = (value: unknown, ok = true, status = 200) => ({ ok, status, json:async () => value } as Response);

const LocaleSwitchingApiClients = () => {
  const { toggleLocale } = useLanguage();
  return <><button onClick={toggleLocale} type="button">Toggle locale</button><ApiClientsWorkspace canManage={false} /></>;
};

describe("API Client Management", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    window.history.replaceState({}, "", "/admin/api-clients/new");
  });

  it("renders the bounded server list, typed status/scope presentation, and filters", async () => {
    global.fetch = jest.fn().mockResolvedValue(response({ items:[client], nextOffset:null }));
    render(<ApiClientsWorkspace canManage />);
    expect(await screen.findByRole("region", { name:"API client registry" })).toHaveClass("overflow-x-auto");
    expect(screen.getByRole("rowheader", { name:client.clientId })).toBeInTheDocument();
    expect(screen.getAllByText("Suspended").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Aggregate analytics").length).toBeGreaterThan(0);
    expect(document.querySelector(`time[datetime="${client.createdAt}"]`)).toHaveTextContent("2026");
    expect(screen.getByRole("link", { name:"Register client" })).toHaveAttribute("href", "/admin/api-clients/new");

    fireEvent.change(screen.getByLabelText("Client ID or name"), { target:{ value:"marketing" } });
    fireEvent.change(screen.getByLabelText("Status"), { target:{ value:"SUSPENDED" } });
    fireEvent.change(screen.getByLabelText("Allowed scope"), { target:{ value:"analytics:read" } });
    fireEvent.click(screen.getByRole("button", { name:"Apply filters" }));
    await waitFor(() => expect(fetch).toHaveBeenLastCalledWith(expect.stringContaining("search=marketing"), expect.anything()));
    expect(fetch).toHaveBeenLastCalledWith(expect.stringContaining("scope=analytics%3Aread"), expect.anything());
  });

  it("keeps read-only staff useful and paginates through the server", async () => {
    jest.mocked(global.fetch = jest.fn())
      .mockResolvedValueOnce(response({ items:[client], nextOffset:1 }))
      .mockResolvedValueOnce(response({ items:[{ ...client, clientId:"XFC2222222222222222", name:"Flight Board" }], nextOffset:null }));
    render(<ApiClientsWorkspace canManage={false} />);
    expect(await screen.findByRole("button", { name:"Load more" })).toBeInTheDocument();
    expect(screen.queryByRole("link", { name:"Register client" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name:"Load more" }));
    expect(await screen.findByRole("rowheader", { name:"XFC2222222222222222" })).toBeInTheDocument();
    expect(fetch).toHaveBeenLastCalledWith(expect.stringContaining("offset=1"), expect.anything());
  });

  it("creates one client then replaces the create route with its authoritative detail route", async () => {
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response(scopes))
      .mockResolvedValueOnce(response(client, true, 201));
    render(<ApiClientEditor canManage mode="new" />);
    await screen.findByText("Aggregate analytics");
    fireEvent.change(screen.getByLabelText("Display name"), { target:{ value:"Marketing Insights" } });
    fireEvent.change(screen.getByLabelText("Description (optional)"), { target:{ value:"Approved aggregates" } });
    fireEvent.click(screen.getByRole("checkbox", { name:/Aggregate analytics/ }));
    fireEvent.click(screen.getByRole("button", { name:"Register client" }));
    await screen.findByText("Client registered.");
    await waitFor(() => expect(mockReplace).toHaveBeenCalledWith(`/admin/api-clients/${client.clientId}`));
    const [, init] = jest.mocked(fetch).mock.calls[1];
    const payload = JSON.parse(String(init?.body));
    expect(payload).toEqual({ name:"Marketing Insights", description:"Approved aggregates", status:"SUSPENDED", allowedScopes:["analytics:read"] });
    expect(JSON.stringify(payload)).not.toMatch(/secret|token|credential/i);
    expect(jest.mocked(fetch).mock.calls.filter(([, options]) => options?.method === "POST")).toHaveLength(1);
    const register = screen.getByRole("button", { name:"Register client" });
    expect(register).toBeDisabled();
    fireEvent.click(register);
    expect(jest.mocked(fetch).mock.calls.filter(([, options]) => options?.method === "POST")).toHaveLength(1);
  });

  it("creates a flights-only client from a reversed catalog without swapping scope identity", async () => {
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response([...scopes].reverse()))
      .mockResolvedValueOnce(response(flightsClient, true, 201));
    render(<ApiClientEditor canManage mode="new" />);
    await screen.findByText("Flight data");
    fireEvent.change(screen.getByLabelText("Display name"), { target:{ value:"Flight Data" } });
    fireEvent.change(screen.getByLabelText("Initial status"), { target:{ value:"ACTIVE" } });
    fireEvent.click(screen.getByRole("checkbox", { name:/Flight data/ }));
    expect(screen.getByRole("checkbox", { name:/Flight data/ })).toBeChecked();
    expect(screen.getByRole("checkbox", { name:/Aggregate analytics/ })).not.toBeChecked();
    fireEvent.click(screen.getByRole("button", { name:"Register client" }));
    await screen.findByText("Client registered.");
    const payload = JSON.parse(String(jest.mocked(fetch).mock.calls[1][1]?.body));
    expect(payload.allowedScopes).toEqual(["flights:read"]);
    expect(screen.getByRole("checkbox", { name:/Flight data/ })).toBeChecked();
    expect(screen.getByRole("checkbox", { name:/Aggregate analytics/ })).not.toBeChecked();
  });

  it("creates both scopes in canonical code order regardless of click and catalog order", async () => {
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response([...scopes].reverse()))
      .mockResolvedValueOnce(response(bothClient, true, 201));
    render(<ApiClientEditor canManage mode="new" />);
    await screen.findByText("Flight data");
    fireEvent.change(screen.getByLabelText("Display name"), { target:{ value:"Both Scopes" } });
    fireEvent.change(screen.getByLabelText("Initial status"), { target:{ value:"ACTIVE" } });
    fireEvent.click(screen.getByRole("checkbox", { name:/Flight data/ }));
    fireEvent.click(screen.getByRole("checkbox", { name:/Aggregate analytics/ }));
    fireEvent.click(screen.getByRole("button", { name:"Register client" }));
    await screen.findByText("Client registered.");
    const payload = JSON.parse(String(jest.mocked(fetch).mock.calls[1][1]?.body));
    expect(payload.allowedScopes).toEqual(["analytics:read", "flights:read"]);
    expect(screen.getByRole("checkbox", { name:/Aggregate analytics/ })).toBeChecked();
    expect(screen.getByRole("checkbox", { name:/Flight data/ })).toBeChecked();
  });

  it("keeps client-side validation on the create route without posting or navigating", async () => {
    global.fetch = jest.fn().mockResolvedValueOnce(response(scopes));
    render(<ApiClientEditor canManage mode="new" />);
    await screen.findByText("Aggregate analytics");
    fireEvent.change(screen.getByLabelText("Display name"), { target:{ value:"x".repeat(101) } });
    fireEvent.click(screen.getByRole("button", { name:"Register client" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Enter a display name of 1–100 characters.");
    expect(window.location.pathname).toBe("/admin/api-clients/new");
    expect(mockReplace).not.toHaveBeenCalled();
    expect(jest.mocked(fetch).mock.calls.filter(([, options]) => options?.method === "POST")).toHaveLength(0);
    expect(screen.getByRole("button", { name:"Register client" })).toBeEnabled();
  });

  it("keeps a localized backend failure retryable on the Thai create route", async () => {
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response(scopes))
      .mockResolvedValueOnce(response({ error:{ code:"API_CLIENT_VALIDATION_FAILED" } }, false, 422));
    render(<ApiClientEditor canManage mode="new" />, { locale:"th" });
    await screen.findByText("ข้อมูลวิเคราะห์แบบรวม");
    fireEvent.change(screen.getByLabelText("ชื่อที่แสดง"), { target:{ value:"Marketing Insights" } });
    fireEvent.click(screen.getByRole("button", { name:"ลงทะเบียนไคลเอนต์" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("โปรดตรวจสอบข้อมูลไคลเอนต์ API แล้วลองอีกครั้ง");
    expect(window.location.pathname).toBe("/admin/api-clients/new");
    expect(mockReplace).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name:"ลงทะเบียนไคลเอนต์" })).toBeEnabled();
  });

  it("edits from analytics to flights while keeping Client ID immutable", async () => {
    const saved: ApiClientRecord = {
      ...client,
      name:"Marketing Aggregate Terminal",
      description:"Authoritative flight data",
      allowedScopes:["flights:read"],
      version:2,
      updatedAt:"2026-09-09T03:00:00Z",
      updatedBy:"editor@x.test",
    };
    const synchronized: ApiClientDetail = {
      ...saved,
      audit:[...detail.audit, auditEntry("CLIENT_SCOPES_UPDATED", saved)],
    };
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response(scopes))
      .mockResolvedValueOnce(response(detail))
      .mockResolvedValueOnce(response(saved))
      .mockResolvedValueOnce(response(synchronized));
    render(<ApiClientEditor canManage clientId={client.clientId} mode="detail" />);
    expect((await screen.findAllByText(client.clientId)).length).toBeGreaterThan(0);
    expect(screen.queryByDisplayValue(client.clientId)).not.toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Display name"), { target:{ value:"Marketing Aggregate Terminal" } });
    fireEvent.change(screen.getByLabelText("Description (optional)"), { target:{ value:"Authoritative flight data" } });
    fireEvent.click(screen.getByRole("checkbox", { name:/Aggregate analytics/ }));
    fireEvent.click(screen.getByRole("checkbox", { name:/Flight data/ }));
    fireEvent.click(screen.getByRole("button", { name:"Save changes" }));
    await screen.findByText("Changes saved.");
    const [url, init] = jest.mocked(fetch).mock.calls[2];
    expect(url).toBe(`/admin/api/api-clients/${client.clientId}`);
    expect(JSON.parse(String(init?.body))).toEqual({ name:"Marketing Aggregate Terminal", description:"Authoritative flight data", allowedScopes:["flights:read"], version:1 });
    expect(screen.getByRole("checkbox", { name:/Aggregate analytics/ })).not.toBeChecked();
    expect(screen.getByRole("checkbox", { name:/Flight data/ })).toBeChecked();
    expect(screen.getByLabelText("Description (optional)")).toHaveValue("Authoritative flight data");
    expect(await screen.findByText("Allowed scopes updated")).toBeInTheDocument();
    expect(screen.getByText("Last changed by editor@x.test")).toBeInTheDocument();
    expect(document.querySelector('time[datetime="2026-09-09T03:00:00Z"]')).toBeInTheDocument();
    expect(jest.mocked(fetch).mock.calls.filter(([url, options]) => url === `/admin/api/api-clients/${client.clientId}` && !options?.method)).toHaveLength(2);

    jest.mocked(fetch)
      .mockResolvedValueOnce(response({ ...saved, description:"Second update", version:3 }))
      .mockResolvedValueOnce(response({ ...synchronized, description:"Second update", version:3 }));
    fireEvent.change(screen.getByLabelText("Description (optional)"), { target:{ value:"Second update" } });
    fireEvent.click(screen.getByRole("button", { name:"Save changes" }));
    await waitFor(() => expect(screen.getByLabelText("Description (optional)")).toHaveValue("Second update"));
    const updateCalls = jest.mocked(fetch).mock.calls.filter(([, options]) => options?.method === "PUT");
    expect(updateCalls).toHaveLength(2);
    expect(JSON.parse(String(updateCalls[1][1]?.body)).version).toBe(2);
  });

  it("synchronizes suspend and activate state, audit, controls, and version without a reload", async () => {
    const activeDetail: ApiClientDetail = { ...detail, status:"ACTIVE", allowedScopes:["analytics:read"] };
    const suspended: ApiClientRecord = { ...client, status:"SUSPENDED", version:2, updatedAt:"2026-09-09T03:00:00Z", updatedBy:"lifecycle@x.test" };
    const suspendedDetail: ApiClientDetail = { ...suspended, audit:[...detail.audit, auditEntry("CLIENT_SUSPENDED", suspended)] };
    const activated: ApiClientRecord = { ...suspended, status:"ACTIVE", version:3, updatedAt:"2026-09-09T04:00:00Z" };
    const activatedDetail: ApiClientDetail = { ...activated, audit:[...suspendedDetail.audit, auditEntry("CLIENT_ACTIVATED", activated)] };
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response(scopes))
      .mockResolvedValueOnce(response(activeDetail))
      .mockResolvedValueOnce(response(suspended))
      .mockResolvedValueOnce(response(suspendedDetail))
      .mockResolvedValueOnce(response(activated))
      .mockResolvedValueOnce(response(activatedDetail));
    render(<ApiClientEditor canManage clientId={client.clientId} mode="detail" />);
    const suspendTrigger = await screen.findByRole("button", { name:"Suspend client" });
    suspendTrigger.focus();
    fireEvent.click(suspendTrigger);
    const suspendDialog = screen.getByRole("dialog", { name:"Suspend this client?" });
    expect(within(suspendDialog).queryByRole("button", { name:"Close dialog" })).not.toBeInTheDocument();
    await waitFor(() => expect(within(suspendDialog).getByRole("button", { name:"Keep current status" })).toHaveFocus());
    fireEvent.keyDown(document, { key:"Tab", shiftKey:true });
    expect(suspendDialog).toContainElement(document.activeElement as HTMLElement);
    fireEvent.keyDown(document, { key:"Escape" });
    expect(screen.queryByRole("dialog", { name:"Suspend this client?" })).not.toBeInTheDocument();
    await waitFor(() => expect(suspendTrigger).toHaveFocus());

    fireEvent.click(suspendTrigger);
    fireEvent.click(screen.getByRole("button", { name:"Confirm suspension" }));
    expect(await screen.findByText("Client suspended")).toBeInTheDocument();
    expect(screen.getByText("Suspended")).toBeInTheDocument();
    expect(screen.getByRole("button", { name:"Activate client" })).toBeEnabled();

    fireEvent.click(screen.getByRole("button", { name:"Activate client" }));
    const activateDialog = screen.getByRole("dialog", { name:"Activate this client?" });
    expect(within(activateDialog).queryByRole("button", { name:"Close dialog" })).not.toBeInTheDocument();
    fireEvent.click(within(activateDialog).getByRole("button", { name:"Confirm activation" }));
    expect(await screen.findByText("Client activated")).toBeInTheDocument();
    expect(screen.getByText("Active")).toBeInTheDocument();
    expect(screen.getByRole("button", { name:"Suspend client" })).toBeEnabled();

    const mutations = jest.mocked(fetch).mock.calls.filter(([, options]) => options?.method === "POST");
    expect(mutations).toHaveLength(2);
    expect(JSON.parse(String(mutations[0][1]?.body)).version).toBe(1);
    expect(JSON.parse(String(mutations[1][1]?.body)).version).toBe(2);
    expect(jest.mocked(fetch).mock.calls.filter(([url, options]) => url === `/admin/api/api-clients/${client.clientId}` && !options?.method)).toHaveLength(3);
  });

  it("requires an accessible confirmation and synchronizes terminal revocation", async () => {
    const revoked: ApiClientRecord = { ...client, status:"REVOKED", version:2, updatedAt:"2026-09-09T03:00:00Z", updatedBy:"revoker@x.test" };
    const revokedDetail: ApiClientDetail = { ...revoked, audit:[...detail.audit, auditEntry("CLIENT_REVOKED", revoked, "revoker@x.test")] };
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response(scopes))
      .mockResolvedValueOnce(response(detail))
      .mockResolvedValueOnce(response(revoked))
      .mockResolvedValueOnce(response(revokedDetail));
    render(<ApiClientEditor canManage clientId={client.clientId} mode="detail" />);
    await screen.findAllByText(client.clientId);
    fireEvent.click(screen.getByRole("button", { name:"Revoke client" }));
    const dialog = screen.getByRole("dialog", { name:"Permanently revoke this client?" });
    expect(within(dialog).getByText(/cannot be reactivated/)).toBeInTheDocument();
    expect(within(dialog).queryByRole("button", { name:"Close dialog" })).not.toBeInTheDocument();
    fireEvent.click(within(dialog).getByRole("button", { name:"Confirm revocation" }));
    expect(await screen.findByText("Client revoked")).toBeInTheDocument();
    expect(screen.getByText("Revoked")).toBeInTheDocument();
    expect(jest.mocked(fetch).mock.calls.filter(([url, options]) => url === `/admin/api/api-clients/${client.clientId}/revoke` && options?.method === "POST")).toHaveLength(1);
    expect(jest.mocked(fetch).mock.calls.filter(([url, options]) => url === `/admin/api/api-clients/${client.clientId}` && !options?.method)).toHaveLength(2);
    expect(screen.queryByRole("button", { name:"Save changes" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name:"Activate client" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name:"Suspend client" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name:"Revoke client" })).not.toBeInTheDocument();
    expect(screen.getByLabelText("Display name")).toBeDisabled();
  });

  it("keeps the prior detail when a mutation fails", async () => {
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response(scopes))
      .mockResolvedValueOnce(response({ ...detail, status:"ACTIVE" }))
      .mockResolvedValueOnce(response({ error:{ code:"API_CLIENT_STALE_VERSION" } }, false, 409));
    render(<ApiClientEditor canManage clientId={client.clientId} mode="detail" />);
    fireEvent.click(await screen.findByRole("button", { name:"Suspend client" }));
    fireEvent.click(screen.getByRole("button", { name:"Confirm suspension" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("This client changed after the page loaded");
    fireEvent.click(screen.getByRole("button", { name:"Keep current status" }));
    expect(screen.getByText("Active")).toBeInTheDocument();
    expect(screen.getByText("Client registered")).toBeInTheDocument();
    expect(screen.getByRole("button", { name:"Suspend client" })).toBeEnabled();
    expect(jest.mocked(fetch).mock.calls).toHaveLength(3);
  });

  it("retains a successful mutation record and reports a failed detail synchronization", async () => {
    const suspended: ApiClientRecord = { ...client, status:"SUSPENDED", version:2, updatedAt:"2026-09-09T03:00:00Z", updatedBy:"lifecycle@x.test" };
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response(scopes))
      .mockResolvedValueOnce(response({ ...detail, status:"ACTIVE" }))
      .mockResolvedValueOnce(response(suspended))
      .mockResolvedValueOnce(response(null, false, 503));
    render(<ApiClientEditor canManage clientId={client.clientId} mode="detail" />);
    fireEvent.click(await screen.findByRole("button", { name:"Suspend client" }));
    fireEvent.click(screen.getByRole("button", { name:"Confirm suspension" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Client updated, but the latest audit history could not be loaded");
    expect(screen.getByText("Suspended")).toBeInTheDocument();
    expect(screen.getByText("Last changed by lifecycle@x.test")).toBeInTheDocument();
    expect(screen.getByRole("button", { name:"Activate client" })).toBeEnabled();
    expect(screen.getByText("Client registered")).toBeInTheDocument();
    expect(jest.mocked(fetch).mock.calls.filter(([, options]) => options?.method === "POST")).toHaveLength(1);
    expect(jest.mocked(fetch).mock.calls.filter(([url, options]) => url === `/admin/api/api-clients/${client.clientId}` && !options?.method)).toHaveLength(2);
  });

  it("localizes a post-mutation synchronization failure in Thai", async () => {
    const suspended: ApiClientRecord = { ...client, status:"SUSPENDED", version:2 };
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response(scopes))
      .mockResolvedValueOnce(response({ ...detail, status:"ACTIVE" }))
      .mockResolvedValueOnce(response(suspended))
      .mockResolvedValueOnce(response(null, false, 503));
    render(<ApiClientEditor canManage clientId={client.clientId} mode="detail" />, { locale:"th" });
    fireEvent.click(await screen.findByRole("button", { name:"ระงับไคลเอนต์" }));
    fireEvent.click(screen.getByRole("button", { name:"ยืนยันการระงับ" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("อัปเดตไคลเอนต์แล้ว แต่ไม่สามารถโหลดประวัติการตรวจสอบล่าสุดได้");
    expect(jest.mocked(fetch).mock.calls.filter(([, options]) => options?.method === "POST")).toHaveLength(1);
  });

  it.each(["ACTIVE", "SUSPENDED"] as const)("renders an enabled keyboard-focusable revoke action for %s clients", async (status) => {
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response(scopes))
      .mockResolvedValueOnce(response({ ...detail, status }));
    render(<ApiClientEditor canManage clientId={client.clientId} mode="detail" />);
    const revoke = await screen.findByRole("button", { name:"Revoke client" });
    expect(revoke).toBeEnabled();
    expect(revoke).toHaveAttribute("type", "button");
    expect(revoke).toHaveClass("is-danger");
    revoke.focus();
    expect(revoke).toHaveFocus();
  });

  it("does not expose an actionable revoke control for a revoked client", async () => {
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response(scopes))
      .mockResolvedValueOnce(response({ ...detail, status:"REVOKED" }));
    render(<ApiClientEditor canManage clientId={client.clientId} mode="detail" />);
    await screen.findByText("Revoked");
    expect(screen.queryByRole("button", { name:"Revoke client" })).not.toBeInTheDocument();
  });

  it("localizes statuses, scopes, forms, history, and empty states in Thai", async () => {
    global.fetch = jest.fn().mockResolvedValue(response({ items:[client], nextOffset:null }));
    render(<ApiClientsWorkspace canManage={false} />, { locale:"th" });
    expect(await screen.findByRole("heading", { name:"การจัดการไคลเอนต์ API" })).toBeInTheDocument();
    await screen.findByRole("rowheader", { name:client.clientId });
    expect(screen.getAllByText("ระงับการใช้งาน").length).toBeGreaterThan(1);
    expect(screen.getAllByText("ข้อมูลวิเคราะห์แบบรวม").length).toBeGreaterThan(1);
    expect(screen.getByRole("rowheader", { name:client.clientId })).toBeInTheDocument();
    expect(document.querySelector(`time[datetime="${client.createdAt}"]`)).toHaveTextContent("2569");
  });

  it("uses the Thai staff calendar for detail audit timestamps without altering identifiers or scope codes", async () => {
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response(scopes))
      .mockResolvedValueOnce(response(detail));
    render(<ApiClientEditor canManage={false} clientId={client.clientId} mode="detail" />, { locale:"th" });
    expect((await screen.findAllByText(client.clientId)).length).toBeGreaterThan(0);
    expect(screen.getByText("analytics:read")).toBeInTheDocument();
    expect(document.querySelector(`time[datetime="${client.createdAt}"]`)).toHaveTextContent("2569");
  });

  it("updates staff date presentation on EN/TH switching without changing machine values", async () => {
    global.fetch = jest.fn().mockResolvedValue(response({ items:[client], nextOffset:null }));
    render(<LocaleSwitchingApiClients />);
    await screen.findByRole("rowheader", { name:client.clientId });
    const created = document.querySelector(`time[datetime="${client.createdAt}"]`);
    expect(created).toHaveTextContent("2026");

    fireEvent.click(screen.getByRole("button", { name:"Toggle locale" }));

    expect(created).toHaveTextContent("2569");
    expect(created).toHaveAttribute("datetime", client.createdAt);
    expect(screen.getByRole("rowheader", { name:client.clientId })).toBeInTheDocument();
    expect(screen.getByText("analytics:read")).toBeInTheDocument();
  });
});
