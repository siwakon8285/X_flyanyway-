-- Branch 22 records only successful staff booking mutations. The cancellation
-- row remains the financial/inventory source of truth; this table adds durable
-- actor evidence without copying passenger or payment-instrument data.
CREATE TABLE booking_operations_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_staff_user_id UUID NOT NULL,
    actor_email TEXT NOT NULL CHECK (length(btrim(actor_email)) > 0),
    ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE RESTRICT,
    booking_reference TEXT NOT NULL CHECK (booking_reference ~ '^XF[A-Z2-9]{8}$'),
    cancellation_id UUID NOT NULL UNIQUE REFERENCES booking_cancellations(id) ON DELETE RESTRICT,
    action TEXT NOT NULL CHECK (action = 'STAFF_BOOKING_CANCELLED'),
    before_state JSONB NOT NULL,
    after_state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX booking_operations_audit_booking_idx
    ON booking_operations_audit (ticket_id, created_at DESC);
