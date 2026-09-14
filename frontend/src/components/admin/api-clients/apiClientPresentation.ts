import type { TranslationKey } from "@/i18n/types";
import type { ApiClientAuditAction, ApiClientScope, ApiClientStatus } from "@/lib/admin/apiClientTypes";

const statusKey: Record<ApiClientStatus, TranslationKey> = {
  ACTIVE:"apiClientManagement.status.active",
  SUSPENDED:"apiClientManagement.status.suspended",
  REVOKED:"apiClientManagement.status.revoked",
};
const scopeTitleKey: Record<ApiClientScope, TranslationKey> = {
  "analytics:read":"apiClientManagement.scope.analyticsTitle",
  "flights:read":"apiClientManagement.scope.flightsTitle",
};
const scopeDescriptionKey: Record<ApiClientScope, TranslationKey> = {
  "analytics:read":"apiClientManagement.scope.analyticsDescription",
  "flights:read":"apiClientManagement.scope.flightsDescription",
};
const auditActionKey: Record<ApiClientAuditAction, TranslationKey> = {
  CLIENT_CREATED:"apiClientManagement.audit.created",
  CLIENT_METADATA_UPDATED:"apiClientManagement.audit.metadata",
  CLIENT_SCOPES_UPDATED:"apiClientManagement.audit.scopes",
  CLIENT_ACTIVATED:"apiClientManagement.audit.activated",
  CLIENT_SUSPENDED:"apiClientManagement.audit.suspended",
  CLIENT_REVOKED:"apiClientManagement.audit.revoked",
  CREDENTIAL_ISSUED:"apiClientManagement.audit.credentialIssued",
  CREDENTIAL_REVOKED:"apiClientManagement.audit.credentialRevoked",
};

const safeAuditActionKey = (action:string):TranslationKey => {
  if (Object.prototype.hasOwnProperty.call(auditActionKey, action)) return auditActionKey[action as ApiClientAuditAction];
  return "apiClientManagement.audit.unknown";
};

export { auditActionKey, safeAuditActionKey, scopeDescriptionKey, scopeTitleKey, statusKey };
