CREATE TABLE tickets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_attempt_id UUID NOT NULL UNIQUE REFERENCES payment_attempts(id) ON DELETE RESTRICT,
    booking_reference TEXT NOT NULL UNIQUE CHECK (booking_reference ~ '^XF[A-Z2-9]{8}$'),
    ticket_number TEXT NOT NULL UNIQUE CHECK (ticket_number ~ '^XFT[A-Z2-9]{12}$'),
    status TEXT NOT NULL CHECK (status IN ('ISSUED', 'CANCELLED')),
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cancelled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK ((status = 'CANCELLED') = (cancelled_at IS NOT NULL))
);

CREATE INDEX idx_tickets_booking_reference ON tickets (booking_reference);
