CREATE TABLE booking_contacts (
    seat_hold_id UUID PRIMARY KEY REFERENCES seat_holds(id) ON DELETE CASCADE,
    email TEXT NOT NULL CHECK (length(btrim(email)) > 0),
    preferred_locale TEXT NOT NULL CHECK (preferred_locale IN ('EN', 'TH')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE booking_confirmation_email_outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_attempt_id UUID NOT NULL REFERENCES payment_attempts(id) ON DELETE CASCADE,
    notification_type TEXT NOT NULL DEFAULT 'BOOKING_CONFIRMATION'
        CHECK (notification_type = 'BOOKING_CONFIRMATION'),
    recipient_email TEXT NOT NULL CHECK (length(btrim(recipient_email)) > 0),
    locale TEXT NOT NULL CHECK (locale IN ('EN', 'TH')),
    status TEXT NOT NULL DEFAULT 'PENDING'
        CHECK (status IN ('PENDING', 'IN_FLIGHT', 'SENT', 'PERMANENTLY_FAILED')),
    attempt_count SMALLINT NOT NULL DEFAULT 0 CHECK (attempt_count >= 0 AND attempt_count <= 6),
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    lease_until TIMESTAMPTZ,
    provider_message_id TEXT,
    last_error_code TEXT,
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (payment_attempt_id, notification_type),
    CHECK ((status = 'SENT') = (sent_at IS NOT NULL))
);

CREATE INDEX idx_booking_confirmation_email_outbox_due
    ON booking_confirmation_email_outbox (next_attempt_at)
    WHERE status IN ('PENDING', 'IN_FLIGHT');
