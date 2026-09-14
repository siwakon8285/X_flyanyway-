const API_CLIENT_STATUSES = ["ACTIVE", "SUSPENDED", "REVOKED"] as const;
const API_CLIENT_SCOPES = ["analytics:read", "flights:read"] as const;

type ApiClientStatus = (typeof API_CLIENT_STATUSES)[number];
type ApiClientScope = (typeof API_CLIENT_SCOPES)[number];

type ApiClientRecord = {
  clientId: string;
  name: string;
  description: string | null;
  status: ApiClientStatus;
  allowedScopes: ApiClientScope[];
  version: number;
  createdAt: string;
  updatedAt: string;
  createdBy: string;
  updatedBy: string;
};

type ApiClientPage = { items: ApiClientRecord[]; nextOffset: number | null };
type ApiClientAuditSnapshot = Pick<ApiClientRecord, "name" | "description" | "status" | "allowedScopes">;
type ApiClientAuditAction =
  | "CLIENT_CREATED"
  | "CLIENT_METADATA_UPDATED"
  | "CLIENT_SCOPES_UPDATED"
  | "CLIENT_ACTIVATED"
  | "CLIENT_SUSPENDED"
  | "CLIENT_REVOKED"
  | "CREDENTIAL_ISSUED"
  | "CREDENTIAL_REVOKED";
type ApiClientAuditEntry = {
  actorEmail: string;
  action: ApiClientAuditAction;
  before: ApiClientAuditSnapshot | null;
  after: ApiClientAuditSnapshot;
  createdAt: string;
};
type ApiClientCredentialMetadata = {
  hasLiveCredential: boolean;
  issuedAt: string | null;
  revokedAt: string | null;
};
type IssuedCredentialResponse = {
  clientId: string;
  clientSecret: string;
  issuedAt: string;
};
type ApiClientDetail = ApiClientRecord & {
  audit: ApiClientAuditEntry[];
  credentialMetadata: ApiClientCredentialMetadata;
};
type ApiScopeDefinition = { code: ApiClientScope; description: string };

export { API_CLIENT_SCOPES, API_CLIENT_STATUSES };
export type {
  ApiClientAuditAction,
  ApiClientAuditEntry,
  ApiClientAuditSnapshot,
  ApiClientCredentialMetadata,
  ApiClientDetail,
  ApiClientPage,
  ApiClientRecord,
  ApiClientScope,
  ApiClientStatus,
  ApiScopeDefinition,
  IssuedCredentialResponse,
};
