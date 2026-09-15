import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";

import { ApiClientEditor } from "@/components/admin/api-clients/ApiClientEditor";
import { safeAuditActionKey } from "@/components/admin/api-clients/apiClientPresentation";
import type { ApiClientDetail, ApiClientRecord, ApiScopeDefinition } from "@/lib/admin/apiClientTypes";
import { render } from "@/tests/renderWithLanguage";

const mockReplace = jest.fn();

jest.mock("next/navigation", () => ({
  useRouter: () => ({ replace: mockReplace }),
}));

const scopes: ApiScopeDefinition[] = [
  { code: "analytics:read", description: "Aggregate analytics" },
  { code: "flights:read", description: "Flight data" },
];
const client: ApiClientRecord = {
  clientId: "XFCABCDEFGHJKLMNPQR",
  name: "Credential Test Client",
  description: "Task 9 test client",
  status: "ACTIVE",
  allowedScopes: ["flights:read"],
  version: 7,
  createdAt: "2026-09-09T02:00:00Z",
  updatedAt: "2026-09-09T02:00:00Z",
  createdBy: "api@x.test",
  updatedBy: "api@x.test",
};
const metadata = {
  hasLiveCredential: false,
  issuedAt: null,
  revokedAt: null,
};
const detail: ApiClientDetail = {
  ...client,
  audit: [],
  credentialMetadata: metadata,
};
const response = (value: unknown, ok = true, status = 200) =>
  ({ ok, status, json: async () => value } as Response);
const loadDetail = (overrides: Partial<ApiClientDetail> = {}) => ({
  ...detail,
  ...overrides,
  credentialMetadata: overrides.credentialMetadata ?? metadata,
});
const setupDetail = (nextDetail: ApiClientDetail = detail) => {
  global.fetch = jest.fn()
    .mockResolvedValueOnce(response(scopes))
    .mockResolvedValueOnce(response(nextDetail));
};

describe("API client credential administration", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    window.history.replaceState({}, "", "/admin/api-clients/XFCABCDEFGHJKLMNPQR");
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText: jest.fn().mockResolvedValue(undefined) },
    });
  });

  it("shows issue only to staff with api_clients:manage", async () => {
    setupDetail();
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    await screen.findByText("Credential Test Client");
    expect(screen.getByRole("button", { name: "Issue credential" })).toBeInTheDocument();

    cleanup();
    setupDetail();
    render(<ApiClientEditor canManage={false} mode="detail" clientId={client.clientId} />);
    await screen.findAllByText("Credential Test Client");
    expect(screen.queryByRole("button", { name: "Issue credential" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Revoke credential" })).not.toBeInTheDocument();
  });

  it("disables issue while the mutation is pending and sends the current version", async () => {
    let resolveIssue: ((value: Response) => void) | undefined;
    setupDetail();
    jest.mocked(fetch).mockImplementationOnce(() => new Promise((resolve) => { resolveIssue = resolve; }));
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    const issue = await screen.findByRole("button", { name: "Issue credential" });
    fireEvent.click(issue);
    expect(issue).toBeDisabled();
    expect(jest.mocked(fetch).mock.calls.filter(([url]) => String(url).includes("/credentials"))).toHaveLength(1);
    expect(JSON.parse(String(jest.mocked(fetch).mock.calls.at(-1)?.[1]?.body))).toEqual({ version: 7 });
    resolveIssue?.(response({ clientId: client.clientId, clientSecret: "a".repeat(64), issuedAt: "2026-09-09T03:00:00Z" }, true, 201));
  });

  it("opens a one-time modal, copies only on explicit action, and guards dismissal", async () => {
    const secret = "a".repeat(64);
    setupDetail();
    jest.mocked(fetch)
      .mockResolvedValueOnce(response({ clientId: client.clientId, clientSecret: secret, issuedAt: "2026-09-09T03:00:00Z" }, true, 201))
      .mockResolvedValueOnce(response(loadDetail({ credentialMetadata: { hasLiveCredential: true, issuedAt: "2026-09-09T03:00:00Z", revokedAt: null } })));
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    fireEvent.click(await screen.findByRole("button", { name: "Issue credential" }));
    const modal = await screen.findByRole("dialog", { name: "Credential issued" });
    expect(within(modal).getByTestId("credential-client-id")).toHaveTextContent(client.clientId);
    expect(within(modal).getByTestId("credential-secret")).toHaveTextContent(secret);
    expect(within(modal).getByRole("checkbox", { name: /saved this credential securely/i })).not.toBeChecked();
    expect(within(modal).getByRole("button", { name: "Continue" })).toBeDisabled();
    expect(within(modal).queryByRole("button", { name: "Close dialog" })).not.toBeInTheDocument();

    fireEvent.keyDown(document, { key: "Escape" });
    expect(screen.getByRole("dialog", { name: "Credential issued" })).toBeInTheDocument();
    fireEvent.click(within(modal).getByRole("button", { name: "Copy client secret" }));
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith(secret);
    expect((await within(modal).findAllByText("Copied")).length).toBeGreaterThan(0);
    expect(within(modal).getByText(secret)).toBeInTheDocument();

    fireEvent.click(within(modal).getByRole("checkbox", { name: /saved this credential securely/i }));
    fireEvent.click(within(modal).getByRole("button", { name: "Continue" }));
    await waitFor(() => expect(screen.queryByTestId("credential-secret")).not.toBeInTheDocument());
  });

  it("does not expose the secret before issuance or write it to storage, URL, or console", async () => {
    const log = jest.spyOn(console, "log").mockImplementation(() => undefined);
    const local = jest.spyOn(Storage.prototype, "setItem");
    setupDetail();
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    await screen.findByText("Credential Test Client");
    expect(screen.queryByTestId("credential-secret")).not.toBeInTheDocument();
    expect(local).not.toHaveBeenCalled();
    expect(window.location.href).not.toContain("secret");
    expect(log).not.toHaveBeenCalled();
    log.mockRestore();
    local.mockRestore();
  });

  it("requires confirmation to revoke and sends only the current version", async () => {
    const live = loadDetail({ credentialMetadata: { hasLiveCredential: true, issuedAt: "2026-09-09T03:00:00Z", revokedAt: null } });
    const revoked = loadDetail({ version: 8, credentialMetadata: { hasLiveCredential: false, issuedAt: "2026-09-09T03:00:00Z", revokedAt: "2026-09-09T04:00:00Z" } });
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response(scopes))
      .mockResolvedValueOnce(response(live))
      .mockResolvedValueOnce(response(revoked))
      .mockResolvedValueOnce(response(revoked));
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    fireEvent.click(await screen.findByRole("button", { name: "Revoke credential" }));
    const dialog = screen.getByRole("dialog", { name: "Revoke this credential?" });
    expect(within(dialog).getByRole("button", { name: "Cancel" })).toBeInTheDocument();
    expect(jest.mocked(fetch).mock.calls.filter(([, options]) => options?.method === "POST")).toHaveLength(0);
    fireEvent.click(within(dialog).getByRole("button", { name: "Revoke credential" }));
    await waitFor(() => expect(screen.getByRole("button", { name: "Issue credential" })).toBeInTheDocument());
    const revokeCall = jest.mocked(fetch).mock.calls.find(([url]) => String(url).includes("/credentials/revoke"));
    expect(JSON.parse(String(revokeCall?.[1]?.body))).toEqual({ version: 7 });
  });

  it("does not offer reissue while a live credential exists and enables it after revoke", async () => {
    const live = loadDetail({ credentialMetadata: { hasLiveCredential: true, issuedAt: "2026-09-09T03:00:00Z", revokedAt: null } });
    setupDetail(live);
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    await screen.findByRole("button", { name: "Revoke credential" });
    expect(screen.queryByRole("button", { name: "Issue credential" })).not.toBeInTheDocument();
  });

  it("makes exactly one issue request after an ambiguous transport failure and offers safe recovery", async () => {
    const live = loadDetail({ credentialMetadata: { hasLiveCredential: true, issuedAt: "2026-09-09T03:00:00Z", revokedAt: null } });
    setupDetail();
    jest.mocked(fetch)
      .mockRejectedValueOnce(new TypeError("network disconnected"))
      .mockResolvedValueOnce(response(live));
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    fireEvent.click(await screen.findByRole("button", { name: "Issue credential" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(/may have succeeded|uncertain/i);
    expect(await screen.findByText(/Revoke it, then issue a replacement/i)).toBeInTheDocument();
    expect(jest.mocked(fetch).mock.calls.filter(([url]) => String(url).endsWith("/credentials"))).toHaveLength(1);
  });

  it("blocks a second issue while an ambiguous result cannot be reconciled", async () => {
    setupDetail();
    jest.mocked(fetch)
      .mockRejectedValueOnce(new TypeError("network disconnected"))
      .mockRejectedValueOnce(new TypeError("status unavailable"))
      .mockResolvedValueOnce(response(loadDetail()));
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    fireEvent.click(await screen.findByRole("button", { name: "Issue credential" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(/uncertain/i);
    const issue = screen.getByRole("button", { name: "Issue credential" });
    expect(issue).toBeDisabled();
    fireEvent.click(issue);
    expect(jest.mocked(fetch).mock.calls.filter(([url]) => String(url).endsWith("/credentials"))).toHaveLength(1);
    expect(screen.queryByTestId("credential-secret")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Refresh credential status" }));
    await waitFor(() => expect(screen.getByRole("button", { name: "Issue credential" })).not.toBeDisabled());
    expect(jest.mocked(fetch).mock.calls.filter(([url]) => String(url).endsWith("/credentials"))).toHaveLength(1);
  });

  it("keeps issue unavailable when reconciliation finds a live credential", async () => {
    const live = loadDetail({ credentialMetadata: { hasLiveCredential: true, issuedAt: "2026-09-09T03:00:00Z", revokedAt: null } });
    setupDetail();
    jest.mocked(fetch)
      .mockRejectedValueOnce(new TypeError("network disconnected"))
      .mockResolvedValueOnce(response(live));
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    fireEvent.click(await screen.findByRole("button", { name: "Issue credential" }));
    await screen.findByText(/Revoke it, then issue a replacement/i);
    expect(screen.queryByRole("button", { name: "Issue credential" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Revoke credential" })).toBeInTheDocument();
    expect(jest.mocked(fetch).mock.calls.filter(([url]) => String(url).endsWith("/credentials"))).toHaveLength(1);
  });

  it("resolves ambiguous recovery only after authoritative no-live status and permits a later explicit issue", async () => {
    const noLive = loadDetail();
    const issued = { clientId: client.clientId, clientSecret: "a".repeat(64), issuedAt: "2026-09-09T03:00:00Z" };
    setupDetail();
    jest.mocked(fetch)
      .mockRejectedValueOnce(new TypeError("network disconnected"))
      .mockResolvedValueOnce(response(noLive))
      .mockResolvedValueOnce(response(issued, true, 201))
      .mockResolvedValueOnce(response(loadDetail({ credentialMetadata: { hasLiveCredential: true, issuedAt: issued.issuedAt, revokedAt: null } })));
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    fireEvent.click(await screen.findByRole("button", { name: "Issue credential" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(/uncertain/i);
    await waitFor(() => expect(screen.queryByRole("status")).not.toBeInTheDocument());
    const issue = screen.getByRole("button", { name: "Issue credential" });
    expect(issue).not.toBeDisabled();
    expect(jest.mocked(fetch).mock.calls.filter(([url]) => String(url).endsWith("/credentials"))).toHaveLength(1);
    fireEvent.click(issue);
    await screen.findByRole("dialog", { name: "Credential issued" });
    expect(jest.mocked(fetch).mock.calls.filter(([url]) => String(url).endsWith("/credentials"))).toHaveLength(2);
  });

  it("treats an unreadable successful issuance body as ambiguous without retrying", async () => {
    const live = loadDetail({ credentialMetadata: { hasLiveCredential: true, issuedAt: "2026-09-09T03:00:00Z", revokedAt: null } });
    setupDetail();
    jest.mocked(fetch)
      .mockResolvedValueOnce({ ok:true, status:201, json:async () => { throw new TypeError("response body lost"); } } as unknown as Response)
      .mockResolvedValueOnce(response(live));
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    fireEvent.click(await screen.findByRole("button", { name: "Issue credential" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(/may have succeeded|uncertain/i);
    expect(await screen.findByText(/Revoke it, then issue a replacement/i)).toBeInTheDocument();
    expect(jest.mocked(fetch).mock.calls.filter(([url]) => String(url).endsWith("/credentials"))).toHaveLength(1);
    expect(screen.queryByTestId("credential-secret")).not.toBeInTheDocument();
  });

  it("treats a BFF 5xx issuance response as ambiguous without retrying", async () => {
    const live = loadDetail({ credentialMetadata: { hasLiveCredential: true, issuedAt: "2026-09-09T03:00:00Z", revokedAt: null } });
    setupDetail();
    jest.mocked(fetch)
      .mockResolvedValueOnce(response({ error: { code: "API_CLIENT_MANAGEMENT_UNAVAILABLE" } }, false, 503))
      .mockResolvedValueOnce(response(live));
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    fireEvent.click(await screen.findByRole("button", { name: "Issue credential" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(/may have succeeded|uncertain/i);
    expect(await screen.findByText(/Revoke it, then issue a replacement/i)).toBeInTheDocument();
    expect(jest.mocked(fetch).mock.calls.filter(([url]) => String(url).endsWith("/credentials"))).toHaveLength(1);
  });

  it("refreshes safe metadata without reconstructing a lost secret", async () => {
    const live = loadDetail({ credentialMetadata: { hasLiveCredential: true, issuedAt: "2026-09-09T03:00:00Z", revokedAt: null } });
    setupDetail();
    jest.mocked(fetch)
      .mockRejectedValueOnce(new TypeError("timeout"))
      .mockResolvedValueOnce(response(live));
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    fireEvent.click(await screen.findByRole("button", { name: "Issue credential" }));
    await screen.findByText(/live credential/i);
    expect(screen.queryByTestId("credential-secret")).not.toBeInTheDocument();
    expect(screen.queryByText(/View secret|Recover secret/i)).not.toBeInTheDocument();
  });

  it("keeps a successful revoke separate from a failed detail refresh", async () => {
    const live = loadDetail({ credentialMetadata: { hasLiveCredential: true, issuedAt: "2026-09-09T03:00:00Z", revokedAt: null } });
    global.fetch = jest.fn()
      .mockResolvedValueOnce(response(scopes))
      .mockResolvedValueOnce(response(live))
      .mockResolvedValueOnce(response({ hasLiveCredential: false, issuedAt: live.credentialMetadata.issuedAt, revokedAt: "2026-09-09T04:00:00Z" }))
      .mockResolvedValueOnce(response({}, false, 503));
    render(<ApiClientEditor canManage mode="detail" clientId={client.clientId} />);
    fireEvent.click(await screen.findByRole("button", { name: "Revoke credential" }));
    const dialog = screen.getByRole("dialog", { name: "Revoke this credential?" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Revoke credential" }));
    await waitFor(() => expect(screen.getByText("Credential revoked.")).toBeInTheDocument());
    expect(screen.queryByText("detail-load")).not.toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent(/refresh/i);
    expect(screen.queryByRole("button", { name: "Issue credential" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Revoke credential" })).not.toBeInTheDocument();
    expect(jest.mocked(fetch).mock.calls.filter(([url]) => String(url).includes("/credentials/revoke"))).toHaveLength(1);
  });

  it("uses safe labels for credential audit actions and unknown runtime actions", () => {
    expect(safeAuditActionKey("CREDENTIAL_ISSUED")).toBe("apiClientManagement.audit.credentialIssued");
    expect(safeAuditActionKey("CREDENTIAL_REVOKED")).toBe("apiClientManagement.audit.credentialRevoked");
    expect(safeAuditActionKey("INTERNAL_SECRET_DIGEST_EXPOSED")).toBe("apiClientManagement.audit.unknown");
  });
});
