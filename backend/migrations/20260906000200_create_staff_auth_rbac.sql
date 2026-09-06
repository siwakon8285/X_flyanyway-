CREATE TABLE staff_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL CHECK (length(password_hash) > 0),
    status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'DISABLED')),
    disabled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (email = lower(btrim(email))),
    CHECK (length(email) BETWEEN 3 AND 254),
    CHECK (position('@' IN email) > 1),
    CHECK ((status = 'ACTIVE') = (disabled_at IS NULL))
);

CREATE TABLE roles (
    code TEXT PRIMARY KEY CHECK (code ~ '^[A-Z][A-Z0-9_]*$'),
    display_name TEXT NOT NULL CHECK (length(btrim(display_name)) > 0),
    description TEXT NOT NULL CHECK (length(btrim(description)) > 0),
    is_system BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE permissions (
    code TEXT PRIMARY KEY CHECK (code ~ '^[a-z][a-z0-9_]*:[a-z][a-z0-9_]*$'),
    description TEXT NOT NULL CHECK (length(btrim(description)) > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE staff_user_roles (
    staff_user_id UUID NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    role_code TEXT NOT NULL REFERENCES roles(code) ON DELETE RESTRICT,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by_staff_user_id UUID REFERENCES staff_users(id) ON DELETE SET NULL,
    PRIMARY KEY (staff_user_id, role_code)
);

CREATE INDEX staff_user_roles_role_idx ON staff_user_roles (role_code, staff_user_id);

CREATE TABLE role_permissions (
    role_code TEXT NOT NULL REFERENCES roles(code) ON DELETE RESTRICT,
    permission_code TEXT NOT NULL REFERENCES permissions(code) ON DELETE RESTRICT,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by_staff_user_id UUID REFERENCES staff_users(id) ON DELETE SET NULL,
    PRIMARY KEY (role_code, permission_code)
);

CREATE INDEX role_permissions_permission_idx
    ON role_permissions (permission_code, role_code);

CREATE TABLE staff_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    staff_user_id UUID NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    token_hash BYTEA NOT NULL UNIQUE CHECK (octet_length(token_hash) = 32),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    revocation_reason TEXT,
    CHECK (expires_at > created_at),
    CHECK (revoked_at IS NOT NULL OR revocation_reason IS NULL)
);

CREATE INDEX staff_sessions_active_user_idx
    ON staff_sessions (staff_user_id, expires_at)
    WHERE revoked_at IS NULL;

CREATE TABLE staff_login_throttles (
    identifier_hash BYTEA PRIMARY KEY CHECK (octet_length(identifier_hash) = 32),
    failure_count INTEGER NOT NULL CHECK (failure_count >= 0),
    window_started_at TIMESTAMPTZ NOT NULL,
    blocked_until TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO roles (code, display_name, description) VALUES
    ('EXECUTIVE', 'Executive / Owner', 'Business visibility and planning'),
    ('FLIGHT_MANAGER', 'Flight Manager', 'Flight design and management'),
    ('BOOKING_OPERATIONS', 'Booking Operations', 'Customer booking operations'),
    ('TICKET_PASSENGER_OPERATIONS', 'Ticket / Passenger Operations', 'Ticketing and passenger operations'),
    ('BAGGAGE_STAFF', 'Baggage Staff', 'Limited baggage-operation context'),
    ('API_ADMIN', 'API Admin', 'External-system client administration'),
    ('SYSTEM_ADMIN', 'System Admin', 'Internal staff access administration');

INSERT INTO permissions (code, description) VALUES
    ('dashboard:read', 'Read the future executive dashboard'),
    ('analytics:read', 'Read future business analytics'),
    ('reports:read', 'Read future reports'),
    ('flights:read', 'Read flight information'),
    ('flights:write', 'Create or modify flights'),
    ('bookings:read', 'Read booking information'),
    ('bookings:manage', 'Perform approved booking operations'),
    ('tickets:read', 'Read ticket information'),
    ('tickets:print', 'Print authorized ticket information'),
    ('passengers:read', 'Read operational passenger information'),
    ('bookings:read_limited', 'Read baggage-required booking fields'),
    ('passengers:read_limited', 'Read baggage-required passenger fields'),
    ('baggage_context:read', 'Read baggage-operation context'),
    ('api_clients:read', 'Read future API clients'),
    ('api_clients:manage', 'Manage future API clients'),
    ('staff:read', 'Read staff access information'),
    ('staff:manage', 'Manage staff access'),
    ('roles:read', 'Read role definitions'),
    ('roles:manage', 'Manage role grants');

INSERT INTO role_permissions (role_code, permission_code) VALUES
    ('EXECUTIVE', 'dashboard:read'),
    ('EXECUTIVE', 'analytics:read'),
    ('EXECUTIVE', 'reports:read'),
    ('FLIGHT_MANAGER', 'flights:read'),
    ('FLIGHT_MANAGER', 'flights:write'),
    ('BOOKING_OPERATIONS', 'bookings:read'),
    ('BOOKING_OPERATIONS', 'bookings:manage'),
    ('TICKET_PASSENGER_OPERATIONS', 'tickets:read'),
    ('TICKET_PASSENGER_OPERATIONS', 'tickets:print'),
    ('TICKET_PASSENGER_OPERATIONS', 'passengers:read'),
    ('BAGGAGE_STAFF', 'flights:read'),
    ('BAGGAGE_STAFF', 'bookings:read_limited'),
    ('BAGGAGE_STAFF', 'passengers:read_limited'),
    ('BAGGAGE_STAFF', 'baggage_context:read'),
    ('API_ADMIN', 'api_clients:read'),
    ('API_ADMIN', 'api_clients:manage'),
    ('SYSTEM_ADMIN', 'staff:read'),
    ('SYSTEM_ADMIN', 'staff:manage'),
    ('SYSTEM_ADMIN', 'roles:read'),
    ('SYSTEM_ADMIN', 'roles:manage');
