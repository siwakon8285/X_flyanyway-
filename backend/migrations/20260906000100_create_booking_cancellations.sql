ALTER TABLE payment_attempt_seats
    ADD COLUMN released_at TIMESTAMPTZ;

ALTER TABLE payment_attempt_seats
    DROP CONSTRAINT payment_attempt_seats_flight_seat_id_key;

CREATE UNIQUE INDEX payment_attempt_seats_one_active_owner_idx
    ON payment_attempt_seats (flight_seat_id)
    WHERE released_at IS NULL;

CREATE TABLE booking_cancellations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_id UUID NOT NULL UNIQUE REFERENCES tickets(id) ON DELETE RESTRICT,
    payment_attempt_id UUID NOT NULL UNIQUE REFERENCES payment_attempts(id) ON DELETE RESTRICT,
    refund_provider TEXT NOT NULL CHECK (refund_provider IN ('STRIPE', 'MOCK_BITCOIN')),
    refund_status TEXT NOT NULL DEFAULT 'PENDING'
        CHECK (refund_status IN ('PENDING', 'IN_FLIGHT', 'PROCESSING', 'SUCCEEDED', 'REQUIRES_ATTENTION')),
    refund_amount BIGINT NOT NULL CHECK (refund_amount > 0),
    currency_code TEXT NOT NULL CHECK (currency_code ~ '^[A-Z]{3}$'),
    provider_refund_id TEXT,
    attempt_count SMALLINT NOT NULL DEFAULT 0 CHECK (attempt_count BETWEEN 0 AND 6),
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    lease_until TIMESTAMPTZ,
    lease_token UUID,
    last_error_code TEXT,
    requested_at TIMESTAMPTZ NOT NULL,
    cancelled_at TIMESTAMPTZ NOT NULL,
    refunded_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK ((refund_status = 'IN_FLIGHT') = (lease_until IS NOT NULL AND lease_token IS NOT NULL)),
    CHECK ((refund_status = 'SUCCEEDED') = (refunded_at IS NOT NULL))
);

CREATE UNIQUE INDEX booking_cancellations_provider_refund_unique_idx
    ON booking_cancellations (refund_provider, provider_refund_id)
    WHERE provider_refund_id IS NOT NULL;

CREATE INDEX booking_cancellations_due_idx
    ON booking_cancellations (next_attempt_at, created_at)
    WHERE refund_status IN ('PENDING', 'PROCESSING', 'IN_FLIGHT');

CREATE TABLE stripe_refund_events (
    stripe_event_id TEXT PRIMARY KEY,
    cancellation_id UUID NOT NULL REFERENCES booking_cancellations(id) ON DELETE RESTRICT,
    refund_id TEXT NOT NULL,
    event_type TEXT NOT NULL CHECK (event_type IN ('refund.created', 'refund.updated', 'refund.failed')),
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
