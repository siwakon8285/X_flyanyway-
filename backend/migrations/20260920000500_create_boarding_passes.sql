-- Minimal staff-assisted check-in state. This table is intentionally separate
-- from the booking-level E-Ticket and stores only operational snapshots.
CREATE TABLE boarding_passes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE RESTRICT,
    passenger_ordinal SMALLINT NOT NULL CHECK (passenger_ordinal > 0),
    flight_instance_id UUID NOT NULL REFERENCES flight_instances(id) ON DELETE RESTRICT,
    seat_snapshot TEXT NOT NULL CHECK (seat_snapshot ~ '^[1-9][0-9]*[A-Z]+$'),
    cabin_snapshot TEXT NOT NULL CHECK (cabin_snapshot IN ('economy', 'premium-economy', 'business', 'first')),
    checked_in_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    issued_by_staff_user_id UUID NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (ticket_id, passenger_ordinal)
);

CREATE INDEX boarding_passes_flight_instance_idx
    ON boarding_passes (flight_instance_id, issued_at DESC);

CREATE TABLE boarding_pass_operations_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    boarding_pass_id UUID NOT NULL UNIQUE REFERENCES boarding_passes(id) ON DELETE RESTRICT,
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE RESTRICT,
    passenger_ordinal SMALLINT NOT NULL CHECK (passenger_ordinal > 0),
    actor_staff_user_id UUID NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    action TEXT NOT NULL CHECK (action = 'BOARDING_PASS_ISSUED'),
    before_state JSONB NOT NULL,
    after_state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX boarding_pass_operations_audit_ticket_idx
    ON boarding_pass_operations_audit (ticket_id, created_at DESC);

INSERT INTO permissions (code, description)
VALUES ('boarding_passes:issue', 'Check in passengers and issue Boarding Passes')
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_code, permission_code)
VALUES ('TICKET_PASSENGER_OPERATIONS', 'boarding_passes:issue')
ON CONFLICT (role_code, permission_code) DO NOTHING;

GRANT SELECT ON TABLE
    public.boarding_passes,
    public.boarding_pass_operations_audit
TO x_fly_runtime;

GRANT INSERT (
    ticket_id,
    passenger_ordinal,
    flight_instance_id,
    seat_snapshot,
    cabin_snapshot,
    checked_in_at,
    issued_at,
    issued_by_staff_user_id
)
ON TABLE public.boarding_passes
TO x_fly_runtime;

GRANT INSERT (
    boarding_pass_id,
    ticket_id,
    passenger_ordinal,
    actor_staff_user_id,
    action,
    before_state,
    after_state
)
ON TABLE public.boarding_pass_operations_audit
TO x_fly_runtime;
