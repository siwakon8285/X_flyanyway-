CREATE TABLE staff_security_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action TEXT NOT NULL CHECK (
        action IN (
            'STAFF_LOGIN_SUCCEEDED',
            'STAFF_SESSION_REVOKED',
            'STAFF_AUTHZ_DENIED'
        )
    ),
    actor_staff_user_id UUID NOT NULL,
    session_id UUID NOT NULL,
    permission_code TEXT,
    request_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT staff_security_audit_action_context_check CHECK (
        (
            action IN ('STAFF_LOGIN_SUCCEEDED', 'STAFF_SESSION_REVOKED')
            AND permission_code IS NULL
        )
        OR (
            action = 'STAFF_AUTHZ_DENIED'
            AND permission_code IS NOT NULL
            AND permission_code !~ '^[[:space:]]'
            AND permission_code !~ '[[:space:]]$'
            AND permission_code ~ '[^[:space:]]'
            AND octet_length(permission_code) BETWEEN 1 AND 64
        )
    )
);

CREATE UNIQUE INDEX staff_security_audit_login_session_idx
    ON staff_security_audit (session_id)
    WHERE action = 'STAFF_LOGIN_SUCCEEDED';

CREATE UNIQUE INDEX staff_security_audit_revoked_session_idx
    ON staff_security_audit (session_id)
    WHERE action = 'STAFF_SESSION_REVOKED';

CREATE UNIQUE INDEX staff_security_audit_authz_dedup_idx
    ON staff_security_audit (actor_staff_user_id, session_id, permission_code)
    WHERE action = 'STAFF_AUTHZ_DENIED';

CREATE INDEX staff_security_audit_request_idx
    ON staff_security_audit (request_id);

CREATE INDEX staff_security_audit_created_idx
    ON staff_security_audit (created_at DESC, id DESC);
