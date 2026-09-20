-- Branch 25 runtime correction: expose only the credential and token columns
-- required by the staff API-client and external-auth services.
-- No secret-bearing or immutable columns are writable by x_fly_runtime beyond
-- the explicitly approved credential issuance/revocation fields.
BEGIN;

REVOKE ALL PRIVILEGES ON TABLE
    public.api_client_management_audit,
    public.api_client_credentials,
    public.external_access_tokens
FROM x_fly_runtime, PUBLIC;

GRANT SELECT ON TABLE public.api_client_management_audit
TO x_fly_runtime;

GRANT INSERT (
    api_client_id,
    actor_staff_user_id,
    action,
    before_state,
    after_state,
    credential_id,
    created_at
)
ON TABLE public.api_client_management_audit
TO x_fly_runtime;

GRANT SELECT (
    id,
    secret_digest,
    digest_version,
    issued_at,
    revoked_at,
    revocation_reason
)
ON TABLE public.api_client_credentials
TO x_fly_runtime;

GRANT INSERT (
    api_client_id,
    secret_digest,
    digest_version,
    issued_at,
    issued_by_staff_user_id
)
ON TABLE public.api_client_credentials
TO x_fly_runtime;

GRANT UPDATE (
    revoked_at,
    revoked_by_staff_user_id,
    revocation_reason
)
ON TABLE public.api_client_credentials
TO x_fly_runtime;

GRANT SELECT (
    id,
    api_client_credential_id,
    token_hash,
    issued_at,
    expires_at,
    revoked_at
)
ON TABLE public.external_access_tokens
TO x_fly_runtime;

GRANT INSERT (
    api_client_credential_id,
    token_hash,
    issued_at,
    expires_at
)
ON TABLE public.external_access_tokens
TO x_fly_runtime;

GRANT UPDATE (revoked_at)
ON TABLE public.external_access_tokens
TO x_fly_runtime;

COMMIT;
