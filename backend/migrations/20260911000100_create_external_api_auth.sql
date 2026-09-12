CREATE TABLE api_client_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_client_id UUID NOT NULL REFERENCES api_clients(id) ON DELETE RESTRICT,
    secret_digest BYTEA NOT NULL CHECK (octet_length(secret_digest) = 32),
    digest_version SMALLINT NOT NULL CHECK (digest_version = 1),
    issued_at TIMESTAMPTZ NOT NULL,
    issued_by_staff_user_id UUID NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    revoked_at TIMESTAMPTZ,
    revoked_by_staff_user_id UUID REFERENCES staff_users(id) ON DELETE RESTRICT,
    revocation_reason TEXT CHECK (
        revocation_reason IS NULL
        OR revocation_reason IN (
            'ADMIN_REQUEST',
            'CLIENT_SUSPENDED',
            'CLIENT_REVOKED',
            'REPLACED'
        )
    )
);

CREATE UNIQUE INDEX api_client_credentials_one_live_idx
    ON api_client_credentials (api_client_id)
    WHERE revoked_at IS NULL;

CREATE TABLE external_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_client_credential_id UUID NOT NULL
        REFERENCES api_client_credentials(id) ON DELETE RESTRICT,
    token_hash BYTEA NOT NULL
        CONSTRAINT external_access_tokens_token_hash_length_check
        CHECK (octet_length(token_hash) = 32),
    issued_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    CONSTRAINT external_access_tokens_token_hash_key UNIQUE (token_hash),
    CONSTRAINT external_access_tokens_expiry_check CHECK (expires_at > issued_at)
);

CREATE INDEX external_access_tokens_credential_live_idx
    ON external_access_tokens (api_client_credential_id)
    WHERE revoked_at IS NULL;

ALTER TABLE api_client_management_audit
    ADD COLUMN credential_id UUID NULL;

ALTER TABLE api_client_management_audit
    ADD CONSTRAINT api_client_management_audit_credential_id_fkey
    FOREIGN KEY (credential_id) REFERENCES api_client_credentials(id) ON DELETE RESTRICT;

CREATE INDEX api_client_management_audit_credential_idx
    ON api_client_management_audit (credential_id, created_at DESC, id DESC)
    WHERE credential_id IS NOT NULL;

ALTER TABLE api_client_management_audit
    DROP CONSTRAINT api_client_management_audit_action_check;

ALTER TABLE api_client_management_audit
    ADD CONSTRAINT api_client_management_audit_action_check CHECK (action IN (
        'CLIENT_CREATED',
        'CLIENT_METADATA_UPDATED',
        'CLIENT_SCOPES_UPDATED',
        'CLIENT_ACTIVATED',
        'CLIENT_SUSPENDED',
        'CLIENT_REVOKED',
        'CREDENTIAL_ISSUED',
        'CREDENTIAL_REVOKED'
    ));

ALTER TABLE api_client_management_audit
    ADD CONSTRAINT api_client_management_audit_credential_context_check CHECK (
        (
            action IN ('CREDENTIAL_ISSUED', 'CREDENTIAL_REVOKED')
            AND credential_id IS NOT NULL
        )
        OR (
            action NOT IN ('CREDENTIAL_ISSUED', 'CREDENTIAL_REVOKED')
            AND credential_id IS NULL
        )
    );
