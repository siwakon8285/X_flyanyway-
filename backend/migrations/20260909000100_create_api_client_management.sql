-- Branch 24 registers approved external systems and their future allowed scopes.
-- It intentionally creates no credentials, secrets, tokens, or external API access.
CREATE TABLE api_scope_catalog (
    code TEXT PRIMARY KEY CHECK (code ~ '^[a-z][a-z0-9_]*:[a-z][a-z0-9_]*$'),
    description TEXT NOT NULL CHECK (length(btrim(description)) BETWEEN 1 AND 200),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO api_scope_catalog (code, description) VALUES
    ('flights:read', 'Future read access to approved flight data'),
    ('analytics:read', 'Future read access to approved aggregate analytics');

CREATE TABLE api_clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id TEXT NOT NULL UNIQUE CHECK (client_id ~ '^XFC[A-HJ-NP-Z2-9]{16}$'),
    display_name TEXT NOT NULL CHECK (
        display_name = btrim(display_name)
        AND length(display_name) BETWEEN 1 AND 100
    ),
    description TEXT CHECK (
        description IS NULL OR (
            description = btrim(description)
            AND length(description) BETWEEN 1 AND 500
        )
    ),
    status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'SUSPENDED', 'REVOKED')),
    version BIGINT NOT NULL DEFAULT 1 CHECK (version > 0),
    created_by_staff_user_id UUID NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    updated_by_staff_user_id UUID NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX api_clients_management_order_idx
    ON api_clients (updated_at DESC, client_id);
CREATE INDEX api_clients_status_order_idx
    ON api_clients (status, updated_at DESC, client_id);
CREATE INDEX api_clients_name_trgm_idx
    ON api_clients USING GIN (lower(display_name) gin_trgm_ops);

CREATE TABLE api_client_allowed_scopes (
    api_client_id UUID NOT NULL REFERENCES api_clients(id) ON DELETE RESTRICT,
    scope_code TEXT NOT NULL REFERENCES api_scope_catalog(code) ON DELETE RESTRICT,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by_staff_user_id UUID NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    PRIMARY KEY (api_client_id, scope_code)
);

CREATE INDEX api_client_allowed_scopes_scope_idx
    ON api_client_allowed_scopes (scope_code, api_client_id);

CREATE TABLE api_client_management_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_client_id UUID NOT NULL REFERENCES api_clients(id) ON DELETE RESTRICT,
    actor_staff_user_id UUID NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    action TEXT NOT NULL CHECK (action IN (
        'CLIENT_CREATED',
        'CLIENT_METADATA_UPDATED',
        'CLIENT_SCOPES_UPDATED',
        'CLIENT_ACTIVATED',
        'CLIENT_SUSPENDED',
        'CLIENT_REVOKED'
    )),
    before_state JSONB,
    after_state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX api_client_management_audit_client_idx
    ON api_client_management_audit (api_client_id, created_at DESC, id DESC);
